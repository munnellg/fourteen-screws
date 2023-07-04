pub mod consts;
pub mod fp;

#[macro_export]
macro_rules! radian {
    ($angle:expr) => {
        $angle as f64 * std::f64::consts::PI / shared::consts::ANGLE_180 as f64
    }
}