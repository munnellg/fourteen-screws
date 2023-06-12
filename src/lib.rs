extern crate web_sys;
use wasm_bindgen::prelude::*;

mod consts;
mod trig;
mod utils;
mod raycast;
mod fp;

use fp::{ ToFixedPoint, FromFixedPoint };

macro_rules! log {
	( $( $t:tt )* ) => {
		web_sys::console::log_1(&format!( $( $t )* ).into());
	}
}

#[derive(PartialEq)]
enum HitResult {
	Nothing,
	SlideX,
	SlideY,
	WallX,
	WallY,
}

#[wasm_bindgen]
pub struct Cluiche {
	world: raycast::World,
	player: raycast::Player,
}

#[wasm_bindgen]
impl Cluiche {
	pub fn new() -> Cluiche {
		let world = raycast::World::new(13, 6, "WHHHHWHWHHHHWVOOOOVOVOOOOVVOOOOVOVOOOOVVOOOOVOOOOOOVVOOOOOOVOOOOVWHHHHWHWHHHWW").unwrap();
		let player = raycast::Player::new(160, 160, 0, 5, 10);
		Cluiche { world, player }
	}

	fn move_player(&mut self, mut direction: i32, amount: i32) -> HitResult {
		while direction >= trig::ANGLE_360 { direction -= trig::ANGLE_360; }
		while direction < trig::ANGLE_0    { direction += trig::ANGLE_360; }

		let xp = self.player.x;
		let yp = self.player.y;

		// get bounds of the tile player currently occupies
		let x_left   = xp & 0xFFC0;
		let y_top    = yp & 0xFFC0;
		let x_right  = x_left + consts::TILE_SIZE;
		let y_bottom = y_top + consts::TILE_SIZE;

		let mut hit_result = HitResult::Nothing;

		let mut x1 = xp + fp::mul(trig::cos(direction), amount.to_fp()).to_i32();
		let mut y1 = yp + fp::mul(trig::sin(direction), amount.to_fp()).to_i32();
		
		let grid_x = x_left / consts::TILE_SIZE;
		let grid_y = y_top / consts::TILE_SIZE;

		if x1 < xp { // are we moving left
			if self.world.is_x_wall(grid_x, grid_y) {
				if x1 < x_left || (x1 - x_left).abs() < 28 { // we crossed the wall or we're too close
					x1 = xp;
					hit_result = HitResult::SlideX;
				}
			}
		}

		if x1 > xp { // are we moving right
			if self.world.is_x_wall(grid_x + 1, grid_y) { // wall found in current square (right edge)
				if x1 > x_right || (x_right - x1).abs() < 28 { // we crossed the wall or we're too close
					x1 = xp;
					hit_result = HitResult::SlideX;
				}
			}
		}

		if y1 < yp { // are we moving up			
			if self.world.is_y_wall(grid_x, grid_y) {
				if y1 < y_top || (y1 - y_top).abs() < 28 {
					y1 = yp;
					hit_result = HitResult::SlideY;
				}
			}
		}

		if y1 > yp { // are we moving down
			if self.world.is_y_wall(grid_x, grid_y + 1) {
				if y1 > y_bottom || (y_bottom - y1).abs() < 28 {
					y1 = yp;
					hit_result = HitResult::SlideY;
				}
			}
		}

		// A wall or object hasn't been hit yet. We must look further.
		// The current grid square will be divided into four regions:
		// A = top left; B = top right; C = bottom left; D = bottom right
		// Each of these regions will be checked to see if the player's new position
		// (x1, y1) is close to a wall or object that borders one of these regions.
		// Each grid square is 64x64 units, so each region to check is 32x32 units


		if hit_result == HitResult::Nothing {
			if y1 < (y_top + 32) {    // new y position falls in top half
				
				// check region A-top left area of grid
				if x1 < x_left + 32 { // new x position falls in left half
					let m_code_x = self.world.is_x_wall(grid_x, grid_y - 1); // check adjacent x wall (to left)
					let m_code_y = self.world.is_y_wall(grid_x - 1, grid_y); // check adjacent y wall (above)
					
					if m_code_x && y1 < (y_top + 28) { // adjacent x wall found and new y coord is within 28 units
						if x1 < x_left + 28 {
							if xp > x_left + 27 {
								x1 = xp;
								hit_result = HitResult::SlideX;
							} else {
								y1 = yp;
								hit_result = HitResult::SlideY;
							}
						}
					}

					if m_code_y && x1 < x_left + 28 {
						if y1 < y_top + 28 {
							if yp > y_top + 27 {
								y1 = yp;
								hit_result = HitResult::SlideY;
							} else {
								x1 = xp;
								hit_result = HitResult::SlideX;
							}
						}
					}
				}

				// check region B-top right area
				if x1 > x_right - 32 && hit_result == HitResult::Nothing {
					let m_code_x = self.world.is_x_wall(grid_x + 1, grid_y - 1); // check adjacent x wall (to right)
					let m_code_y = self.world.is_y_wall(grid_x + 1, grid_y); // check adjacent y wall (above)
					if m_code_x && y1 < y_top + 28 {
						if x1 > x_right - 28 {
							if xp < x_right - 27 {
								x1 = xp;
								hit_result = HitResult::SlideX;
							} else {
								y1 = yp;
								hit_result = HitResult::SlideY;
							}
						}
					}

					if m_code_y && x1 > x_right - 28 {
						if y1 < y_top + 28 {
							if yp < y_top + 27 {
								y1 = yp;
								hit_result = HitResult::SlideY;
							} else {
								x1 = xp;
								hit_result = HitResult::SlideX;
							}
						}
					}
				}
			}

			// check region C-bottom left area
			if y1 > y_top + 32 && hit_result == HitResult::Nothing {
				if x1 < x_left + 32 {
					let m_code_x = self.world.is_x_wall(grid_x, grid_y + 1); // check adjacent x wall (to left)
					let m_code_y = self.world.is_y_wall(grid_x - 1, grid_y + 1); // check adjacent y wall (below)
					
					if m_code_x && y1 > y_bottom - 28 {
						if x1 < x_left + 28 {
							if xp > x_left + 27 {
								x1 = xp;
								hit_result = HitResult::SlideX;
							} else {
								y1 = yp;
								hit_result = HitResult::SlideY;
							}
						}
					}

					if m_code_y && x1 < x_left + 28 {
						if y1 > y_bottom - 28 {
							if yp < y_bottom - 27 {
								y1 = yp;
								hit_result = HitResult::SlideY;
							} else {
								x1 = xp;
								hit_result = HitResult::SlideX;
							}
						}
					}
				}

				// check region D-bottom right area
				if x1 > x_right - 32 && hit_result == HitResult::Nothing {
					let m_code_x = self.world.is_x_wall(grid_x + 1, grid_y + 1); // check adjacent x wall (to right)
					let m_code_y = self.world.is_y_wall(grid_x + 1, grid_y + 1); // check adjacent y wall (below)
					
					if m_code_x && y1 > y_bottom - 28 {
						if x1 > x_right - 28 {
							if xp < x_right - 27 {
								x1 = xp;
								hit_result = HitResult::SlideX;
							} else {
								y1 = yp;
								hit_result = HitResult::SlideY;
							}
						}
					}

					if m_code_y && x1 > x_right - 28 {
						if y1 > y_bottom - 28 {
							if yp < y_bottom - 27 {
								y1 = yp;
								hit_result = HitResult::SlideY;
							} else {
								x1 = xp;
								hit_result = HitResult::SlideX;
							}
						}
					}
				}
			}
		}
		
		if hit_result == HitResult::SlideX && y1 == yp {
			hit_result = HitResult::WallX;
		}

		if hit_result == HitResult::SlideY && x1 == xp {
			hit_result = HitResult::WallY;
		}

		self.player.pos(x1, y1);

		log!("pos=({}, {}) a={}", self.player.x, self.player.y, self.player.rotation);

		hit_result
	}

