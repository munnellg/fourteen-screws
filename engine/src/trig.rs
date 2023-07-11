use macros::insert_lookup_tables;

pub use shared::consts::trig::*;
pub use shared::radian;

insert_lookup_tables!();

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

pub fn x_step(degrees: i32) -> i32 {
	X_STEP[degrees as usize]
}

pub fn y_step(degrees: i32) -> i32 {
	Y_STEP[degrees as usize]
}

pub fn fisheye_correction(degrees: i32) -> i32 {
	FISHEYE[degrees as usize]
}

pub fn wall_height(distance: i32) -> i32 {
	WALL_HEIGHT[distance.min(shared::consts::MAX_RAY_LENGTH) as usize]
}

pub fn wall_texture_index(height: i32) -> &'static [usize] {
	let height      = height.clamp(shared::consts::render::WALL_HEIGHT_MIN, shared::consts::render::WALL_HEIGHT_MAX);
	let true_i      = height - shared::consts::render::WALL_HEIGHT_MIN;
	let head: usize = (true_i * shared::consts::display::PROJECTION_PLANE_HEIGHT) as usize;
	let tail: usize = head + shared::consts::display::PROJECTION_PLANE_HEIGHT as usize;
	&WALL_TEXTURE_INDEX[head..tail]
}
