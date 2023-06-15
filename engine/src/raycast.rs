use crate::trig;
use crate::consts;
use crate::fp;
use crate::fp::{ ToFixedPoint, FromFixedPoint };

#[derive(Debug, Copy, Clone)]
pub enum TextureCode {
	Wall(u8, i32, bool)
}

#[derive(Debug, Copy, Clone)]
pub struct Slice {
	pub texture: TextureCode,
	pub distance: i32,
}

impl Slice {
	fn new(texture: TextureCode, distance: i32) -> Slice {
		Slice { texture, distance }
	}
}

pub struct Player {
	pub x: i32,
	pub y: i32,
	pub rotation: i32,
	pub move_speed: i32,
	pub rotate_speed: i32,
}

impl Player {
	pub fn new(x: i32, y: i32, rotation: i32, move_speed: i32, rotate_speed: i32) -> Player {
		Player { x, y, rotation, move_speed, rotate_speed }
	}

	pub fn pos(&mut self, x: i32, y: i32) {
		self.x = x;
		self.y = y;
	}

	pub fn rotation(&mut self, mut rotation: i32) {
		// ensure the input rotation is within bounds
		while rotation >= trig::ANGLE_360 { rotation -= trig::ANGLE_360; }
		while rotation < trig::ANGLE_0 { rotation += trig::ANGLE_360; }
		self.rotation = rotation;
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
	Empty,
	Wall(u8, bool),
	OutOfBounds,
}

pub struct World {
	width: i32,
	height: i32,
	y_walls: Vec<Tile>,
	x_walls: Vec<Tile>,
}

impl World {
	pub fn new(width: i32, height: i32, map_str: &str) -> Result<World, &str> {
		if width < 0 || height < 0 {
			return Err("Width and height must be positive values");
		}

		if (width * height) as usize != map_str.chars().count() {
			return Err("Width and height parameters do not match size of serialized map string");
		}

		let y_walls: Vec<Tile> = map_str.chars()
			.map(|c| {
				if c == 'W' || c == 'H' || c == 's' || c == 'c' {
					Tile::Wall(3, false)
				} else if c == 'h' { 
					Tile::Wall(65, false)
				} else {
					Tile::Empty
				}
			})
			.collect();

		let x_walls: Vec<Tile> = map_str.chars()
			.map(|c| {
				if c == 'W' || c == 'V' {
					Tile::Wall(3, false)
				} else if c == 'X' {
					Tile::Wall(1, true)
				} else if c == 'v' {
					Tile::Wall(65, false)
				} else if c == 's' {
					Tile::Wall(5, false)
				} else if c == 'c' {
					Tile::Wall(4, false)
				} else {
					Tile::Empty
				}
			})
			.collect();

		Ok(World { width, height, y_walls, x_walls })
	}

	fn is_within_bounds(&self, x: i32, y: i32) -> bool {
		x >= 0 && x < self.width && y >= 0 && y < self.height
	}

	pub fn y_wall(&self, x: i32, y: i32) -> Tile {
		if !self.is_within_bounds(x, y) { return Tile::OutOfBounds; }
		self.y_walls[(x + y  * self.width) as usize]
	}

	pub fn x_wall(&self, x: i32, y: i32) -> Tile {
		if !self.is_within_bounds(x, y) { return Tile::OutOfBounds; }
		self.x_walls[(x + y  * self.width) as usize]
	}

