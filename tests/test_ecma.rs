#![allow(clippy::unreadable_literal)]

use zmij_ecma as zmij;

fn number_to_string(value: f64) -> String {
    zmij::Buffer::new().format(value).to_owned()
}

#[cfg(test)]
mod number_to_string_test {
    use super::number_to_string;

    #[test]
    fn normal() {
        assert_eq!(number_to_string(6.62607015e-34), "6.62607015e-34");

        // Exact half-ulp tie when rounding to nearest integer.
        assert_eq!(number_to_string(5.444310685350916e+14), "544431068535091.6");
    }

    #[test]
    fn subnormal() {
        assert_eq!(number_to_string(0.0f64.next_up()), "5e-324");
        assert_eq!(number_to_string(1e-323), "1e-323");
        assert_eq!(number_to_string(1.2e-322), "1.2e-322");
        assert_eq!(number_to_string(1.24e-322), "1.24e-322");
        assert_eq!(number_to_string(1.234e-320), "1.234e-320");
    }

    #[test]
    fn all_irregular() {
        for exp in 1..0x3ff {
            let bits = exp << 52;
            let value = f64::from_bits(bits);

            assert_eq!(number_to_string(value), ryu_js::Buffer::new().format(value));
        }
    }

    #[test]
    fn all_exponents() {
        for exp in 0..=0x3ff {
            let bits = (exp << 52) | 1;
            let value = f64::from_bits(bits);

            assert_eq!(number_to_string(value), ryu_js::Buffer::new().format(value));
        }
    }

    #[test]
    fn small_int() {
        assert_eq!(number_to_string(1.0), "1");
    }

    #[test]
    fn zero() {
        assert_eq!(number_to_string(0.0), "0");
        assert_eq!(number_to_string(-0.0), "0");
    }

    #[test]
    fn inf() {
        assert_eq!(number_to_string(f64::INFINITY), "Infinity");
    }

    #[test]
    fn nan() {
        assert_eq!(number_to_string(f64::NAN.copysign(-1.0)), "NaN");
    }

    #[test]
    fn shorter() {
        // A possibly shorter underestimate is picked (u' in Schubfach).
        assert_eq!(
            number_to_string(-4.932096661796888e-226),
            "-4.932096661796888e-226"
        );

        // A possibly shorter overestimate is picked (w' in Schubfach).
        assert_eq!(
            number_to_string(3.439070283483335e+35),
            "3.439070283483335e+35"
        );
    }

    #[test]
    fn single_candidate() {
        // Only an underestimate is in the rounding region (u in Schubfach).
        assert_eq!(
            number_to_string(6.606854224493745e-17),
            "6.606854224493745e-17"
        );

        // Only an overestimate is in the rounding region (w in Schubfach).
        assert_eq!(
            number_to_string(6.079537928711555e+61),
            "6.079537928711555e+61"
        );
    }

    #[test]
    fn null_terminated() {
        assert_eq!(number_to_string(9.061488e15), "9061488000000000");
        assert_eq!(number_to_string(f64::NAN.copysign(1.0)), "NaN");
    }

    #[test]
    fn no_buffer() {
        assert_eq!(number_to_string(6.62607015e-34), "6.62607015e-34");
    }

    #[test]
    fn test_ecma262_compliance() {
        assert_eq!(number_to_string(f64::NAN), "NaN");
        assert_eq!(number_to_string(f64::INFINITY), "Infinity");
        assert_eq!(number_to_string(f64::NEG_INFINITY), "-Infinity");
        assert_eq!(number_to_string(0.0), "0");
        assert_eq!(number_to_string(9.0), "9");
        assert_eq!(number_to_string(90.0), "90");
        assert_eq!(number_to_string(90.12), "90.12");

        assert_eq!(number_to_string(0.000001), "0.000001");
        assert_eq!(number_to_string(0.0000001), "1e-7");
        assert_eq!(number_to_string(3e50), "3e+50");

        assert_eq!(number_to_string(90.12), "90.12");

        assert_eq!(
            number_to_string(111111111111111111111.0),
            "111111111111111110000"
        );
        assert_eq!(
            number_to_string(1111111111111111111111.0),
            "1.1111111111111111e+21"
        );
        assert_eq!(
            number_to_string(11111111111111111111111.0),
            "1.1111111111111111e+22"
        );

        assert_eq!(number_to_string(0.1), "0.1");
        assert_eq!(number_to_string(0.01), "0.01");
        assert_eq!(number_to_string(0.001), "0.001");
        assert_eq!(number_to_string(0.0001), "0.0001");
        assert_eq!(number_to_string(0.00001), "0.00001");
        assert_eq!(number_to_string(0.000001), "0.000001");
        assert_eq!(number_to_string(0.0000001), "1e-7");
        assert_eq!(number_to_string(0.00000012), "1.2e-7");
        assert_eq!(number_to_string(0.000000123), "1.23e-7");
        assert_eq!(number_to_string(0.00000001), "1e-8");

        assert_eq!(number_to_string(-0.0), "0");
        assert_eq!(number_to_string(-9.0), "-9");
        assert_eq!(number_to_string(-90.12), "-90.12");
        assert_eq!(number_to_string(-0.0000000123), "-1.23e-8");
        assert_eq!(
            number_to_string(-111111111111111111111.0),
            "-111111111111111110000"
        );
        assert_eq!(
            number_to_string(-1111111111111111111111.0),
            "-1.1111111111111111e+21"
        );
        assert_eq!(number_to_string(-0.000000123), "-1.23e-7");

        assert_eq!(
            number_to_string(123456789010111213141516171819.0),
            "1.234567890101112e+29"
        );
    }

    #[test]
    fn max_size_double_to_string() {
        // See: https://viewer.scuttlebot.io/%25LQo5KOMeR%2Baj%2BEj0JVg3qLRqr%2BwiKo74nS8Uz7o0LDM%3D.sha256
        assert_eq!(
            number_to_string(-0.0000015809161985788154),
            "-0.0000015809161985788154"
        );
    }
}

mod ryu_comparison_test {
    use rand::rngs::{SmallRng, SysRng};
    use rand::{Rng as _, SeedableRng as _};
    use zmij_ecma as zmij;

    const N: usize = if cfg!(miri) {
        500
    } else if let b"0" = opt_level::OPT_LEVEL.as_bytes() {
        1_000_000
    } else {
        100_000_000
    };

    #[test]
    fn ryu_comparison() {
        let mut ryu_buffer = ryu_js::Buffer::new();
        let mut zmij_buffer = zmij::Buffer::new();
        let mut rng = SmallRng::try_from_rng(&mut SysRng).unwrap();
        let mut fail = 0;

        for _ in 0..N {
            let bits = rng.next_u64();
            let float = f64::from_bits(bits);
            let ryu = ryu_buffer.format(float);
            let zmij = zmij_buffer.format(float);

            if ryu != zmij {
                eprintln!("RYU={ryu} ZMIJ={zmij}");
                fail += 1;
            }
        }

        assert!(fail == 0, "{fail} mismatches");
    }
}
