use crate::scene::{ Tile, Scene };
use crate::trig;
use itertools::Itertools;
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

struct Ray<'a> {
	step_x: i32, // distance to next vertical intersect
	step_y: i32, // distance to next horizontal intersect
	x: i32,      // x coordinate of current ray intersect
	y: i32,      // y coordinate of current ray intersect
	flipped: bool,
	direction: i32,
	scene: &'a Scene,
	origin_x: i32,
	origin_y: i32,

	cast_ray: fn(&mut Self) -> Option<Intersection>,
	check_undefined: fn(&Self) -> bool
}

impl Ray<'_> {
	pub fn horizontal(origin_x: i32, origin_y: i32, direction: i32, scene: &Scene) -> Ray {
		let step_x: i32;
		let step_y: i32;
		let x: i32;
		let y: i32;
		let flipped: bool;

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

		Ray { step_x, step_y, x, y, flipped, direction, scene, origin_x, origin_y, check_undefined: Ray::horizontal_is_undefined, cast_ray: Ray::cast_horizontal }
	}

	pub fn vertical(origin_x: i32, origin_y: i32, direction: i32, scene: &Scene) -> Ray {
		let step_x: i32; // distance to next vertical intersect
		let step_y: i32; // distance to next horizontal intersect
		let x: i32;      // x coordinate of current ray intersect
		let y: i32;      // y coordinate of current ray intersect
		let flipped: bool;

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

		Ray { step_x, step_y, x, y, flipped, direction, scene, origin_x, origin_y, check_undefined: Ray::vertical_is_undefined, cast_ray: Ray::cast_vertical }
	}
	
	pub fn is_undefined(&self) -> bool {
		(self.check_undefined)(self)
	}

	pub fn cast(&mut self) -> Option<Intersection> {
		(self.cast_ray)(self)
	}

	fn horizontal_is_undefined(&self) -> bool {
		self.direction == trig::ANGLE_0 || self.direction == trig::ANGLE_180
	}

	fn vertical_is_undefined(&self) -> bool {
		self.direction == trig::ANGLE_90 || self.direction == trig::ANGLE_270
	}

	fn cast_horizontal(&mut self) -> Option<Intersection> {
		let mut result = None;

		while !result.is_some() {
			let grid_x = fp::div(self.x, consts::FP_TILE_SIZE).to_i32();
			let grid_y = fp::div(self.y, consts::FP_TILE_SIZE).to_i32();
			
			match self.scene.x_wall(grid_x, grid_y) {
				Tile::Wall(wall) => {
					let world_x  = self.x.to_i32();
					let world_y  = self.y.to_i32();
					let distance = fp::mul(fp::sub(self.y, self.origin_y), trig::isin(self.direction)).abs();
					let texture  = wall.texture;
					let texture_column = world_x & (consts::TILE_SIZE - 1);
					result = Some(Intersection::new(world_x, world_y, distance, texture, texture_column, self.flipped));
				},
				Tile::OutOfBounds => break,
				Tile::Empty => {}
			}

			self.x = fp::add(self.x, self.step_x);
			self.y = fp::add(self.y, self.step_y);
		}

		result
	}

	fn cast_vertical(&mut self) -> Option<Intersection> {
		let mut result = None;

		while !result.is_some() {
			let grid_x = fp::div(self.x, consts::FP_TILE_SIZE).to_i32();
			let grid_y = fp::div(self.y, consts::FP_TILE_SIZE).to_i32();
			
			match self.scene.x_wall(grid_x, grid_y) {
				Tile::Wall(wall) => {
					let world_x  = self.x.to_i32();
					let world_y  = self.y.to_i32();
					let distance = fp::mul(fp::sub(self.x, self.origin_x), trig::icos(self.direction)).abs();
					let texture  = wall.texture;
					let texture_column = world_y & (consts::TILE_SIZE - 1);
					result = Some(Intersection::new(world_x, world_y, distance, texture, texture_column, self.flipped));
				},
				Tile::OutOfBounds => break,
				Tile::Empty => {}
			}

			self.x = fp::add(self.x, self.step_x);
			self.y = fp::add(self.y, self.step_y);
		}

		result
	}
}

impl Iterator for Ray<'_> {
	type Item = Intersection;

	fn next(&mut self) -> Option<Self::Item> {
		self.cast()
	}
}

pub struct RayCaster {}

impl RayCaster {
	pub fn new() -> RayCaster {
		RayCaster {}
	}

	pub fn find_wall_intersections(&self, origin_x: i32, origin_y: i32, direction: i32, scene: &Scene) -> Vec<Intersection> {
		let ray_h = Ray::horizontal(origin_x, origin_y, direction, scene);
		let ray_v = Ray::vertical(origin_x, origin_y, direction, scene);

		if ray_h.is_undefined() { return ray_v.collect(); }
		if ray_v.is_undefined() { return ray_h.collect(); }

		vec![ray_h, ray_v].into_iter().kmerge_by(|a, b| a.dist < b.dist).collect()
	}
}


#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn name() {
		let raycaster = RayCaster::new();
	}
}
