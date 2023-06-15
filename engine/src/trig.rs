use crate::consts::{ PROJECTION_PLANE_WIDTH, MAX_RAY_LENGTH };
use core::f64::consts::PI;

include!(concat!(env!("OUT_DIR"), "/lookup.rs"));

pub const ANGLE_60:  i32 = PROJECTION_PLANE_WIDTH;

pub const ANGLE_0:   i32 = 0;
pub const ANGLE_5:   i32 = ANGLE_60 / 12;
pub const ANGLE_10:  i32 = ANGLE_60 / 6;
pub const ANGLE_30:  i32 = ANGLE_60 / 2;
pub const ANGLE_90:  i32 = ANGLE_30 * 3;
pub const ANGLE_180: i32 = ANGLE_60 * 3;
pub const ANGLE_270: i32 = ANGLE_90 * 3;
pub const ANGLE_360: i32 = ANGLE_60 * 6;

pub fn radian(angle: i32) -> f64 {
	angle as f64 * PI / ANGLE_180 as f64
}

pub fn cos(degrees: i32) -> i32 {
	COS[degrees as usize]
}

pub fn sin(degrees: i32) -> i32 {
	SIN[degrees as usize]
}

pub fn tan(degrees: i32) -> i32 {
	TAN[degrees as usize]
}

pub fn icos(degrees: i32) -> i32 {
	ICOS[degrees as usize]
}

pub fn isin(degrees: i32) -> i32 {
	ISIN[degrees as usize]
}

pub fn itan(degrees: i32) -> i32 {
	ITAN[degrees as usize]
}

pub fn xstep(degrees: i32) -> i32 {
	X_STEP[degrees as usize]
}

pub fn ystep(degrees: i32) -> i32 {
	Y_STEP[degrees as usize]
}

pub fn fisheye_correction(degrees: i32) -> i32 {
	FISHEYE[degrees as usize]
}

pub fn wall_height(dist: i32) -> i32 {
	WALL_HEIGHT[dist.min(MAX_RAY_LENGTH) as usize]
}

// #[cfg(test)]
// mod tests {
// 	use float_cmp;
// 	use super::*;

// 	#[test]
// 	fn test_cos_values() {
// 		let tests = [
// 			("ANGLE_0",   ANGLE_0,    1.0),
// 			("ANGLE_30",  ANGLE_30,   0.8660254),
// 			("ANGLE_60",  ANGLE_60,   0.5),
// 			("ANGLE_90",  ANGLE_90,   0.0),
// 			("ANGLE_180", ANGLE_180, -1.0),
// 			("ANGLE_270", ANGLE_270,  0.0),
// 			("ANGLE_360", ANGLE_360,  1.0),
// 		];

// 		for (label, angle, result) in tests {
// 			println!("cos({label})");
// 			float_cmp::assert_approx_eq!(f64, cos(angle), result, epsilon = 0.00000003, ulps = 2);
// 		}
// 	}

// 	#[test]
// 	fn test_sin_values() {
// 		let tests = [
// 			("ANGLE_0",   ANGLE_0,    0.0),
// 			("ANGLE_30",  ANGLE_30,   0.5),
// 			("ANGLE_60",  ANGLE_60,   0.8660254),
// 			("ANGLE_90",  ANGLE_90,   1.0),
// 			("ANGLE_180", ANGLE_180,  0.0),
// 			("ANGLE_270", ANGLE_270, -1.0),
// 			("ANGLE_360", ANGLE_360,  0.0),
// 		];

// 		for (label, angle, result) in tests {
// 			println!("sin({label})");
// 			float_cmp::assert_approx_eq!(f64, sin(angle), result, epsilon = 0.00000003, ulps = 2);
// 		}
// 	}

// 	#[test]
// 	fn test_tan_values() {
// 		let tests = [
// 			("ANGLE_0",   ANGLE_0,    0.0),
// 			("ANGLE_30",  ANGLE_30,   0.577350269),
// 			("ANGLE_60",  ANGLE_60,   1.732050808),
// 			("ANGLE_90",  ANGLE_90,   f64::INFINITY),
// 			("ANGLE_180", ANGLE_180,  0.0),
// 			("ANGLE_270", ANGLE_270,  f64::NEG_INFINITY),
// 			("ANGLE_360", ANGLE_360,  0.0),
// 		];

// 		for (label, angle, result) in tests {
// 			println!("tan({label})");
// 			float_cmp::assert_approx_eq!(f64, tan(angle), result, epsilon = 0.00000003, ulps = 2);
// 		}
// 	}

// 	#[test]
// 	fn test_icos_values() {
// 		let tests = [
// 			("ANGLE_0",   ANGLE_0,    1.0),
// 			("ANGLE_30",  ANGLE_30,   1.154700538),
// 			("ANGLE_60",  ANGLE_60,   2.0),
// 			("ANGLE_90",  ANGLE_90,   f64::INFINITY),
// 			("ANGLE_180", ANGLE_180, -1.0),
// 			("ANGLE_270", ANGLE_270,  f64::INFINITY),
// 			("ANGLE_360", ANGLE_360,  1.0),
// 		];

// 		for (label, angle, result) in tests {
// 			println!("icos({label})");
// 			float_cmp::assert_approx_eq!(f64, icos(angle), result, epsilon = 0.00000003, ulps = 2);
// 		}
// 	}

// 	#[test]
// 	fn test_isin_values() {
// 		let tests = [
// 			("ANGLE_0",   ANGLE_0,    f64::INFINITY),
// 			("ANGLE_30",  ANGLE_30,   2.0),
// 			("ANGLE_60",  ANGLE_60,   1.154700538),
// 			("ANGLE_90",  ANGLE_90,   1.0),
// 			("ANGLE_180", ANGLE_180,  f64::INFINITY),
// 			("ANGLE_270", ANGLE_270, -1.0),
// 			("ANGLE_360", ANGLE_360,  f64::INFINITY),
// 		];

// 		for (label, angle, result) in tests {
// 			println!("isin({label})");
// 			float_cmp::assert_approx_eq!(f64, isin(angle), result, epsilon = 0.00000003, ulps = 2);
// 		}
// 	}

// 	#[test]
// 	fn test_itan_values() {
// 		let tests = [
// 			("ANGLE_0",   ANGLE_0,    f64::INFINITY),
// 			("ANGLE_30",  ANGLE_30,   1.732050808),
// 			("ANGLE_60",  ANGLE_60,   0.577350269),
// 			("ANGLE_90",  ANGLE_90,   0.0),
// 			("ANGLE_180", ANGLE_180,  f64::NEG_INFINITY),
// 			("ANGLE_270", ANGLE_270,  0.0),
// 			("ANGLE_360", ANGLE_360,  f64::INFINITY),
// 		];

// 		for (label, angle, result) in tests {
// 			println!("itan({label})");
// 			float_cmp::assert_approx_eq!(f64, itan(angle), result, epsilon = 0.00000003, ulps = 2);
// 		}
// 	}
// }
