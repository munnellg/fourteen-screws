use crate::trig;
use crate::consts;

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
	Wall,
}

pub struct World {
	tile_size: i32,
	width: i32,
	height: i32,
	h_walls: Vec<Tile>,
	v_walls: Vec<Tile>,
}

impl World {
	pub fn new(width: i32, height: i32, map_str: &str) -> Result<World, &str> {
		if width < 0 || height < 0 {
			return Err("Width and height must be positive values");
		}

		if (width * height) as usize != map_str.chars().count() {
			return Err("Width and height parameters do not match size of serialized map string");
		}

		let h_walls: Vec<Tile> = map_str.chars()
			.map(|c| {
				if c == 'W' || c == 'H' {
					Tile::Wall
				} else {
					Tile::Empty
				}
			})
			.collect();

		let v_walls: Vec<Tile> = map_str.chars()
			.map(|c| {
				if c == 'W' || c == 'V' {
					Tile::Wall
				} else {
					Tile::Empty
				}
			})
			.collect();

		Ok(World { tile_size: consts::TILE_SIZE, width, height, h_walls, v_walls })
	}

	fn is_within_bounds(&self, x: f64, y: f64) -> bool {
		let x = x as i32 / self.tile_size;
		let y = y as i32 / self.tile_size;
		x >= 0 && x < self.width && y >= 0 && y < self.height
	}

	fn is_h_wall(&self, x:f64, y:f64) -> bool {
		let x = x as i32 / self.tile_size;
		let y = y as i32 / self.tile_size;
		self.h_walls[(x + y  * self.width) as usize] == Tile::Wall
	}

	fn is_v_wall(&self, x:f64, y:f64) -> bool {
		let x = x as i32 / self.tile_size;
		let y = y as i32 / self.tile_size;
		self.v_walls[(x + y  * self.width) as usize] == Tile::Wall
	}

	pub fn find_horizontal_intersect(&self, origin_x: i32, origin_y: i32, direction: i32) -> f64 {
		let step_x: f64; // distance to next vertical intersect
		let step_y: f64; // distance to next horizontal intersect
		let mut x: f64;  // x coordinate of current ray intersect
		let mut y: f64;  // y coordinate of current ray intersect

		// determine if looking up or down and find horizontal intersection
		if direction > trig::ANGLE_0 && direction < trig::ANGLE_180 { // looking down
			let hi = (origin_y / self.tile_size) * self.tile_size + self.tile_size;
			step_x = trig::xstep(direction);
			step_y = consts::F_TILE_SIZE;
			x = origin_x as f64 + (hi - origin_y) as f64 * trig::itan(direction);
			y = hi as f64;
		} else {                     // looking up
			let hi = (origin_y / self.tile_size) * self.tile_size;
			step_x = trig::xstep(direction);
			step_y = -consts::F_TILE_SIZE;
			x = origin_x as f64 + (hi - origin_y) as f64 * trig::itan(direction);
			y = (hi - consts::TILE_SIZE) as f64;
		}

		if direction == trig::ANGLE_0 || direction == trig::ANGLE_180 {
			return f64::MAX;
		}

		// Cast x axis intersect rays, build up xSlice
		while self.is_within_bounds(x, y) {
			if self.is_h_wall(x, y) {
				return ((y - origin_y as f64) * trig::isin(direction)).abs();
			}

			x += step_x;
			y += step_y;
		}

		f64::MAX
	}

	pub fn find_vertical_intersect(&self, origin_x: i32, origin_y: i32, direction: i32) -> f64 {
		let step_x: f64;
		let step_y: f64;
		let mut x: f64;
		let mut y: f64;

		// determine if looking left or right and find vertical intersection
		if direction <= trig::ANGLE_90 || direction > trig::ANGLE_270 { // looking right
			let vi = (origin_x / self.tile_size) * self.tile_size + self.tile_size;
			
			step_x = consts::F_TILE_SIZE;
			step_y = trig::ystep(direction);
			
			x = vi as f64;
			y = origin_y as f64 + (vi - origin_x) as f64 * trig::tan(direction);
		} else {
			let vi = (origin_x / self.tile_size) * self.tile_size;
			
			step_x = -consts::F_TILE_SIZE;
			step_y = trig::ystep(direction);
			
			x = (vi - consts::TILE_SIZE) as f64;
			y = origin_y as f64 + (vi - origin_x) as f64 * trig::tan(direction);
		};

		if direction == trig::ANGLE_90 || direction == trig::ANGLE_270 {
			return f64::MAX;
		}

		// Cast y axis intersect rays, build up ySlice
		while self.is_within_bounds(x, y) {
			if self.is_v_wall(x, y) {
				return ((x - origin_x as f64) * trig::icos(direction)).abs();				
			}

			x += step_x;
			y += step_y;
		}

		f64::MAX
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use float_cmp;

	#[test]
	fn create_new_world() {
		let width: i32 = 3;
		let height: i32 = 3;
		let world_str = "WHWVOVWHW";
		let world = World::new(width, height, world_str).unwrap();

		assert_eq!(world.width, width);
		assert_eq!(world.height, height);
		assert_eq!(world.h_walls, vec!(
			Tile::Wall,  Tile::Wall,  Tile::Wall,
			Tile::Empty, Tile::Empty, Tile::Empty,
			Tile::Wall,  Tile::Wall,  Tile::Wall
		));
		assert_eq!(world.v_walls, vec!(
			Tile::Wall, Tile::Empty, Tile::Wall,
			Tile::Wall, Tile::Empty, Tile::Wall,
			Tile::Wall, Tile::Empty, Tile::Wall
		));
	}

	#[test]
	fn cast_ray() {
		let width: i32 = 3;
		let height: i32 = 3;
		let world_str = "WHWVOVWHW";
		let world = World::new(width, height, world_str).unwrap();

		assert_eq!(world.find_horizontal_intersect(64, 64, trig::ANGLE_0),   f64::MAX);
		assert_eq!(world.find_horizontal_intersect(64, 64, trig::ANGLE_90),  64.0);
		assert_eq!(world.find_horizontal_intersect(64, 64, trig::ANGLE_180), f64::MAX);
		assert_eq!(world.find_horizontal_intersect(64, 64, trig::ANGLE_270), 64.0);
		
		assert_eq!(world.find_vertical_intersect(64, 64, trig::ANGLE_0),   64.0);
		assert_eq!(world.find_vertical_intersect(64, 64, trig::ANGLE_90),  f64::MAX);
		assert_eq!(world.find_vertical_intersect(64, 64, trig::ANGLE_180), 64.0);
		assert_eq!(world.find_vertical_intersect(64, 64, trig::ANGLE_270), f64::MAX);
	}

	#[test]
	fn cast_ray_2() {
		let world = World::new(7, 7, "WHHHHHWVOOOOOVVOOOOOVVOOOOOVVOOOOOVVOOOOOVWHHHHHW").unwrap();
		float_cmp::assert_approx_eq!(f64, world.find_horizontal_intersect(76, 76, 295),   0.0, epsilon = 0.00000003, ulps = 2);
		float_cmp::assert_approx_eq!(f64, world.find_vertical_intersect(76, 76, 295),   0.0, epsilon = 0.00000003, ulps = 2);
	}
}