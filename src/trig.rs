// pub const DISPLAY_HEIGHT: i32 = 200;
use core::f64::consts::PI;
use crate::consts::{ PROJECTION_PLANE_WIDTH, TILE_SIZE };

pub const ANGLE_0:   i32 = 0;
pub const ANGLE_60:  i32 = PROJECTION_PLANE_WIDTH;
pub const ANGLE_30:  i32 = ANGLE_60 / 2;
pub const ANGLE_90:  i32 = ANGLE_30 * 3;
pub const ANGLE_180: i32 = ANGLE_60 * 3;
pub const ANGLE_270: i32 = ANGLE_90 * 3;
pub const ANGLE_360: i32 = ANGLE_60 * 6;

// pub const ANGLE_15: i32  = ANGLE_60 / 4;
// pub const ANGLE_90: i32  = ANGLE_30 * 3;

// pub const SIN: [f64; (ANGLE_360 + 1) as usize] = gen_sin_table();

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
	radian(degrees).cos()
}

pub fn sin(degrees: i32) -> f64 {
	radian(degrees).sin()

}

pub fn tan(degrees: i32) -> f64 {
	radian(degrees).tan()

}

pub fn icos(degrees: i32) -> f64 {
	let x = cos(degrees);
	if x == 0.0 { f64::MAX } else { 1.0 / x }

}

pub fn isin(degrees: i32) -> f64 {
	let x = sin(degrees);
	if x == 0.0 { f64::MAX } else { 1.0 / x }

}

pub fn itan(degrees: i32) -> f64 {
	let x = tan(degrees);
	if x == 0.0 { f64::MAX } else { 1.0 / x }

}

pub fn xstep(degrees: i32) -> f64 {
	if tan(degrees) == 0.0 {
		return f64::MAX
	}

	let mut step = TILE_SIZE as f64 * itan(degrees);

	if degrees >= ANGLE_90 && degrees < ANGLE_270 {
		if step < 0.0 {
		  return -step;
		}
	} else {
		if step > 0.0 {
		  return -step;
		}
	}

	step
}

pub fn ystep(degrees: i32) -> f64 {

	let mut step = TILE_SIZE as f64 * tan(degrees);

	if degrees >= ANGLE_0 && degrees < ANGLE_180 {
		if step < 0.0 {
		  return -step;
		}
	} else {
		if step > 0.0 {
		  return -step;
		}
	}

	step
}

pub fn fisheye_correction(degrees: i32) -> f64 {
	1.0 / cos(degrees - ANGLE_30)
}

pub fn wall_height(dist: i32) -> i32 {
	const WALL_HEIGHT_SCALE_FACTOR: i32 = 18000; 
	const WALL_HEIGHT_MAX: i32          = 640;
	const WALL_HEIGHT_MIN: i32          = 8;
	clamp(WALL_HEIGHT_SCALE_FACTOR / dist, WALL_HEIGHT_MIN, WALL_HEIGHT_MAX)
}