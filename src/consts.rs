use crate::fp::ToFixedPoint;

pub const PROJECTION_PLANE_HEIGHT: i32 = 200;
pub const PROJECTION_PLANE_WIDTH: i32 = 320;
pub const TILE_SIZE: i32 = 64;
pub const FP_TILE_SIZE: i32 = TILE_SIZE << 16;

pub const WALL_HEIGHT_SCALE_FACTOR: i32 = 18000;
pub const WALL_HEIGHT_MIN: i32 = 8;
pub const WALL_HEIGHT_MAX: i32 = 640;

pub const MAX_RAY_LENGTH: i32 = 2048;
pub const FP_MAX_RAY_LENGTH: i32 = MAX_RAY_LENGTH << 16;