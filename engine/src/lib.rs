use base64::{Engine as _, engine::general_purpose};
use fp::{ ToFixedPoint, FromFixedPoint };
use wasm_bindgen::prelude::*;
extern crate web_sys;

mod consts;
mod trig;
mod utils;
mod raycast;
mod fp;

macro_rules! log {
	( $( $t:tt )* ) => {
		web_sys::console::log_1(&format!( $( $t )* ).into());
	}
}

struct ColumnRenderParameters<'a> {
	pub texture: &'a [u8],
	pub step: f64,
	pub wall_height: i32,
	pub tex_pos: f64,
	pub y_min: i32,
	pub y_max: i32,
}

impl<'a> ColumnRenderParameters<'_> {
	pub fn new(texture: &'a[u8], step: f64, wall_height: i32, tex_pos:f64, y_min: i32, y_max: i32) -> ColumnRenderParameters {
		ColumnRenderParameters { texture, step, wall_height, tex_pos, y_min, y_max }
	}

	pub fn step(&mut self) {
		self.tex_pos += self.step;
	}
}

struct TextureMap {
	texture_width: usize,
	texture_height: usize,
	texture_size: usize,
	num_textures: usize,
	textures: Vec<u8>
}

impl TextureMap {
	pub fn new(texture_width: usize, texture_height: usize, textures: Vec<u8>) -> TextureMap {
		let texture_size = texture_width * texture_height;
		let num_textures = textures.len() / (texture_size * 4);
		TextureMap { texture_width, texture_height, texture_size, num_textures, textures }
	}

	pub fn empty() -> TextureMap {
		TextureMap { texture_width: 0, texture_height: 0, texture_size: 0, num_textures: 0, textures: vec![] }
	}

	pub fn get(&self, code: u8, column: i32, flipped: bool) -> &[u8] {
		let column = if flipped { self.texture_width - 1 - column as usize } else { column as usize };
		let pos: usize = (self.texture_size * code as usize + column as usize * self.texture_width) * 4 as usize;
		&self.textures[pos..pos + self.texture_size]
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
	textures: TextureMap,
}

fn alpha_blend(c1: f64, a1: f64, c2: f64, a2: f64, ao: f64) -> f64 {
	(c1 * a1 + c2 * a2 * (1.0 - a1)) / ao
}

fn blend_colours(r1: u8, g1: u8, b1: u8, a1: u8, r2:u8, g2:u8, b2:u8, a2:u8) -> (u8, u8, u8, u8) {
	let fa1 = a1 as f64 / 255.0;
	let fa2 = a2 as f64 / 255.0;
	let fao = alpha_blend(1.0, fa1, 1.0, fa2, 1.0);

	let r = alpha_blend(r1 as f64, fa1, r2 as f64, fa2, fao) as u8;
	let g = alpha_blend(g1 as f64, fa1, g2 as f64, fa2, fao) as u8;
	let b = alpha_blend(b1 as f64, fa1, b2 as f64, fa2, fao) as u8;
	let a = (255.0 * fao) as u8;

	(r, g, b, a)
}

#[wasm_bindgen]
impl Cluiche {
	pub fn new() -> Cluiche {
		let world = raycast::World::new(13, 6, "WHHhHWHWHHHcWvOOOOVOVOOOcVVOOOOvOvOOOcVVOOOOVOXOOOsVVOOOOXOVOOOWVWHHHHWHWHHHWW").unwrap();
		let player = raycast::Player::new(160, 160, 0, 8, 10);
		let textures = TextureMap::empty();
		Cluiche { world, player, textures }
	}

	pub fn load_textures(&mut self, encoded: &str) {
		let bytes: Vec<u8> = general_purpose::STANDARD_NO_PAD.decode(encoded).expect("failed to decode textures");
		self.textures = TextureMap::new(consts::TEXTURE_WIDTH, consts::TEXTURE_HEIGHT, bytes);
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
			if let raycast::Tile::Wall(_, passable) = self.world.x_wall(grid_x, grid_y) {
				if !passable && (x1 < x_left || (x1 - x_left).abs() < 28) { // we crossed the wall or we're too close
					x1 = xp;
					hit_result = HitResult::SlideX;
				}
			}
		}

		if x1 > xp { // are we moving right
			if let raycast::Tile::Wall(_, passable) = self.world.x_wall(grid_x + 1, grid_y) { // wall found in current square (right edge)
				if !passable && (x1 > x_right || (x_right - x1).abs() < 28) { // we crossed the wall or we're too close
					x1 = xp;
					hit_result = HitResult::SlideX;
				}
			}
		}

		if y1 < yp { // are we moving up			
			if let raycast::Tile::Wall(_, passable) = self.world.y_wall(grid_x, grid_y) {
				if !passable && (y1 < y_top || (y1 - y_top).abs() < 28) {
					y1 = yp;
					hit_result = HitResult::SlideY;
				}
			}
		}

