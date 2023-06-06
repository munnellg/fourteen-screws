extern crate web_sys;
use wasm_bindgen::prelude::*;

mod utils;
mod consts;
mod trig;

const MAP_WIDTH: i32  = 7;
const MAP_HEIGHT: i32 = 7;

static H_WALLS: &'static[i32] = &[
	1, 1, 1, 1, 1, 1, 1,
	0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0,
	1, 1, 1, 1, 1, 1, 1,
];

static V_WALLS: &'static[i32] = &[
	1, 0, 0, 0, 0, 0, 1,
	1, 0, 0, 0, 0, 0, 1,
	1, 0, 0, 0, 0, 0, 1,
	1, 0, 0, 0, 0, 0, 1,
	1, 0, 0, 0, 0, 0, 1,
	1, 0, 0, 0, 0, 0, 1,
	1, 0, 0, 0, 0, 0, 1,
];

macro_rules! log {
	( $( $t:tt )* ) => {
		web_sys::console::log_1(&format!( $( $t )* ).into());
	}
}

fn is_within_bounds(x: f64, y: f64) -> bool {
	let x = x as i32 / consts::TILE_SIZE;
	let y = y as i32 / consts::TILE_SIZE;
	x >= 0 && x < MAP_WIDTH && y >= 0 && y < MAP_HEIGHT
}

fn is_h_wall(x:f64, y:f64) -> bool {
	let x = x as i32 / consts::TILE_SIZE;
	let y = y as i32 / consts::TILE_SIZE;
	H_WALLS[(x + y  * MAP_WIDTH) as usize] > 0
}

fn is_v_wall(x:f64, y:f64) -> bool {
	let x = x as i32 / consts::TILE_SIZE;
	let y = y as i32 / consts::TILE_SIZE;
	V_WALLS[(x + y  * MAP_WIDTH) as usize] > 0
}

fn find_vertical_intersect(player_x: i32, player_y: i32, angle: i32) -> f64 {
	// determine if looking up or down and find horizontal intersection
	let hi: i32 = if angle > trig::ANGLE_0 && angle < trig::ANGLE_180 { // looking down
		(player_y / consts::TILE_SIZE) * consts::TILE_SIZE + consts::TILE_SIZE
	} else {                     // looking up
		(player_y / consts::TILE_SIZE) * consts::TILE_SIZE
	};

	if angle == trig::ANGLE_0 || angle == trig::ANGLE_180 {
		return f64::MAX;
	}

	let step_y = consts::F_TILE_SIZE;
	let step_x = trig::xstep(angle);

	let mut x: f64 = (player_x + (player_y - hi)) as f64 * trig::itan(angle);
	let mut y: f64 = hi as f64;

	// Cast x axis intersect rays, build up xSlice
	while is_within_bounds(x, y) {
		if is_v_wall(x, y) {
			break;
		}

		x += step_x;
		y += step_y;
	}	
	
	((player_y as f64 - y) * trig::isin(angle)).abs()
}

fn find_horizontal_intersect(player_x: i32, player_y: i32, angle: i32) -> f64 {
	// determine if looking left or right and find vertical intersection
	let vi: i32 = if angle <= trig::ANGLE_90 || angle > trig::ANGLE_270 { // looking right
		(player_x / consts::TILE_SIZE) * consts::TILE_SIZE +  consts::TILE_SIZE
	} else {
		(player_x / consts::TILE_SIZE) * consts::TILE_SIZE
	};

	let step_x = consts::F_TILE_SIZE;
	let step_y = trig::ystep(angle);

	let mut x: f64 = vi as f64;
	let mut y: f64 = (player_y + (player_x - vi)) as f64 * trig::tan(angle);

	// Cast y axis intersect rays, build up ySlice
	while is_within_bounds(x, y) {
		if is_h_wall(x, y) {
			break;
		}

		x += step_x;
		y += step_y;
	}

	((player_y as f64 - y) * trig::isin(angle)).abs()
}

fn draw_wall_column(buf: &mut[u8], column: i32, dist: f64) {
	// get wall texture, draw into column
	let wall_height: i32 = consts::WALL_HEIGHT_SCALE_FACTOR / dist as i32;

	let y_min = std::cmp::max(0, (200 - wall_height) / 2);
	let y_max = std::cmp::min(200 - 1, y_min + wall_height);

	for y in y_min..=y_max {
		let idx: usize = 4 * (column + y * consts::PROJECTION_PLANE_WIDTH) as usize;
		buf[idx] = 0xFF;
		buf[idx + 3] = 0xFF; // alpha channel
	}
}

#[wasm_bindgen]
pub fn render(buf: &mut[u8]) {
	utils::set_panic_hook();

	// put the player in the middle of the test map
	let player_x = 5 * consts::TILE_SIZE / 2;
	let player_y = 5 * consts::TILE_SIZE / 2;
	let theta    = 0;

	// theta is the direction player is facing
	// need to start out sweep 30 degrees to the left
	let mut angle = if theta < trig::ANGLE_30 {
		theta - trig::ANGLE_30 + trig::ANGLE_360
	} else {
		theta - trig::ANGLE_30
	};

	// sweep of the rays will be through 60 degrees
	for sweep in 0..trig::ANGLE_60 {
		log!("starting sweep {sweep}");

		let hdist = find_vertical_intersect(player_x, player_y, angle);
		let vdist = find_horizontal_intersect(player_x, player_y, angle);
		let dist = hdist.min(vdist) / trig::fisheye_correction(sweep);

		draw_wall_column(buf, sweep, dist);

		angle += 1;
		if angle >= trig::ANGLE_360 {
			angle -= trig::ANGLE_360;
		}
	}
}
