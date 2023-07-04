use crate::consts::display::PROJECTION_PLANE_WIDTH;

pub const ANGLE_60:  i32 = PROJECTION_PLANE_WIDTH;

pub const ANGLE_0:   i32 = 0;
pub const ANGLE_5:   i32 = ANGLE_60 / 12;
pub const ANGLE_10:  i32 = ANGLE_60 / 6;
pub const ANGLE_30:  i32 = ANGLE_60 / 2;
pub const ANGLE_90:  i32 = ANGLE_30 * 3;
pub const ANGLE_180: i32 = ANGLE_60 * 3;
pub const ANGLE_270: i32 = ANGLE_90 * 3;
pub const ANGLE_360: i32 = ANGLE_60 * 6;