		if y1 > yp { // are we moving down
			if let raycast::Tile::Wall(_, passable) = self.world.y_wall(grid_x, grid_y + 1) {
				if !passable && (y1 > y_bottom || (y_bottom - y1).abs() < 28) {
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

					// check adjacent x wall (to left)
					if let raycast::Tile::Wall(_, x_passable) = self.world.x_wall(grid_x, grid_y - 1) { 
						if !x_passable && y1 < (y_top + 28) { // adjacent x wall found and new y coord is within 28 units
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
					}

					// check adjacent y wall (above)
					if let raycast::Tile::Wall(_, y_passable) = self.world.y_wall(grid_x - 1, grid_y) {
						if !y_passable && x1 < x_left + 28 {
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
				}

				// check region B-top right area
				if x1 > x_right - 32 && hit_result == HitResult::Nothing {
					
					// check adjacent x wall (to right)
					if let raycast::Tile::Wall(_, x_passable) = self.world.x_wall(grid_x + 1, grid_y - 1) {
						if !x_passable && y1 < y_top + 28 {
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
					}

					// check adjacent y wall (above)
					if let raycast::Tile::Wall(_, y_passable) = self.world.y_wall(grid_x + 1, grid_y) {
						if !y_passable && x1 > x_right - 28 {
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
			}

			// check region C-bottom left area
			if y1 > y_top + 32 && hit_result == HitResult::Nothing {
				if x1 < x_left + 32 {					
					
					// check adjacent x wall (to left)
					if let raycast::Tile::Wall(_, x_passable) = self.world.x_wall(grid_x, grid_y + 1) {
						if !x_passable && y1 > y_bottom - 28 {
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
					}
					
					// check adjacent y wall (below)
					if let raycast::Tile::Wall(_, y_passable) = self.world.y_wall(grid_x - 1, grid_y + 1) {
						if !y_passable && x1 < x_left + 28 {
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

				// check region D-bottom right area
				if x1 > x_right - 32 && hit_result == HitResult::Nothing {
					
					// check adjacent x wall (to right)
					if let raycast::Tile::Wall(_, x_passable) = self.world.x_wall(grid_x + 1, grid_y + 1) {
						if !x_passable && y1 > y_bottom - 28 {
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
					}

					// check adjacent y wall (below)
					if let raycast::Tile::Wall(_, y_passable) = self.world.y_wall(grid_x + 1, grid_y + 1) {
						if !y_passable && x1 > x_right - 28 {
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
		}
		
		if hit_result == HitResult::SlideX && y1 == yp {
			hit_result = HitResult::WallX;
		}

		if hit_result == HitResult::SlideY && x1 == xp {
			hit_result = HitResult::WallY;
		}

		self.player.pos(x1, y1);

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
	}

	pub fn player_turn_right(&mut self) {
		self.player.rotation(self.player.rotation + self.player.rotate_speed);
	}

	fn draw_wall_column(&self, buf: &mut[u8], column: i32, parameters: &mut Vec<ColumnRenderParameters>) {
		let y_min = parameters[0].y_min;
		let y_max = parameters[0].y_max;

		for y in y_min..=y_max {
			let mut r: u8 = 0;
			let mut g: u8 = 0;
			let mut b: u8 = 0;
			let mut a: u8 = 0;
			
			let idx: usize = 4 * (column + y * consts::PROJECTION_PLANE_WIDTH) as usize;

			for slice in parameters.iter_mut() {
				if y < slice.y_min || y > slice.y_max { break; }
				let tex_y = (slice.tex_pos.clamp(0.0, 63.0) as usize) * 4;
				
				(r, g, b, a) = blend_colours(r, g, b, a, slice.texture[tex_y + 0], slice.texture[tex_y + 1], slice.texture[tex_y + 2], slice.texture[tex_y + 3]);

				if a >= 255 { break; }
			}

			for slice in parameters.iter_mut() {
				if y < slice.y_min || y > slice.y_max { break; }
				slice.step();
			}

			(buf[idx + 0], buf[idx + 1], buf[idx + 2], buf[idx + 3]) = blend_colours(r, g, b, a, buf[idx + 0], buf[idx + 1], buf[idx + 2], buf[idx + 3]);
		}
	}

	fn draw_background(&self, buf: &mut[u8]) {

		for y in 0..consts::PROJECTION_PLANE_HEIGHT / 2 {
			for x in 0..consts::PROJECTION_PLANE_WIDTH {
				let idx: usize = 4 * (x + y * consts::PROJECTION_PLANE_WIDTH) as usize;
				buf[idx + 0] = 0x38;
				buf[idx + 1] = 0x38;
				buf[idx + 2] = 0x38;
				buf[idx + 3] = 0xFF; // alpha channel				
			}
		}

		for y in consts::PROJECTION_PLANE_HEIGHT / 2..consts::PROJECTION_PLANE_HEIGHT {
			for x in 0..consts::PROJECTION_PLANE_WIDTH {
				let idx: usize = 4 * (x + y * consts::PROJECTION_PLANE_WIDTH) as usize;
				buf[idx + 0] = 0x70;
				buf[idx + 1] = 0x70;
				buf[idx + 2] = 0x70;
				buf[idx + 3] = 0xFF; // alpha channel
			}
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
			let slices = self.world.find_closest_intersect(origin_x, origin_y, angle);
			if slices.len() <= 0 { continue; }
			let mut parameters: Vec<ColumnRenderParameters> = Vec::new();
			parameters.reserve(slices.len());

			// for each slice, get a reference to its texture and figure out how
			// it should be drawn
			for slice in slices {
				let dist = fp::div(slice.distance, trig::fisheye_correction(sweep)).to_i32();
				let wall_height: i32 = trig::wall_height(dist);
				let y_min = std::cmp::max(0, (200 - wall_height) / 2);
				let y_max = std::cmp::min(200 - 1, y_min + wall_height);
				let step: f64 = consts::TEXTURE_HEIGHT as f64 / wall_height as f64;
				let raycast::TextureCode::Wall(code, texture_column, flipped) = slice.texture;
				let texture = self.textures.get(code, texture_column, flipped);
				let tex_pos: f64 = (y_min as f64 - consts::PROJECTION_PLANE_HEIGHT as f64 / 2.0 + wall_height as f64 / 2.0) * step;
				parameters.push(ColumnRenderParameters::new(texture, step, wall_height, tex_pos, y_min, y_max))
			}

			self.draw_wall_column(buf, sweep, &mut parameters);

			angle += 1;
			if angle >= trig::ANGLE_360 {
				angle -= trig::ANGLE_360;
			}
		}
	}
}