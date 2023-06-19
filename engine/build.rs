// primarily used for writing the file
use std::f64::consts::PI;
use std::{env, fs, path::Path};

const PROJECTION_PLANE_WIDTH: usize  = 320;
const PROJECTION_PLANE_HEIGHT: usize = 200;
const DISTANCE_TO_PROJECTION_PLANE: usize = 277;

const TILE_SIZE: f64 = 64.0;

const ANGLE_0:   usize = 0;
const ANGLE_60:  usize = PROJECTION_PLANE_WIDTH;
const ANGLE_30:  usize = ANGLE_60 / 2;
const ANGLE_90:  usize = ANGLE_30 * 3;
const ANGLE_180: usize = ANGLE_60 * 3;
const ANGLE_270: usize = ANGLE_90 * 3;
const ANGLE_360: usize = ANGLE_60 * 6;

const MAX_RAY_LENGTH: usize = 2048;
const WALL_HEIGHT_SCALE_FACTOR: usize = 18000; 
const WALL_HEIGHT_MAX: i32            = 640;
const WALL_HEIGHT_MIN: i32            = 8;

const PLAYER_HEIGHT: usize             = 32;
const PROJECTION_PLANE_CENTRE_Y: usize = PROJECTION_PLANE_HEIGHT >> 1;


fn clamp(x: i32, min: i32, max: i32) -> i32 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

fn radian(angle: usize) -> f64 {
    angle as f64 * PI / ANGLE_180 as f64
}

fn iradian(angle: i32) -> f64 {
    angle as f64 * PI / ANGLE_180 as f64
}

fn float_to_fix(x: f64) -> i32 {    
    (x * 65536.0) as i32
}

fn stringify(name: &str, arr: &[i32], size: usize) -> String {
    let mut array_string = String::from("static ");
    array_string.push_str(name);
    array_string.push_str(":[i32; ");
    array_string.push_str(size.to_string().as_str());
    array_string.push_str("] = [\r\n");
    for a in arr {
        // a little bit of formatting is happening as well
        array_string.push_str("\u{20}\u{20}\u{20}\u{20}");
        array_string.push_str(a.to_string().as_str());
        array_string.push_str(",\r\n");
    }
    array_string.push_str("];\r\n");
    array_string
}

fn main() {
    const SIZE: usize = ANGLE_360 + 1;

    let mut sin: [i32; SIZE] = [0; SIZE];
    let mut cos: [i32; SIZE] = [0; SIZE];
    let mut tan: [i32; SIZE] = [0; SIZE];
    let mut isin: [i32; SIZE] = [0; SIZE];
    let mut icos: [i32; SIZE] = [0; SIZE];
    let mut itan: [i32; SIZE] = [0; SIZE];
    
    for i in 0..=1920 {
        sin[i] = float_to_fix(radian(i).sin());
        cos[i] = float_to_fix(radian(i).cos());
        tan[i] = float_to_fix(radian(i).tan());
        isin[i] = float_to_fix(1.0 / radian(i).sin());
        icos[i] = float_to_fix(1.0 / radian(i).cos());
        itan[i] = float_to_fix(1.0 / radian(i).tan());        
    }



    let mut output = stringify("SIN", &sin, SIZE);
    output.push_str(stringify("COS", &cos, SIZE).as_str());
    output.push_str(stringify("TAN", &tan, SIZE).as_str());
    output.push_str(stringify("ISIN", &isin, SIZE).as_str());
    output.push_str(stringify("ICOS", &icos, SIZE).as_str());
    output.push_str(stringify("ITAN", &itan, SIZE).as_str());

    let mut x_step: [i32; SIZE] = [0; SIZE];
    let mut y_step: [i32; SIZE] = [0; SIZE];

    for i in 0..=1920 {
        let mut step: f64;

        if radian(i).tan() == 0.0 {
            step = f64::MAX
        } else {
            step = TILE_SIZE / radian(i).tan();

            if i >= ANGLE_90 && i < ANGLE_270 {
                if step > 0.0 {
                  step = -step;
                }
            } else {
                if step < 0.0 {
                  step = -step;
                }
            }
        }

        x_step[i] = float_to_fix(step);
    }

    for i in 0..=1920 {
        let mut step = TILE_SIZE * radian(i).tan();

        if i >= ANGLE_0 && i < ANGLE_180 {
            if step < 0.0 {
              step = -step;
            }
        } else {
            if step > 0.0 {
              step = -step;
            }
        }

        y_step[i] = (step * 65536.0) as i32; //float_to_fix(step);
    }

    output.push_str(stringify("X_STEP", &x_step, SIZE).as_str());
    output.push_str(stringify("Y_STEP", &y_step, SIZE).as_str());

    let mut fisheye: [i32; PROJECTION_PLANE_WIDTH] = [0; PROJECTION_PLANE_WIDTH];

    for i in 0..PROJECTION_PLANE_WIDTH {
        fisheye[i] = float_to_fix(1.0 / iradian(i as i32 - ANGLE_30 as i32).cos());    
    }

    output.push_str(stringify("FISHEYE", &fisheye, PROJECTION_PLANE_WIDTH).as_str());
    
    let mut wall_height: [i32; MAX_RAY_LENGTH + 1] = [0; MAX_RAY_LENGTH + 1];
    for i in 0..=MAX_RAY_LENGTH {
        wall_height[i] = clamp((WALL_HEIGHT_SCALE_FACTOR / i.max(1)) as i32, WALL_HEIGHT_MIN, WALL_HEIGHT_MAX);
    }

    output.push_str(stringify("WALL_HEIGHT", &wall_height, MAX_RAY_LENGTH + 1).as_str());

    // let mut FLOOR_TEXTURE_Y_RAYS: [i32; PROJECTION_PLANE_WIDTH * PROJECTION_PLANE_CENTRE_Y] = [0; PROJECTION_PLANE_WIDTH * PROJECTION_PLANE_CENTRE_Y];
    // let mut FLOOR_TEXTURE_X_RAYS: [i32; PROJECTION_PLANE_WIDTH * PROJECTION_PLANE_CENTRE_Y] = [0; PROJECTION_PLANE_WIDTH * PROJECTION_PLANE_CENTRE_Y];

    // for y in (PROJECTION_PLANE_CENTRE_Y + 1)..PROJECTION_PLANE_HEIGHT {
    //     let ratio: f64 = PLAYER_HEIGHT as f64 / (y - PROJECTION_PLANE_CENTRE_Y) as f64;
    //     for sweep in 0..PROJECTION_PLANE_WIDTH {
    //         let distance = DISTANCE_TO_PROJECTION_PLANE as f64 * ratio * fisheye[sweep];
    //     }
    // }

// var diagonalDistance=Math.floor((this.fPlayerDistanceToTheProjectionPlane * ratio) * (this.fFishTable[castColumn]));

// var yEnd = Math.floor(diagonalDistance * this.fSinTable[castArc]);
// var xEnd = Math.floor(diagonalDistance * this.fCosTable[castArc]);

    // write the string to a file. OUT_DIR environment variable is defined by cargo
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("lookup.rs");
    fs::write(&dest_path, output).unwrap();
}