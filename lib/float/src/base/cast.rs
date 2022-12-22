use super::float::{Float, RoundingMode, FP32, FP64};
use super::utils;
use super::utils::{expand_mantissa_to_explicit, mask};
use crate::base::float::Category;
use crate::base::float::LossFraction;

impl<const EXPONENT: usize, const MANTISSA: usize> Float<EXPONENT, MANTISSA> {
    pub fn from_u64(val: u64) -> Self {
        if val == 0 {
            return Self::zero(false);
        }

        // Figure out how to shift the input to align the first bit with the
        // msb of the mantissa.
        let lz = val.leading_zeros();
        let size_in_bits = 64 - lz - 1;

        // If we can't adjust the exponent then this is infinity.
        if size_in_bits > Self::get_exp_bounds().1 as u32 {
            return Self::inf(false);
        }

        let mut a = Self::new(false, size_in_bits as i64, val << lz);
        a.normalize(RoundingMode::NearestTiesToEven, LossFraction::ExactlyZero);
        a
    }

    pub fn from_i64(val: i64) -> Self {
        if val < 0 {
            let mut a = Self::from_u64(-val as u64);
            a.set_sign(true);
            return a;
        }

        Self::from_u64(val as u64)
    }

    fn from_bits<const E: usize, const M: usize>(float: u64) -> Self {
        // Extract the biased exponent (wipe the sign and mantissa).
        let biased_exp = ((float >> M) & mask(E) as u64) as i64;
        // Wipe the original exponent and mantissa.
        let sign = (float >> (E + M)) & 1;
        // Wipe the sign and exponent.
        let mantissa = float & mask(M) as u64;

        let sign = sign == 1;

        // Check for NaN/Inf
        if biased_exp == mask(E) as i64 {
            if mantissa == 0 {
                return Self::inf(sign);
            }
            return Self::nan(sign);
        }

        // Check that the loaded value fits within the bounds of the float.
        let exp = biased_exp - utils::compute_ieee745_bias(E) as i64;
        let bounds = Self::get_exp_bounds();
        if exp < bounds.0 {
            return Self::zero(sign);
        } else if exp > bounds.1 {
            return Self::inf(sign);
        }

        let leading_1 = biased_exp != 0;
        let new_mantissa =
            expand_mantissa_to_explicit::<M>(mantissa, leading_1);
        Self::new(sign, exp, new_mantissa)
    }

    pub fn cast<const E: usize, const S: usize>(&self) -> Float<E, S> {
        let mut x = Float::<E, S>::raw(
            self.get_sign(),
            self.get_exp(),
            self.get_mantissa(),
            self.get_category(),
        );
        x.normalize(RoundingMode::NearestTiesToEven, LossFraction::ExactlyZero);
        x
    }

    fn as_native_float<const E: usize, const M: usize>(&self) -> u64 {
        // https://en.wikipedia.org/wiki/IEEE_754
        let mantissa: u64;
        let exp: u64;

        println!("Encoding :");
        self.dump();
        println!("E = {}, M = {}", E, M);

        match self.get_category() {
            Category::Infinity => {
                mantissa = 0;
                exp = mask(E) as u64;
            }
            Category::NaN => {
                mantissa = 1 << (M - 1);
                exp = mask(E) as u64;
            }
            Category::Zero => {
                mantissa = 0;
                exp = 0;
            }
            Category::Normal => {
                let mant = self.get_mantissa();
                let mant = mant << 1; // Clear the explicit '1' bit.
                mantissa = mant >> (64 - M); // Put the mantissa in place.
                println!("bias = {}", Float::<E, M>::get_bias());
                println!("exp = {}", self.get_exp());
                exp = (self.get_exp() + Float::<E, M>::get_bias()) as u64;
            }
        }

        // Encode denormals, get rid of the implicit bit.
        if exp == 0 {
            // mantissa = mantissa >> 1;
        }

        println!("exp = {:x}, m = {:x}", exp, mantissa);

        let mut bits: u64 = self.get_sign() as u64;
        bits <<= E;
        bits |= exp;
        bits <<= M;
        assert!(mantissa <= 1 << M);
        bits |= mantissa;
        bits
    }
    pub fn as_f32(&self) -> f32 {
        let b: FP32 = self.cast();
        b.dump();
        let bits = b.as_native_float::<8, 23>();
        f32::from_bits(bits as u32)
    }
    pub fn as_f64(&self) -> f64 {
        let b: FP64 = self.cast();
        let bits = b.as_native_float::<11, 52>();
        f64::from_bits(bits)
    }

    pub fn from_f32(float: f32) -> Self {
        Self::from_bits::<8, 23>(float.to_bits() as u64)
    }

    pub fn from_f64(float: f64) -> Self {
        Self::from_bits::<11, 52>(float.to_bits())
    }
}

