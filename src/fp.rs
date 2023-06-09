const FP_SHIFT: i32 = 16;
const FP_MULT: f64  = 65536.0;
const FP_HALF: f64  = 32768.0;

pub trait ToFixedPoint {
    fn to_fp(&self) -> i32;
}

pub trait FromFixedPoint {
    fn to_f64(&self) -> f64;
    fn to_i32(&self) -> i32;
}

pub trait FixedPointMath {
	fn fp_add(&self, b: i32) -> i32;
	fn fp_sub(&self, b: i32) -> i32;
	fn fp_mul(&self, b: i32) -> i32;
	fn fp_div(&self, b: i32) -> i32;
}

impl ToFixedPoint for f64 {
    fn to_fp(&self) -> i32 {
        (*self * FP_MULT) as i32
    }
}

impl ToFixedPoint for i32 {
    fn to_fp(&self) -> i32 {
        *self << FP_SHIFT
    }
}

impl FromFixedPoint for i32 {
	fn to_f64(&self) -> f64 {
		*self as f64 / FP_MULT
	}

    fn to_i32(&self) -> i32 {
    	*self >> FP_SHIFT
    }
}

pub const fn add(a: i32, b: i32) -> i32 {
	a + b
}

pub const fn sub(a: i32, b: i32) -> i32 {
	a - b
}

pub const fn mul(a: i32, b: i32) -> i32 {
	((a as i64 * b as i64) >> FP_SHIFT) as i32
}

pub const fn div(a: i32, b: i32) -> i32 {
	(((a as i64)  << FP_SHIFT) / b as i64) as i32
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn f64_add() {
		let test_pairs = [
			(0.5, 0.5),
			(-0.754, 0.123)
		];

		for (a, b) in test_pairs {
			let fp_sum = add(a.to_fp(), b.to_fp());
			float_cmp::assert_approx_eq!(f64, fp_sum.to_f64(), a + b, epsilon = 0.003, ulps = 2)
		}
	}

	#[test]
	fn f64_sub() {
		let test_pairs = [
			(0.5, 0.5),
			(-0.754, 0.123)
		];

		for (a, b) in test_pairs {
			let fp_diff = sub(a.to_fp(), b.to_fp());
			float_cmp::assert_approx_eq!(f64, fp_diff.to_f64(), a - b, epsilon = 0.003, ulps = 2)
		}
	}

	#[test]
	fn f64_mul() {
		let test_pairs = [
			(0.5, 0.5),
			(-0.754, 0.123)
		];

		for (a, b) in test_pairs {
			let fp_prod = mul(a.to_fp(), b.to_fp());
			float_cmp::assert_approx_eq!(f64, fp_prod.to_f64(), a * b, epsilon = 0.003, ulps = 2)
		}
	}

	#[test]
	fn f64_div() {
		let test_pairs = [
			(0.5, 0.5),
			(-0.754, 0.123)
		];

		for (a, b) in test_pairs {
			let fp_quot = div(a.to_fp(), b.to_fp());
			float_cmp::assert_approx_eq!(f64, fp_quot.to_f64(), a / b, epsilon = 0.003, ulps = 2)
		}
	}
}