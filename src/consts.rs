use crate::fp::ToFixedPoint;

pub const PROJECTION_PLANE_WIDTH: i32 = 320;
pub const TILE_SIZE: i32 = 64;
pub const FP_TILE_SIZE: i32 = TILE_SIZE << 16;

pub const WALL_HEIGHT_SCALE_FACTOR: i32 = 18000;
pub const WALL_HEIGHT_MIN: i32 = 8;
pub const WALL_HEIGHT_MAX: i32 = 640;
