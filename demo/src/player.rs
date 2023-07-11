use fourteen_screws::{ Camera, Scene };
use fourteen_screws::maths;
use fourteen_screws::maths::{ ToFixedPoint, FromFixedPoint };
use fourteen_screws::trig;
use fourteen_screws::Tile;

use wasm_bindgen::prelude::*;
use web_sys;

macro_rules! log {
	( $( $t:tt )* ) => {
		web_sys::console::log_1(&format!( $( $t )* ).into());
	}
}

#[derive(PartialEq)]
pub enum HitResult {
	Nothing,
	SlideX,
	SlideY,
	WallX,
	WallY,
}

pub struct Player {
	pub camera: Camera,
	move_speed: i32,
	rotate_speed: i32,
	margin: i32,
}

impl Player {
	pub fn new(camera: Camera, move_speed: i32, rotate_speed: i32, margin: i32) -> Player {
		Player { camera, move_speed, rotate_speed, margin }
	}

	fn translate(&mut self, mut direction: i32, amount: i32, scene: &Scene) -> HitResult {
		while direction >= trig::ANGLE_360 { direction -= trig::ANGLE_360; }
		while direction < trig::ANGLE_0    { direction += trig::ANGLE_360; }

		let xp = self.camera.x();
		let yp = self.camera.y();

		let half_tile = fourteen_screws::TILE_SIZE >> 1;

		// get bounds of the tile player currently occupies
		let x_left   = xp & 0xFFC0;
		let y_top    = yp & 0xFFC0;
		let x_right  = x_left + fourteen_screws::TILE_SIZE;
		let y_bottom = y_top + fourteen_screws::TILE_SIZE;

		let mut hit_result = HitResult::Nothing;

		let mut x1 = xp + maths::mul(trig::cos(direction), amount.to_fp()).to_i32();
		let mut y1 = yp + maths::mul(trig::sin(direction), amount.to_fp()).to_i32();
		
		let grid_x = x_left / fourteen_screws::TILE_SIZE;
		let grid_y = y_top / fourteen_screws::TILE_SIZE;

		if x1 < xp { // are we moving left
			if let Tile::Wall(wall) = scene.x_wall(grid_x, grid_y) {
				if !wall.passable && (x1 < x_left || (x1 - x_left).abs() < self.margin) { // we crossed the wall or we're too close
					log!("Blocked Left");
					x1 = xp;
					hit_result = HitResult::SlideX;
				}
			}
		}

		if x1 > xp { // are we moving right
			if let Tile::Wall(wall) = scene.x_wall(grid_x + 1, grid_y) { // wall found in current square (right edge)
				if !wall.passable && (x1 > x_right || (x_right - x1).abs() < self.margin) { // we crossed the wall or we're too close
					x1 = xp;
					hit_result = HitResult::SlideX;
				}
			} else if let Tile::OutOfBounds = scene.x_wall(grid_x + 1, grid_y) {
				log!("TILE IS OUT OF BOUNDS");
			}
		}

		if y1 < yp { // are we moving up			
			if let Tile::Wall(wall) = scene.y_wall(grid_x, grid_y) {
				if !wall.passable && (y1 < y_top || (y1 - y_top).abs() < self.margin) {
					log!("Blocked Up");
					y1 = yp;
					hit_result = HitResult::SlideY;
				}
			}
		}

		if y1 > yp { // are we moving down
			if let Tile::Wall(wall) = scene.y_wall(grid_x, grid_y + 1) {
				if !wall.passable && (y1 > y_bottom || (y_bottom - y1).abs() < self.margin) {
					log!("Blocked Down");
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
			if y1 < (y_top + half_tile) {    // new y position falls in top half
				
				// check region A-top left area of grid
				if x1 < x_left + half_tile { // new x position falls in left half

					// check adjacent x wall (to left)
					if let Tile::Wall(wall) = scene.x_wall(grid_x, grid_y - 1) { 
						if !wall.passable && y1 < (y_top + self.margin) { // adjacent x wall found and new y coord is within 28 units
							if x1 < x_left + self.margin {
								if xp > x_left + (self.margin - 1) {
									x1 = xp;
									hit_result = HitResult::SlideX;
								} else {
									y1 = yp;
									hit_result = HitResult::SlideY;
								}
							}
						}
					}

					// check adjacent y wall (above)
					if let Tile::Wall(wall) = scene.y_wall(grid_x - 1, grid_y) {
						if !wall.passable && x1 < x_left + self.margin {
							if y1 < y_top + self.margin {
								if yp > y_top + (self.margin - 1) {
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

				// check region B-top right area
				if x1 > x_right - half_tile && hit_result == HitResult::Nothing {
					
					// check adjacent x wall (to right)
					if let Tile::Wall(wall) = scene.x_wall(grid_x + 1, grid_y - 1) {
						if !wall.passable && y1 < y_top + self.margin {
							if x1 > x_right - self.margin {
								if xp < x_right - (self.margin - 1) {
									x1 = xp;
									hit_result = HitResult::SlideX;
								} else {
									y1 = yp;
									hit_result = HitResult::SlideY;
								}
							}
						}
					}

					// check adjacent y wall (above)
					if let Tile::Wall(wall) = scene.y_wall(grid_x + 1, grid_y) {
						if !wall.passable && x1 > x_right - self.margin {
							if y1 < y_top + self.margin {
								if yp < y_top + (self.margin - 1) {
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

			// check region C-bottom left area
			if y1 > y_top + half_tile && hit_result == HitResult::Nothing {
				if x1 < x_left + half_tile {					
					
					// check adjacent x wall (to left)
					if let Tile::Wall(wall) = scene.x_wall(grid_x, grid_y + 1) {
						if !wall.passable && y1 > y_bottom - self.margin {
							if x1 < x_left + self.margin {
								if xp > x_left + (self.margin - 1) {
									x1 = xp;
									hit_result = HitResult::SlideX;
								} else {
									y1 = yp;
									hit_result = HitResult::SlideY;
								}
							}
						}
					}
					
					// check adjacent y wall (below)
					if let Tile::Wall(wall) = scene.y_wall(grid_x - 1, grid_y + 1) {
						if !wall.passable && x1 < x_left + self.margin {
							if y1 > y_bottom - self.margin {
								if yp < y_bottom - (self.margin - 1) {
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

				// check region D-bottom right area
				if x1 > x_right - half_tile && hit_result == HitResult::Nothing {
					
					// check adjacent x wall (to right)
					if let Tile::Wall(wall) = scene.x_wall(grid_x + 1, grid_y + 1) {
						if !wall.passable && y1 > y_bottom - self.margin {
							if x1 > x_right - self.margin {
								if xp < x_right - (self.margin - 1) {
									x1 = xp;
									hit_result = HitResult::SlideX;
								} else {
									y1 = yp;
									hit_result = HitResult::SlideY;
								}
							}
						}
					}

					// check adjacent y wall (below)
					if let Tile::Wall(wall) = scene.y_wall(grid_x + 1, grid_y + 1) {
						if !wall.passable && x1 > x_right - self.margin {
							if y1 > y_bottom - self.margin {
								if yp < y_bottom - (self.margin - 1) {
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
		}
		
		if hit_result == HitResult::SlideX && y1 == yp {
			hit_result = HitResult::WallX;
		}

		if hit_result == HitResult::SlideY && x1 == xp {
			hit_result = HitResult::WallY;
		}

		self.camera.move_to(x1, y1);

		hit_result
	}

	pub fn forward(&mut self, scene: &Scene) -> HitResult {
		return self.translate(self.camera.angle(), self.move_speed, scene);
	}

	pub fn back(&mut self, scene: &Scene) -> HitResult {
		return self.translate(self.camera.angle() + trig::ANGLE_180, self.move_speed, scene);
	}

	pub fn strafe_left(&mut self, scene: &Scene) -> HitResult {
		return self.translate(self.camera.angle() - trig::ANGLE_90, self.move_speed, scene);
	}

	pub fn strafe_right(&mut self, scene: &Scene) -> HitResult {
		return self.translate(self.camera.angle() + trig::ANGLE_90, self.move_speed, scene);
	}	

	pub fn turn_left(&mut self) {
		self.camera.rotate(-self.rotate_speed);
		log!("{}", self.camera.angle());
	}

	pub fn turn_right(&mut self) {
		self.camera.rotate(self.rotate_speed);
		log!("{}", self.camera.angle());
	}
}