	pub fn player_forward(&mut self) {
		self.move_player(self.player.rotation, self.player.move_speed);
	}

	pub fn player_back(&mut self) {
		self.move_player(self.player.rotation + trig::ANGLE_180, self.player.move_speed);
	}

	pub fn player_strafe_left(&mut self) {
		self.move_player(self.player.rotation - trig::ANGLE_90, self.player.move_speed);
	}

	pub fn player_strafe_right(&mut self) {
		self.move_player(self.player.rotation + trig::ANGLE_90, self.player.move_speed);
	}	

	pub fn player_turn_left(&mut self) {
		self.player.rotation(self.player.rotation - self.player.rotate_speed);
		log!("pos=({}, {}) a={}", self.player.x, self.player.y, self.player.rotation);
	}

	pub fn player_turn_right(&mut self) {
		self.player.rotation(self.player.rotation + self.player.rotate_speed);
		log!("pos=({}, {}) a={}", self.player.x, self.player.y, self.player.rotation);
	}

	fn draw_wall_column(&self, buf: &mut[u8], column: i32, dist: i32) {
		// get wall texture, draw into column
		let wall_height: i32 = trig::wall_height(dist);

		let y_min = std::cmp::max(0, (200 - wall_height) / 2);
		let y_max = std::cmp::min(200 - 1, y_min + wall_height);

		let mut colour: i32 = 255 - ((dist as f64 / 750.0) * 255.0) as i32;
		
		if colour < 20  { colour = 20; }
		if colour > 255 { colour = 255; }

		for y in y_min..=y_max {
			let idx: usize = 4 * (column + y * consts::PROJECTION_PLANE_WIDTH) as usize;
			buf[idx + 0] = colour as u8;
			buf[idx + 1] = colour as u8;
			buf[idx + 2] = colour as u8;
			buf[idx + 3] = 0xFF; // alpha channel
		}
	}

