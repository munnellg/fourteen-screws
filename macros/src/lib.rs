use shared::consts;
use shared::fp::{ ToFixedPoint };
use shared::radian;
use proc_macro2::TokenStream;
use quote::quote;

const TILE_SIZE: f64 = consts::TILE_SIZE as f64;

fn clamp(x: i32, min: i32, max: i32) -> i32 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

fn declare_trig_tables() -> TokenStream {
    const SIZE: usize    = (consts::ANGLE_360 + 1) as usize;

    let mut sin: [i32; SIZE] = [0; SIZE];
    let mut cos: [i32; SIZE] = [0; SIZE];
    let mut tan: [i32; SIZE] = [0; SIZE];
    let mut isin: [i32; SIZE] = [0; SIZE];
    let mut icos: [i32; SIZE] = [0; SIZE];
    let mut itan: [i32; SIZE] = [0; SIZE];
    
    for i in 0..SIZE {
        sin[i] = (radian!(i).sin()).to_fp();
        cos[i] = (radian!(i).cos()).to_fp();
        tan[i] = (radian!(i).tan()).to_fp();
        isin[i] = (1.0 / radian!(i).sin()).to_fp();
        icos[i] = (1.0 / radian!(i).cos()).to_fp();
        itan[i] = (1.0 / radian!(i).tan()).to_fp() ;
    }

    quote! {
		static SIN: [i32; #SIZE] = [ #(#sin),* ];
	    static COS: [i32; #SIZE] = [ #(#cos),* ];
	    static TAN: [i32; #SIZE] = [ #(#tan),* ];
	    static ISIN: [i32; #SIZE] = [ #(#isin),* ];
	    static ICOS: [i32; #SIZE] = [ #(#icos),* ];
	    static ITAN: [i32; #SIZE] = [ #(#itan),* ];
	}
}

fn declare_step_tables() -> TokenStream {
    const SIZE: usize    = (consts::ANGLE_360 + 1) as usize;

	let mut x_step: [i32; SIZE] = [0; SIZE];
    let mut y_step: [i32; SIZE] = [0; SIZE];

    for i in 0..SIZE {
        let mut step: f64;

        if radian!(i).tan() == 0.0 {
            step = f64::MAX
        } else {
            step = TILE_SIZE / radian!(i).tan();

            if i >= consts::ANGLE_90.try_into().unwrap() && i < consts::ANGLE_270.try_into().unwrap() {
                if step > 0.0 {
                  step = -step;
                }
            } else {
                if step < 0.0 {
                  step = -step;
                }
            }
        }

        x_step[i] = step.to_fp();
    }

    for i in 0..SIZE {
        let mut step = TILE_SIZE * radian!(i).tan();

        if i >= consts::ANGLE_0.try_into().unwrap() && i < consts::ANGLE_180.try_into().unwrap() {
            if step < 0.0 {
              step = -step;
            }
        } else {
            if step > 0.0 {
              step = -step;
            }
        }

        y_step[i] = step.to_fp();
    }

    quote! {
    	static X_STEP: [i32; #SIZE] = [ #(#x_step),* ];
    	static Y_STEP: [i32; #SIZE] = [ #(#y_step),* ];
    }
}

fn declare_fisheye_table() -> TokenStream {
    const SIZE: usize = consts::PROJECTION_PLANE_WIDTH as usize;

    let mut fisheye: [i32; SIZE] = [0; SIZE];

    for i in 0..SIZE {
        fisheye[i] = (1.0 / radian!(i as i32 - consts::ANGLE_30 as i32).cos()).to_fp();
    }

    quote! {
        static FISHEYE: [i32; #SIZE] = [ #(#fisheye),* ];
    }
}

fn declare_wall_height_table() -> TokenStream {
    const SIZE: usize = (consts::MAX_RAY_LENGTH + 1) as usize;

    let mut wall_height: [i32; SIZE] = [0; SIZE];

    for i in 0..=consts::MAX_RAY_LENGTH {
        wall_height[i as usize] = clamp(consts::WALL_HEIGHT_SCALE_FACTOR / i.max(1), consts::WALL_HEIGHT_MIN, consts::WALL_HEIGHT_MAX);
    }

    quote! {
        static WALL_HEIGHT: [i32; #SIZE] = [ #(#wall_height),* ];
    }
}

fn declare_floor_ceiling_tables() -> TokenStream {
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
    quote! {

    }
}

#[proc_macro]
pub fn insert_lookup_tables(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let trig_tables          = declare_trig_tables();
	let step_tables          = declare_step_tables();
    let fisheye_table        = declare_fisheye_table();
    let wall_height_table    = declare_wall_height_table();
    let floor_ceiling_tables = declare_floor_ceiling_tables();

	proc_macro::TokenStream::from(quote! {
		#trig_tables
		#step_tables
        #fisheye_table
        #wall_height_table
        #floor_ceiling_tables
	})
}