#[test]
fn test_round_trip_native_float_cast() {
    let f = f32::from_bits(0x41700000);
    let a = FP32::from_f32(f);
    assert_eq!(f, a.as_f32());

    let pi = 355. / 113.;
    let a = FP64::from_f64(pi);
    assert_eq!(pi, a.as_f64());

    assert!(FP64::from_f64(f64::NAN).is_nan());
    assert!(!FP64::from_f64(f64::NAN).is_inf());
    assert!(FP64::from_f64(f64::INFINITY).is_inf());
    assert!(!FP64::from_f64(f64::INFINITY).is_nan());
    assert!(FP64::from_f64(f64::NEG_INFINITY).is_inf());

    let a_float = f32::from_bits(0x3f8fffff);
    let a = FP64::from_f32(a_float);
    let b: FP32 = a.cast();
    assert_eq!(a.as_f32(), a_float);
    assert_eq!(b.as_f32(), a_float);

    let f = f32::from_bits(0x000000);
    let a = FP32::from_f32(f);
    a.dump();
    assert!(!a.is_normal());
    assert_eq!(f, a.as_f32());
}

#[test]
fn test_cast_wide_range() {
    for i in 0..(1 << 14) {
        let val = f32::from_bits(i << 16);
        assert!(val.is_finite());
        let a = FP64::from_f32(val);
        let b: FP32 = a.cast();
        let res = b.as_f32();
        assert_eq!(res.to_bits(), (i << 16));
    }
}

#[test]
fn test_cast_easy_ctor() {
    let values = [0x3f8fffff, 0x40800000, 0x3f000000, 0xc60b40ec, 0xbc675793];

    for v in values {
        let output = f32::from_bits(v);
        let a = FP64::from_f32(output);
        let b: FP32 = a.cast();
        assert_eq!(a.as_f32(), output);
        assert_eq!(b.as_f32(), output);
    }
}

#[test]
fn test_cast_from_integers() {
    use super::float::FP16;

    let pi = 355. / 133.;
    let e = 193. / 71.;

    assert_eq!(FP32::from_i64(1 << 32).as_f32(), (1u64 << 32) as f32);
    assert_eq!(FP32::from_i64(1 << 34).as_f32(), (1u64 << 34) as f32);
    assert_eq!(FP32::from_f64(pi).as_f32(), (pi) as f32);
    assert_eq!(FP32::from_f64(e).as_f32(), (e) as f32);
    assert_eq!(FP32::from_u64(8388610).as_f32(), 8388610 as f32);

    for i in 0..(1 << 16) {
        assert_eq!(FP32::from_u64(i << 12).as_f32(), (i << 12) as f32);
    }

    assert_eq!(FP64::from_i64(0).as_f64(), 0.);
    assert_eq!(FP16::from_i64(65500).as_f64(), 65504.0);
    assert_eq!(FP16::from_i64(65504).as_f64(), 65504.0);
    assert_eq!(FP16::from_i64(65535).as_f64(), 65536.0);
    assert_eq!(FP16::from_i64(65536).as_f64(), f64::INFINITY);

    for i in -100..100 {
        let a = FP32::from_i64(i);
        let b = FP32::from_f64(i as f64);
        assert_eq!(a.as_f32(), b.as_f32());
    }
}

#[test]
fn test_cast_zero_nan_inf() {
    assert!(FP64::nan(true).as_f64().is_nan());
    assert_eq!(FP64::zero(false).as_f64(), 0.0);
    assert_eq!(FP64::zero(true).as_f64(), -0.0);

    assert!(FP64::nan(true).is_nan());
    assert!(FP64::inf(true).is_inf());
    {
        let a = FP32::from_f32(f32::from_bits(0x3f8fffff));
        assert!(!a.is_inf());
        assert!(!a.is_nan());
        assert!(!a.is_negative());
    }
    {
        let a = FP32::from_f32(f32::from_bits(0xf48fffff));
        assert!(!a.is_inf());
        assert!(!a.is_nan());
        assert!(a.is_negative());
    }
    {
        let a = FP32::from_f32(f32::from_bits(0xff800000)); // -Inf
        assert!(a.is_inf());
        assert!(!a.is_nan());
        assert!(a.is_negative());
    }
    {
        let a = FP32::from_f32(f32::from_bits(0xffc00000)); // -Nan.
        assert!(!a.is_inf());
        assert!(a.is_nan());
        assert!(a.is_negative());
    }

    {
        let a = FP64::from_f64(f64::from_bits((mask(32) << 32) as u64));
        assert!(!a.is_inf());
        assert!(a.is_nan());
    }
    {
        // Check that casting propagates inf/nan.
        let a = FP32::from_f32(f32::from_bits(0xff800000)); // -Inf
        let b: FP64 = a.cast();
        assert!(b.is_inf());
        assert!(!b.is_nan());
        assert!(b.is_negative());
    }
}

#[test]
fn test_cast_down_easy() {
    // Check that we can cast the numbers down, matching the hardware casting.
    for v in [0.3, 0.1, 14151241515., 14151215., 0.0000000001, 1000000000.] {
        let res = FP64::from_f64(v).as_f32();
        assert_eq!(FP64::from_f64(v).as_f64().to_bits(), v.to_bits());
        assert!(res == v as f32);
    }
}

#[test]
fn test_cast_down_complex() {
    // Try casting a bunch of difficult values such as inf, nan, denormals, etc.
    for v in utils::get_special_test_values() {
        let res = FP64::from_f64(v).as_f32();
        assert_eq!(FP64::from_f64(v).as_f64().to_bits(), v.to_bits());
        assert_eq!(v.is_nan(), res.is_nan());
        println!("{} != {}", res, v as f32);
        println!("{:32b} X!=\n{:32b} V", res.to_bits(), (v as f32).to_bits());

        assert!(v.is_nan() || res == v as f32);
    }
}