	fn find_horizontal_intersect(&self, origin_x: i32, origin_y: i32, direction: i32) -> Vec<Slice> {
		let step_x: i32; // distance to next vertical intersect
		let step_y: i32; // distance to next horizontal intersect
		let mut x: i32;  // x coordinate of current ray intersect
		let mut y: i32;  // y coordinate of current ray intersect
		let flipped: bool;

		let mut slices = Vec::new();

		// determine if looking up or down and find horizontal intersection
		if direction > trig::ANGLE_0 && direction < trig::ANGLE_180 { // looking down
			step_x = trig::xstep(direction);
			step_y = consts::FP_TILE_SIZE;

			y = ((origin_y.to_i32() / consts::TILE_SIZE) * consts::TILE_SIZE + consts::TILE_SIZE).to_fp();
			x = fp::add(origin_x, fp::mul(fp::sub(y, origin_y), trig::itan(direction)));
			flipped = true;
		} else {                     // looking up
			step_x = trig::xstep(direction);
			step_y = -consts::FP_TILE_SIZE;

			y = ((origin_y.to_i32() / consts::TILE_SIZE) * consts::TILE_SIZE).to_fp();
			x = fp::add(origin_x, fp::mul(fp::sub(y, origin_y), trig::itan(direction)));
			flipped = false;
		}

		if direction == trig::ANGLE_0 || direction == trig::ANGLE_180 {
			return slices;
			// return Slice::new(TextureCode::None, consts::FP_MAX_RAY_LENGTH);
		}

		// Cast x axis intersect rays, build up xSlice

		while self.is_within_bounds(fp::div(x, consts::FP_TILE_SIZE).to_i32(), fp::div(y, consts::FP_TILE_SIZE).to_i32()) {
			if let Tile::Wall(texture, _) = self.y_wall(fp::div(x, consts::FP_TILE_SIZE).to_i32(), fp::div(y, consts::FP_TILE_SIZE).to_i32()) {
				let slice = Slice::new(
					TextureCode::Wall(texture, x.to_i32() & (consts::TILE_SIZE - 1), flipped),
					fp::mul(fp::sub(y, origin_y), trig::isin(direction)).abs(),					
				);
				slices.push(slice);
			}

			x = fp::add(x, step_x);
			y = fp::add(y, step_y);
		}

		slices
	}

	fn find_vertical_intersect(&self, origin_x: i32, origin_y: i32, direction: i32) -> Vec<Slice> {
		let step_x: i32; // distance to next vertical intersect
		let step_y: i32; // distance to next horizontal intersect
		let mut x: i32;  // x coordinate of current ray intersect
		let mut y: i32;  // y coordinate of current ray intersect
		let flipped: bool;

		let mut slices = Vec::new();

		// determine if looking left or right and find vertical intersection
		if direction <= trig::ANGLE_90 || direction > trig::ANGLE_270 { // looking right
			step_x = consts::FP_TILE_SIZE;
			step_y = trig::ystep(direction);
			
			x = ((origin_x.to_i32() / consts::TILE_SIZE) * consts::TILE_SIZE + consts::TILE_SIZE).to_fp();
			y = fp::add(origin_y, fp::mul(fp::sub(x, origin_x), trig::tan(direction)));
			
			flipped = false;
		} else {
			step_x = -consts::FP_TILE_SIZE;
			step_y = trig::ystep(direction);
			
			x = (((origin_x.to_i32() / consts::TILE_SIZE) * consts::TILE_SIZE)).to_fp();
			y = fp::add(origin_y, fp::mul(fp::sub(x, origin_x), trig::tan(direction)));
			
			flipped = true;
		};

		if direction == trig::ANGLE_90 || direction == trig::ANGLE_270 {
			return slices;
		}

		// Cast y axis intersect rays, build up ySlice
		while self.is_within_bounds(fp::div(x, consts::FP_TILE_SIZE).to_i32(), fp::div(y, consts::FP_TILE_SIZE).to_i32()) {
			if let Tile::Wall(texture, _) = self.x_wall(fp::div(x, consts::FP_TILE_SIZE).to_i32(), fp::div(y, consts::FP_TILE_SIZE).to_i32()) {
				let slice = Slice::new(
					TextureCode::Wall(texture, y.to_i32() & (consts::TILE_SIZE - 1), flipped),
					fp::mul(fp::sub(x, origin_x), trig::icos(direction)).abs()
				);

				slices.push(slice);
			}

			x = fp::add(x, step_x);
			y = fp::add(y, step_y);
		}

		slices
	}

