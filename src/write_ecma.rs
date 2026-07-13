//! ECMA formatting variant of `crate::write`.
//!
//! The conversion pipeline mirrors `crate::write` from zmij@7426670. Sections
//! marked as ECMA policy intentionally differ from the core formatter.

use crate::{
    digits2, div10, ptr, to_decimal, umul128_hi64, ExpFloatShuffleTable, ExpStringTable,
    FloatTraits, ToDecimalResult, DIV100_EXP, DIV100_SIG, STATIC_DATA, USE_UMUL128_HI64,
};
#[cfg(all(any(target_arch = "aarch64", target_arch = "x86_64"), not(miri)))]
use core::arch::asm;
use core::ops::RangeInclusive;

#[cfg(feature = "no-panic")]
use no_panic::no_panic;

pub(crate) const BUFFER_SIZE: usize = 25;
pub(crate) const INFINITY: &str = "Infinity";
pub(crate) const NEG_INFINITY: &str = "-Infinity";

const ECMA_FIXED_DEC_EXP: RangeInclusive<i32> = -6..=20;

/// Writes the shortest correctly rounded decimal representation of `value` to
/// `buffer`. `buffer` should point to a buffer of size `BUFFER_SIZE` or larger.
///
/// ECMA policy: formatting follows the ECMA-262 Number::toString specification.
#[cfg_attr(feature = "no-panic", no_panic)]
pub(crate) unsafe fn write_ecma<Float>(value: Float, mut buffer: *mut u8) -> *mut u8
where
    Float: FloatTraits,
{
    let bits = value.to_bits();
    // It is beneficial to extract exponent and significand early.
    let bin_exp = Float::get_exp(bits); // binary exponent
    let bin_sig = Float::get_sig(bits); // binary significand

    // ECMA policy: -0 and +0 both return "0".
    if bin_exp == 0 && bin_sig == Float::SigType::from(0) {
        return unsafe {
            *buffer = b'0';
            buffer.add(1)
        };
    }

    unsafe {
        *buffer = b'-';
    }
    buffer = unsafe { buffer.add(usize::from(Float::is_negative(bits))) };

    #[allow(unused_mut)]
    let mut d = ptr::addr_of!(STATIC_DATA);
    let d = unsafe {
        // Load constants from memory.
        #[cfg(all(any(target_arch = "aarch64", target_arch = "x86_64"), not(miri)))]
        asm!("/*{0}*/", inout(reg) d);
        &*d
    };
    let threshold = if Float::NUM_BITS == 64 {
        d.threshold.get()
    } else {
        10_000_000
    };

    let mut dec;
    if bin_exp == 0 {
        dec = to_decimal::<Float, Float::SigType>(bin_sig, 1, true, d);
        let mut dec_sig =
            dec.sig * 10 + (-i64::from(dec.has_last_digit) & i64::from(dec.last_digit));
        let mut dec_exp = dec.exp;
        while dec_sig < threshold as i64 {
            dec_sig *= 10;
            dec_exp -= 1;
        }
        let d = div10(dec_sig as u64);
        let last_digit = dec_sig - d as i64 * 10;
        dec = ToDecimalResult {
            sig: d as i64,
            exp: dec_exp,
            last_digit: last_digit as u8,
            has_last_digit: last_digit != 0,
        };
    } else {
        dec = to_decimal::<Float, Float::SigType>(
            bin_sig | Float::IMPLICIT_BIT,
            bin_exp,
            bin_sig != Float::SigType::from(0),
            d,
        );
    }
    let mut has_last_digit = dec.has_last_digit;
    let has_extra_digit = dec.sig >= threshold as i64;
    let mut dec_exp = dec.exp + Float::MAX_DIGITS10 as i32 - 2 + i32::from(has_extra_digit);
    if Float::NUM_BITS == 32 && dec.sig < 1_000_000 {
        dec.sig = 10 * dec.sig + (-i64::from(has_last_digit) & i64::from(dec.last_digit));
        has_last_digit = false;
        dec_exp -= 1;
    }

    // Write significand.
    let dig = Float::to_digits(dec.sig as u64, d);

    // ECMA policy: the exponential fast path uses the ECMA fixed-notation range.
    if Float::NUM_BITS == 32
        && ExpFloatShuffleTable::ENABLE
        && !ECMA_FIXED_DEC_EXP.contains(&dec_exp)
    {
        unsafe {
            let exp_data = *d
                .exp_strings
                .data
                .get_unchecked((dec_exp + ExpStringTable::OFFSET) as usize);
            return Float::write_exp_float_simd(
                buffer,
                &dig,
                i32::from(dec.last_digit),
                has_last_digit,
                has_extra_digit,
                exp_data,
                d,
            );
        }
    }

    let bcd_size = if Float::NUM_BITS == 64 { 16 } else { 8 };
    unsafe {
        buffer
            .add(usize::from(has_extra_digit))
            .cast::<Float::DecDigitsType>()
            .write_unaligned(dig.digits);
        buffer
            .add(usize::from(has_extra_digit) + bcd_size)
            .write(b'0' + dec.last_digit);
    }
    let length = usize::from(has_extra_digit)
        + if has_last_digit {
            bcd_size + 1
        } else {
            dig.num_digits
        }
        - 1;

    // ECMA policy: use fixed notation when the decimal point is in [-5, 21].
    // In zmij, dec_exp represents (decimal_point - 1), so we check [-6, 20].
    if ECMA_FIXED_DEC_EXP.contains(&dec_exp) {
        if length as i32 - 1 <= dec_exp {
            // ECMA policy: integer output does not receive a ".0" suffix.
            // 1234e7 -> 12340000000
            return unsafe {
                ptr::copy(buffer.add(1), buffer, length);
                let padding = dec_exp as usize + 1 - length;
                ptr::write_bytes(buffer.add(length), b'0', padding);
                buffer.add(dec_exp as usize + 1)
            };
        } else if 0 <= dec_exp {
            // 1234e-2 -> 12.34
            return unsafe {
                ptr::copy(buffer.add(1), buffer, dec_exp as usize + 1);
                *buffer.add(dec_exp as usize + 1) = b'.';
                buffer.add(length + 1)
            };
        } else {
            // 1234e-6 -> 0.001234
            return unsafe {
                ptr::copy(buffer.add(1), buffer.add((1 - dec_exp) as usize), length);
                ptr::write_bytes(buffer, b'0', (1 - dec_exp) as usize);
                *buffer.add(1) = b'.';
                buffer.add((1 - dec_exp) as usize + length)
            };
        }
    }

    unsafe {
        // 1234e30 -> 1.234e33
        *buffer = *buffer.add(1);
        *buffer.add(1) = b'.';
    }
    buffer = unsafe { buffer.add(length + usize::from(length > 1)) };

    // Write exponent.
    if ExpStringTable::ENABLE {
        let mut exp_data = unsafe {
            *d.exp_strings
                .data
                .get_unchecked((dec_exp + ExpStringTable::OFFSET) as usize)
        };
        let len = (exp_data >> 48) as usize;
        exp_data = exp_data.to_le();
        unsafe {
            ptr::copy_nonoverlapping(
                ptr::addr_of!(exp_data).cast::<u8>(),
                buffer,
                if Float::MAX_10_EXP >= 100 { 5 } else { 4 },
            );
            return buffer.add(len);
        }
    }
    let sign_ptr = buffer;
    let e_sign = if dec_exp >= 0 {
        (u16::from(b'+') << 8) | u16::from(b'e')
    } else {
        (u16::from(b'-') << 8) | u16::from(b'e')
    };
    buffer = unsafe { buffer.add(1) };
    dec_exp = if dec_exp >= 0 { dec_exp } else { -dec_exp };
    buffer = unsafe { buffer.add(usize::from(dec_exp >= 10)) };
    if Float::MAX_10_EXP >= 100 {
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
        dec_exp -= (digit * 100) as i32;
    }
    unsafe {
        buffer
            .cast::<u16>()
            .write_unaligned(*digits2(dec_exp as usize));
        sign_ptr.cast::<u16>().write_unaligned(e_sign.to_le());
        buffer.add(2)
    }
}
