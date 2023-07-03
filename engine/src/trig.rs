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