	fn draw_background(&self, buf: &mut[u8]) {
		let mut c = 255;

		for y in 0..consts::PROJECTION_PLANE_HEIGHT / 2 {
			for x in 0..consts::PROJECTION_PLANE_WIDTH {
				let idx: usize = 4 * (x + y * consts::PROJECTION_PLANE_WIDTH) as usize;
				buf[idx + 0] = c;
				buf[idx + 1] = 0x7D;
				buf[idx + 2] = 0xE1;
				buf[idx + 3] = 0xFF; // alpha channel				
			}

			c -= 1;
		}

		c = 22;
		for y in consts::PROJECTION_PLANE_HEIGHT / 2..consts::PROJECTION_PLANE_HEIGHT {
			for x in 0..consts::PROJECTION_PLANE_WIDTH {
				let idx: usize = 4 * (x + y * consts::PROJECTION_PLANE_WIDTH) as usize;
				buf[idx + 0] = c;
				buf[idx + 1] = 20;
				buf[idx + 2] = 20;
				buf[idx + 3] = 0xFF; // alpha channel
			}

			c += 1;
		}
	}

	pub fn render(&mut self, buf: &mut[u8]) {
		self.draw_background(buf);

		// theta is the direction player is facing
		// need to start out sweep 30 degrees to the left
		let mut angle = if self.player.rotation < trig::ANGLE_30 {
			self.player.rotation - trig::ANGLE_30 + trig::ANGLE_360
		} else {
			self.player.rotation - trig::ANGLE_30
		};

		// ray casting uses fixed point notation, so convert player coordinates to fixed point
		let origin_x = self.player.x.to_fp();
		let origin_y = self.player.y.to_fp();


		// sweep of the rays will be through 60 degrees
		for sweep in 0..trig::ANGLE_60 {
			let dist = self.world.find_closest_intersect(origin_x, origin_y, angle);
			let dist = fp::div(dist, trig::fisheye_correction(sweep));

			self.draw_wall_column(buf, sweep, dist.to_i32());

			angle += 1;
			if angle >= trig::ANGLE_360 {
				angle -= trig::ANGLE_360;
			}
		}
	}
}