	pub fn find_closest_intersect(&self, origin_x: i32, origin_y: i32, direction: i32) -> Vec<Slice> {
		let hslices = self.find_horizontal_intersect(origin_x, origin_y, direction);
		let vslices = self.find_vertical_intersect(origin_x, origin_y, direction);
		
		let mut slices = Vec::new();
		slices.reserve(hslices.len() + vslices.len());

		let mut i = 0;
		let mut j = 0;

		while i < hslices.len() && j < vslices.len() {
			if hslices[i].distance < vslices[j].distance {
				slices.push(hslices[i]);
				i += 1;
			} else {
				slices.push(vslices[j]);
				j += 1;
			}
		}

		while i < hslices.len() {			
			slices.push(hslices[i]);
			i += 1;
		}

		while j < vslices.len() {
			slices.push(vslices[j]);
			j += 1;
		}

		slices
	}
}

// #[cfg(test)]
// mod test {
// 	use super::*;
// 	use float_cmp;

// 	#[test]
// 	fn create_new_world() {
// 		let width: i32 = 3;
// 		let height: i32 = 3;
// 		let world_str = "WHWVOVWHW";
// 		let world = World::new(width, height, world_str).unwrap();

// 		assert_eq!(world.width, width);
// 		assert_eq!(world.height, height);
// 		assert_eq!(world.y_walls, vec!(
// 			Tile::Wall,  Tile::Wall,  Tile::Wall,
// 			Tile::Empty, Tile::Empty, Tile::Empty,
// 			Tile::Wall,  Tile::Wall,  Tile::Wall
// 		));
// 		assert_eq!(world.x_walls, vec!(
// 			Tile::Wall, Tile::Empty, Tile::Wall,
// 			Tile::Wall, Tile::Empty, Tile::Wall,
// 			Tile::Wall, Tile::Empty, Tile::Wall
// 		));
// 	}

// 	#[test]
// 	fn cast_ray() {
// 		let width: i32 = 3;
// 		let height: i32 = 3;
// 		let world_str = "WHWVOVWHW";
// 		let world = World::new(width, height, world_str).unwrap();

// 		assert_eq!(world.find_horizontal_intersect(64.to_fp(), 64.to_fp(), trig::ANGLE_0).to_i32(),   consts::MAX_RAY_LENGTH);
// 		assert_eq!(world.find_horizontal_intersect(64.to_fp(), 64.to_fp(), trig::ANGLE_90).to_i32(),  64);
// 		assert_eq!(world.find_horizontal_intersect(64.to_fp(), 64.to_fp(), trig::ANGLE_180).to_i32(), consts::MAX_RAY_LENGTH);
// 		assert_eq!(world.find_horizontal_intersect(64.to_fp(), 64.to_fp(), trig::ANGLE_270).to_i32(), 64);
		
// 		assert_eq!(world.find_vertical_intersect(64.to_fp(), 64.to_fp(), trig::ANGLE_0).to_i32(),   64);
// 		assert_eq!(world.find_vertical_intersect(64.to_fp(), 64.to_fp(), trig::ANGLE_90).to_i32(),  consts::MAX_RAY_LENGTH);
// 		assert_eq!(world.find_vertical_intersect(64.to_fp(), 64.to_fp(), trig::ANGLE_180).to_i32(), 64);
// 		assert_eq!(world.find_vertical_intersect(64.to_fp(), 64.to_fp(), trig::ANGLE_270).to_i32(), consts::MAX_RAY_LENGTH);

// 		let world = World::new(7, 7, "WHHHHHWVOOOOOVVOOOOOVVOOOOOVVOOOOOVVOOOOOVWHHHHHW").unwrap();
// 		assert_eq!(world.find_horizontal_intersect(76.to_fp(), 76.to_fp(), 295).to_i32(), 374);
// 		assert_eq!(world.find_vertical_intersect(76.to_fp(), 76.to_fp(), 295).to_i32(), consts::MAX_RAY_LENGTH);

// 		assert_eq!(world.find_horizontal_intersect(160.to_fp(), 160.to_fp(), 1730).to_i32(), 274);
// 		assert_eq!(world.find_vertical_intersect(160.to_fp(), 160.to_fp(), 1730).to_i32(), consts::MAX_RAY_LENGTH);

// 		let world = World::new(13, 6, "WHHHHWHWHHHHWVOOOOVOVOOOOVVOOOOVOVOOOOVVOOOOVOOOOOOVVOOOOOOVOOOOVWHHHHWHWHHHWW").unwrap();
// 		assert_eq!(world.find_vertical_intersect(698.to_fp(), 145.to_fp(), 820).to_i32(), 278);
// 	}
// }