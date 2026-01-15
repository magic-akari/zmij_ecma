use crate::*;
use core::ptr;

/// Writes the shortest correctly rounded decimal representation of `value` to
/// `buffer` following ECMA-262 Number::toString specification.
#[cfg_attr(feature = "no-panic", no_panic)]
pub(crate) unsafe fn write_ecma<Float>(value: Float, mut buffer: *mut u8) -> *mut u8
where
    Float: FloatTraits,
{
    let bits = value.to_bits();
    // It is beneficial to extract exponent and significand early.
    let bin_exp = Float::get_exp(bits); // binary exponent
    let mut bin_sig = Float::get_sig(bits); // binary significand

    let regular = bin_sig != Float::SigType::from(0);
    let subnormal = bin_exp == 0;
    if bin_exp == 0 {
        if bin_sig == Float::SigType::from(0) {
            // ECMA-262: -0 and +0 both return "0"
            return unsafe {
                *buffer = b'0';
                buffer.add(1)
            };
        }
        bin_sig |= Float::IMPLICIT_BIT;
    }
    bin_sig ^= Float::IMPLICIT_BIT;

    // Handle negative sign (but not for -0, which is already handled above)

    if Float::is_negative(bits) {
        unsafe {
            *buffer = b'-';
        }
        buffer = unsafe { buffer.add(1) };
    }

    // Here be 🐉s.
    let mut dec = to_decimal::<Float, Float::SigType>(bin_sig, bin_exp, regular, subnormal);
    let mut dec_exp = dec.exp;

    // Write significand.
    let end = if Float::NUM_BITS == 64 {
        let has17digits = dec.sig >= 10_000_000_000_000_000;
        dec_exp += Float::MAX_DIGITS10 as i32 - 2 + i32::from(has17digits);
        unsafe { write_significand17(buffer.add(1), dec.sig as u64, has17digits) }
    } else {
        if dec.sig < 10_000_000 {
            dec.sig *= 10;
            dec_exp -= 1;
        }
        let has9digits = dec.sig >= 100_000_000;
        dec_exp += Float::MAX_DIGITS10 as i32 - 2 + i32::from(has9digits);
        unsafe { write_significand9(buffer.add(1), dec.sig as u32, has9digits) }
    };

    let length = unsafe { end.offset_from(buffer.add(1)) } as usize;

    // ECMA-262 uses n (decimal_point) in range [-5, 21] for non-exponential notation.
    // In zmij, dec_exp represents (decimal_point - 1), so we check [-6, 20].
    if (-6..=20).contains(&dec_exp) {
        if length as i32 - 1 <= dec_exp {
            // ECMA-262 step 6: n >= k, output integer format (no .0 suffix)
            // Example: 1234e7 -> "12340000000"
            return unsafe {
                ptr::copy(buffer.add(1), buffer, length);
                // Add padding zeros if needed
                let padding = dec_exp as usize + 1 - length;
                ptr::write_bytes(buffer.add(length), b'0', padding);
                buffer.add(dec_exp as usize + 1)
            };
        } else if 0 <= dec_exp {
            // ECMA-262 step 7: 0 < n < k, output decimal format
            // Example: 1234e-2 -> "12.34"
            return unsafe {
                ptr::copy(buffer.add(1), buffer, dec_exp as usize + 1);
                *buffer.add(dec_exp as usize + 1) = b'.';
                buffer.add(length + 1)
            };
        } else {
            // ECMA-262 step 8: -5 <= n <= 0, output "0.00...0xxx" format
            // Example: 1234e-6 -> "0.001234"
            return unsafe {
                ptr::copy(buffer.add(1), buffer.add((1 - dec_exp) as usize), length);
                ptr::write_bytes(buffer, b'0', (1 - dec_exp) as usize);
                *buffer.add(1) = b'.';
                buffer.add((1 - dec_exp) as usize + length)
            };
        }
    }

    // ECMA-262 steps 9-10: scientific notation

    // 1234e30 -> 1.234e+33
    unsafe {
        *buffer = *buffer.add(1);
        *buffer.add(1) = b'.';
    }
    buffer = unsafe { buffer.add(length + usize::from(length > 1)) };

    // Write exponent.
    let sign_ptr = buffer;
    let e_sign = if dec_exp >= 0 {
        (u16::from(b'+') << 8) | u16::from(b'e')
    } else {
        (u16::from(b'-') << 8) | u16::from(b'e')
    };
    buffer = unsafe { buffer.add(1) };
    dec_exp = if dec_exp >= 0 { dec_exp } else { -dec_exp };
    buffer = unsafe { buffer.add(usize::from(dec_exp >= 10)) };
    if Float::MIN_10_EXP >= -99 && Float::MAX_10_EXP <= 99 {
        unsafe {
            buffer
                .cast::<u16>()
                .write_unaligned(*digits2(dec_exp as usize));
            sign_ptr.cast::<u16>().write_unaligned(e_sign.to_le());
            return buffer.add(2);
        }
    }

    // digit = dec_exp / 100
    let digit = if USE_UMUL128_HI64 {
        umul128_hi64(dec_exp as u64, 0x290000000000000) as u32
    } else {
        (dec_exp as u32 * DIV100_SIG) >> DIV100_EXP
    };
    unsafe {
        *buffer = b'0' + digit as u8;
    }
    buffer = unsafe { buffer.add(usize::from(dec_exp >= 100)) };
    unsafe {
        buffer
            .cast::<u16>()
            .write_unaligned(*digits2((dec_exp as u32 - digit * 100) as usize));
        sign_ptr.cast::<u16>().write_unaligned(e_sign.to_le());
        buffer.add(2)
    }
}
