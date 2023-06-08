use core::f64::consts::PI;
use crate::consts::{ PROJECTION_PLANE_WIDTH, TILE_SIZE };

include!(concat!(env!("OUT_DIR"), "/lookup.rs"));

pub const ANGLE_0:   i32 = 0;
pub const ANGLE_60:  i32 = PROJECTION_PLANE_WIDTH;
pub const ANGLE_30:  i32 = ANGLE_60 / 2;
pub const ANGLE_90:  i32 = ANGLE_30 * 3;
pub const ANGLE_180: i32 = ANGLE_60 * 3;
pub const ANGLE_270: i32 = ANGLE_90 * 3;
pub const ANGLE_360: i32 = ANGLE_60 * 6;

fn clamp(x: i32, min: i32, max: i32) -> i32 {
	if x < min {
		min
	} else if x > max {
		max
	} else {
		x
	}
}

pub fn radian(angle: i32) -> f64 {
	angle as f64 * PI / ANGLE_180 as f64
}

pub fn cos(degrees: i32) -> f64 {
	COS[degrees as usize] as f64 / 65536.0
}

pub fn sin(degrees: i32) -> f64 {
	SIN[degrees as usize] as f64 / 65536.0
}

pub fn tan(degrees: i32) -> f64 {
	TAN[degrees as usize] as f64 / 65536.0
}

pub fn icos(degrees: i32) -> f64 {
	ICOS[degrees as usize] as f64 / 65536.0
}

pub fn isin(degrees: i32) -> f64 {
	ISIN[degrees as usize] as f64 / 65536.0
}

pub fn itan(degrees: i32) -> f64 {
	ITAN[degrees as usize] as f64 / 65536.0
}

pub fn xstep(degrees: i32) -> f64 {
	X_STEP[degrees as usize] as f64 / 65536.0
}

pub fn ystep(degrees: i32) -> f64 {
	Y_STEP[degrees as usize] as f64 / 65536.0
}

pub fn fisheye_correction(degrees: i32) -> f64 {
	FISHEYE[degrees as usize] as f64 / 65536.0
}

pub fn wall_height(dist: i32) -> i32 {
	WALL_HEIGHT[dist.min(2048) as usize]
}

#[cfg(test)]
mod tests {
	use float_cmp;
	use super::*;

	#[test]
	fn test_cos_values() {
		let tests = [
			("ANGLE_0",   ANGLE_0,    1.0),
			("ANGLE_30",  ANGLE_30,   0.8660254),
			("ANGLE_60",  ANGLE_60,   0.5),
			("ANGLE_90",  ANGLE_90,   0.0),
			("ANGLE_180", ANGLE_180, -1.0),
			("ANGLE_270", ANGLE_270,  0.0),
			("ANGLE_360", ANGLE_360,  1.0),
		];

		for (label, angle, result) in tests {
			println!("cos({label})");
			float_cmp::assert_approx_eq!(f64, cos(angle), result, epsilon = 0.00000003, ulps = 2);
		}
	}

	#[test]
	fn test_sin_values() {
		let tests = [
			("ANGLE_0",   ANGLE_0,    0.0),
			("ANGLE_30",  ANGLE_30,   0.5),
			("ANGLE_60",  ANGLE_60,   0.8660254),
			("ANGLE_90",  ANGLE_90,   1.0),
			("ANGLE_180", ANGLE_180,  0.0),
			("ANGLE_270", ANGLE_270, -1.0),
			("ANGLE_360", ANGLE_360,  0.0),
		];

		for (label, angle, result) in tests {
			println!("sin({label})");
			float_cmp::assert_approx_eq!(f64, sin(angle), result, epsilon = 0.00000003, ulps = 2);
		}
	}

	#[test]
	fn test_tan_values() {
		let tests = [
			("ANGLE_0",   ANGLE_0,    0.0),
			("ANGLE_30",  ANGLE_30,   0.577350269),
			("ANGLE_60",  ANGLE_60,   1.732050808),
			("ANGLE_90",  ANGLE_90,   f64::INFINITY),
			("ANGLE_180", ANGLE_180,  0.0),
			("ANGLE_270", ANGLE_270,  f64::NEG_INFINITY),
			("ANGLE_360", ANGLE_360,  0.0),
		];

		for (label, angle, result) in tests {
			println!("tan({label})");
			float_cmp::assert_approx_eq!(f64, tan(angle), result, epsilon = 0.00000003, ulps = 2);
		}
	}

	#[test]
	fn test_icos_values() {
		let tests = [
			("ANGLE_0",   ANGLE_0,    1.0),
			("ANGLE_30",  ANGLE_30,   1.154700538),
			("ANGLE_60",  ANGLE_60,   2.0),
			("ANGLE_90",  ANGLE_90,   f64::INFINITY),
			("ANGLE_180", ANGLE_180, -1.0),
			("ANGLE_270", ANGLE_270,  f64::INFINITY),
			("ANGLE_360", ANGLE_360,  1.0),
		];

		for (label, angle, result) in tests {
			println!("icos({label})");
			float_cmp::assert_approx_eq!(f64, icos(angle), result, epsilon = 0.00000003, ulps = 2);
		}
	}

	#[test]
	fn test_isin_values() {
		let tests = [
			("ANGLE_0",   ANGLE_0,    f64::INFINITY),
			("ANGLE_30",  ANGLE_30,   2.0),
			("ANGLE_60",  ANGLE_60,   1.154700538),
			("ANGLE_90",  ANGLE_90,   1.0),
			("ANGLE_180", ANGLE_180,  f64::INFINITY),
			("ANGLE_270", ANGLE_270, -1.0),
			("ANGLE_360", ANGLE_360,  f64::INFINITY),
		];

		for (label, angle, result) in tests {
			println!("isin({label})");
			float_cmp::assert_approx_eq!(f64, isin(angle), result, epsilon = 0.00000003, ulps = 2);
		}
	}

	#[test]
	fn test_itan_values() {
		let tests = [
			("ANGLE_0",   ANGLE_0,    f64::INFINITY),
			("ANGLE_30",  ANGLE_30,   1.732050808),
			("ANGLE_60",  ANGLE_60,   0.577350269),
			("ANGLE_90",  ANGLE_90,   0.0),
			("ANGLE_180", ANGLE_180,  f64::NEG_INFINITY),
			("ANGLE_270", ANGLE_270,  0.0),
			("ANGLE_360", ANGLE_360,  f64::INFINITY),
		];

		for (label, angle, result) in tests {
			println!("itan({label})");
			float_cmp::assert_approx_eq!(f64, itan(angle), result, epsilon = 0.00000003, ulps = 2);
		}
	}
}
