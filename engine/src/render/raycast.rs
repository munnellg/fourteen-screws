use crate::scene::{ Tile, Scene };
use crate::trig;
use shared::consts;
use shared::fp;
use shared::fp::{ ToFixedPoint, FromFixedPoint };

#[derive(Copy, Clone)]
pub struct Intersection {
	pub x: i32,
	pub y: i32,
	pub dist: i32,
	pub texture: u32,
	pub texture_column: i32,
	pub reverse: bool,
}

impl Intersection {
	pub fn new(x: i32, y: i32, dist:i32, texture: u32, texture_column: i32, reverse: bool) -> Intersection {
		Intersection { x, y, dist, texture, texture_column, reverse }
	}
}

pub struct RayCaster {}

impl RayCaster {
	pub fn new() -> RayCaster {
		RayCaster {}
	}

	fn find_horizontal_intersect(&self, origin_x: i32, origin_y: i32, direction: i32, scene: &Scene) -> Vec<Intersection> {
		let step_x: i32; // distance to next vertical intersect
		let step_y: i32; // distance to next horizontal intersect
		let mut x: i32;  // x coordinate of current ray intersect
		let mut y: i32;  // y coordinate of current ray intersect
		let flipped: bool;

		let mut intersects = Vec::new();

		// determine if looking up or down and find horizontal intersection
		if direction > trig::ANGLE_0 && direction < trig::ANGLE_180 { // looking down
			step_x = trig::x_step(direction);
			step_y = consts::FP_TILE_SIZE;

			y = ((origin_y.to_i32() / consts::TILE_SIZE) * consts::TILE_SIZE + consts::TILE_SIZE).to_fp();
			x = fp::add(origin_x, fp::mul(fp::sub(y, origin_y), trig::itan(direction)));
			flipped = true;
		} else {                     // looking up
			step_x = trig::x_step(direction);
			step_y = -consts::FP_TILE_SIZE;

			y = ((origin_y.to_i32() / consts::TILE_SIZE) * consts::TILE_SIZE).to_fp();
			x = fp::add(origin_x, fp::mul(fp::sub(y, origin_y), trig::itan(direction)));
			flipped = false;
		}

		if direction == trig::ANGLE_0 || direction == trig::ANGLE_180 {
			return intersects;
		}

		// Cast x axis intersect rays, build up horizontal intersections
		loop {
			let grid_x = fp::div(x, consts::FP_TILE_SIZE).to_i32();
			let grid_y = fp::div(y, consts::FP_TILE_SIZE).to_i32();
			
			match scene.x_wall(grid_x, grid_y) {
				Tile::Wall(wall) => {
					let world_x  = x.to_i32();
					let world_y  = y.to_i32();
					let distance = fp::mul(fp::sub(y, origin_y), trig::isin(direction)).abs();
					let texture  = wall.texture;
					let texture_column = world_x & (consts::TILE_SIZE - 1);
					let intersection = Intersection::new(world_x, world_y, distance, texture, texture_column, flipped);
					intersects.push(intersection);
				},
				Tile::OutOfBounds => break,
				Tile::Empty => {}
			}

			x = fp::add(x, step_x);
			y = fp::add(y, step_y);
		}


		intersects
	}

	fn find_vertical_intersect(&self, origin_x: i32, origin_y: i32, direction: i32, scene: &Scene) -> Vec<Intersection> {
		let step_x: i32; // distance to next vertical intersect
		let step_y: i32; // distance to next horizontal intersect
		let mut x: i32;  // x coordinate of current ray intersect
		let mut y: i32;  // y coordinate of current ray intersect
		let flipped: bool;

		let mut intersects = Vec::new();

		// determine if looking left or right and find vertical intersection
		if direction <= trig::ANGLE_90 || direction > trig::ANGLE_270 { // looking right
			step_x = consts::FP_TILE_SIZE;
			step_y = trig::y_step(direction);
			
			x = ((origin_x.to_i32() / consts::TILE_SIZE) * consts::TILE_SIZE + consts::TILE_SIZE).to_fp();
			y = fp::add(origin_y, fp::mul(fp::sub(x, origin_x), trig::tan(direction)));
			
			flipped = false;
		} else {
			step_x = -consts::FP_TILE_SIZE;
			step_y = trig::y_step(direction);
			
			x = (((origin_x.to_i32() / consts::TILE_SIZE) * consts::TILE_SIZE)).to_fp();
			y = fp::add(origin_y, fp::mul(fp::sub(x, origin_x), trig::tan(direction)));
			
			flipped = true;
		};

		if direction == trig::ANGLE_90 || direction == trig::ANGLE_270 {
			return intersects;
		}

		loop {
			let grid_x = fp::div(x, consts::FP_TILE_SIZE).to_i32();
			let grid_y = fp::div(y, consts::FP_TILE_SIZE).to_i32();
			
			match scene.y_wall(grid_x, grid_y) {
				Tile::Wall(wall) => {
					let world_x  = x.to_i32();
					let world_y  = y.to_i32();
					let distance = fp::mul(fp::sub(x, origin_x), trig::icos(direction)).abs();
					let texture  = wall.texture;
					let texture_column = world_y & (consts::TILE_SIZE - 1);
					let intersection = Intersection::new(world_x, world_y, distance, texture, texture_column, flipped);
					intersects.push(intersection);
				},
				Tile::OutOfBounds => break,
				Tile::Empty => {}
			}

			x = fp::add(x, step_x);
			y = fp::add(y, step_y);
		}

		intersects
	}

	pub fn find_wall_intersections(&self, origin_x: i32, origin_y: i32, direction: i32, scene: &Scene) -> Vec<Intersection> {
		let hintersects = self.find_horizontal_intersect(origin_x, origin_y, direction, scene);
		let vintersects = self.find_vertical_intersect(origin_x, origin_y, direction, scene);

		let mut intersects = Vec::new();
		intersects.reserve(hintersects.len() + vintersects.len());

		let mut i = 0;
		let mut j = 0;

		while i < hintersects.len() && j < vintersects.len() {
			if hintersects[i].dist < vintersects[j].dist {
				intersects.push(hintersects[i]);
				i += 1;
			} else {
				intersects.push(vintersects[j]);
				j += 1;
			}
		}

		while i < hintersects.len() {			
			intersects.push(hintersects[i]);
			i += 1;
		}

		while j < vintersects.len() {
			intersects.push(vintersects[j]);
			j += 1;
		}

		intersects
	}
}