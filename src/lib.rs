extern crate web_sys;
use wasm_bindgen::prelude::*;

mod utils;
mod consts;
mod trig;
mod raycast;

macro_rules! log {
	( $( $t:tt )* ) => {
		web_sys::console::log_1(&format!( $( $t )* ).into());
	}
}

fn draw_wall_column(buf: &mut[u8], column: i32, dist: f64) {
	// get wall texture, draw into column
	let wall_height: i32 = consts::WALL_HEIGHT_SCALE_FACTOR / dist.max(1.0) as i32;

	let y_min = std::cmp::max(0, (200 - wall_height) / 2);
	let y_max = std::cmp::min(200 - 1, y_min + wall_height);

	for y in y_min..=y_max {
		let idx: usize = 4 * (column + y * consts::PROJECTION_PLANE_WIDTH) as usize;
		buf[idx + 0] = 0xFF;
		buf[idx + 1] = 0x00;
		buf[idx + 2] = 0x00;
		buf[idx + 3] = 0xFF; // alpha channel
	}
}

#[wasm_bindgen]
pub fn render(buf: &mut[u8], theta: i32) {
	let world = raycast::World::new(7, 7, "WHHHHHWVOOOOOVVOOOOOVVOOOOOVVOOOOOVVOOOOOVWHHHHHW").unwrap();

	// put the player in the middle of the test map
	let player_x = 5 * consts::TILE_SIZE / 2;
	let player_y = 5 * consts::TILE_SIZE / 2;

	// draw a grey background that will represent the ceiling and floor
	for x in &mut *buf { *x = 128; }

	// theta is the direction player is facing
	// need to start out sweep 30 degrees to the left
	let mut angle = if theta < trig::ANGLE_30 {
		theta - trig::ANGLE_30 + trig::ANGLE_360
	} else {
		theta - trig::ANGLE_30
	};

	// sweep of the rays will be through 60 degrees
	for sweep in 0..trig::ANGLE_60 {
		log!("{sweep}");
		let hdist = world.find_vertical_intersect(player_x, player_y, angle);		
		let vdist = world.find_horizontal_intersect(player_x, player_y, angle);
		let dist = hdist.min(vdist) / trig::fisheye_correction(sweep);

		draw_wall_column(buf, sweep, dist);

		angle += 1;
		if angle >= trig::ANGLE_360 {
			angle -= trig::ANGLE_360;
		}
	}
}
