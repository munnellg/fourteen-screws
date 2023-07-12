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

trait Ray {
	fn is_undefined(&self) -> bool;
}

struct RayMeta<'a> {
	pub step_x: i32,        // distance to next vertical intersect
	pub step_y: i32,        // distance to next horizontal intersect
	pub x: i32,             // x coordinate of current ray intersect
	pub y: i32,             // y coordinate of current ray intersect
	pub flipped: bool,      // should the texture of the encountered surface be rendered backwards
	pub direction: i32,     // direction in which the ray is cast
	pub scene: &'a Scene,   // the environment in which the ray is being cast
	pub origin_x: i32,      // x point of origin of the ray in fixed point representation
	pub origin_y: i32,      // y point of origin of the ray in fixed point representation
	pub sweep: i32,
}

struct RayH<'a> {
	meta: RayMeta<'a>,
}

impl RayH<'_> {
	pub fn new(origin_x: i32, origin_y: i32, direction: i32, sweep: i32, scene: &Scene) -> RayH {
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

		let meta = RayMeta { step_x, step_y, x, y, flipped, direction, scene, origin_x, origin_y, sweep };
		RayH { meta }
	}
}

impl Ray for RayH<'_> {
	fn is_undefined(&self) -> bool {
		self.meta.direction == trig::ANGLE_0 || self.meta.direction == trig::ANGLE_180
	}
}

impl Iterator for RayH<'_> {
	type Item = Intersection;
	
	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		let mut result = None;

		while !result.is_some() {
			let grid_x = fp::div(self.meta.x, consts::FP_TILE_SIZE).to_i32();
			let grid_y = fp::div(self.meta.y, consts::FP_TILE_SIZE).to_i32();
			
			match self.meta.scene.y_wall(grid_x, grid_y) {
				Tile::Surface(wall) => {
					let world_x  = self.meta.x.to_i32();
					let world_y  = self.meta.y.to_i32();
					let distance = fp::mul(fp::sub(self.meta.y, self.meta.origin_y), trig::isin(self.meta.direction)).abs();
					let distance = fp::div(distance, trig::fisheye_correction(self.meta.sweep)).to_i32();
					let texture  = wall.texture;
					let texture_column = world_x & (consts::TILE_SIZE - 1);
					result = Some(Intersection::new(world_x, world_y, distance, texture, texture_column, self.meta.flipped));
				},
				Tile::OutOfBounds => break,
				Tile::Empty => {}
			}

			self.meta.x = fp::add(self.meta.x, self.meta.step_x);
			self.meta.y = fp::add(self.meta.y, self.meta.step_y);
		}

		result
	}
}

struct RayV<'a> {
	meta: RayMeta<'a>,
}

impl RayV<'_> {
	pub fn new(origin_x: i32, origin_y: i32, direction: i32, sweep: i32, scene: &Scene) -> RayV {
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

		let meta = RayMeta { step_x, step_y, x, y, flipped, direction, scene, origin_x, origin_y, sweep };
		RayV { meta }
	}
}

impl Ray for RayV<'_> {
	fn is_undefined(&self) -> bool {
		self.meta.direction == trig::ANGLE_90 || self.meta.direction == trig::ANGLE_270
	}
}

impl Iterator for RayV<'_> {
	type Item = Intersection;
	
	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		let mut result = None;

		while !result.is_some() {
			let grid_x = fp::div(self.meta.x, consts::FP_TILE_SIZE).to_i32();
			let grid_y = fp::div(self.meta.y, consts::FP_TILE_SIZE).to_i32();

			match self.meta.scene.x_wall(grid_x, grid_y) {
				Tile::Surface(wall) => {					
					let world_x  = self.meta.x.to_i32();
					let world_y  = self.meta.y.to_i32();
					let distance = fp::mul(fp::sub(self.meta.x, self.meta.origin_x), trig::icos(self.meta.direction)).abs();
					let distance = fp::div(distance, trig::fisheye_correction(self.meta.sweep)).to_i32();
					let texture  = wall.texture;
					let texture_column = world_y & (consts::TILE_SIZE - 1);
					result = Some(Intersection::new(world_x, world_y, distance, texture, texture_column, self.meta.flipped));
				},
				Tile::OutOfBounds => break,
				Tile::Empty => {}
			}

			self.meta.x = fp::add(self.meta.x, self.meta.step_x);
			self.meta.y = fp::add(self.meta.y, self.meta.step_y);
		}

		result
	}
}

pub fn find_wall_intersections(origin_x: i32, origin_y: i32, direction: i32, sweep: i32, scene: &Scene) -> Vec<Intersection> {
	let ray_h = RayH::new(origin_x, origin_y, direction, sweep, scene);
	let ray_v = RayV::new(origin_x, origin_y, direction, sweep, scene);

	if ray_h.is_undefined() { return ray_v.collect(); }
	if ray_v.is_undefined() { return ray_h.collect(); }

	ray_h.merge_by(ray_v, |a, b| a.dist < b.dist).collect()
}

pub fn find_floor_intersection(origin_x: i32, origin_y: i32, direction: i32, row: i32, column: i32, scene: &Scene) -> Option<Intersection> {
	// convert to fixed point
	let player_height = consts::PLAYER_HEIGHT.to_fp(); 
	let pp_distance   = consts::DISTANCE_TO_PROJECTION_PLANE.to_fp();

	// adding 1 to the row exactly on the horizon avoids a division by one error
	// doubles up the texture at the vanishing point, but probably fine
	let row = if row == consts::PROJECTION_PLANE_HORIZON { (row + 1).to_fp() } else { row.to_fp() };

	let ratio = fp::div(player_height, fp::sub(row, consts::PROJECTION_PLANE_HORIZON.to_fp()));

	let distance = fp::mul(fp::floor(fp::mul(pp_distance, ratio)), trig::fisheye_correction(column));

	let x_end = fp::floor(fp::mul(distance, trig::cos(direction)));
	let y_end = fp::floor(fp::mul(distance, trig::sin(direction)));

	let x_end = fp::add(origin_x, x_end);
	let y_end = fp::add(origin_y, y_end);
	
	let x = fp::floor(fp::div(x_end, consts::FP_TILE_SIZE)).to_i32();
	let y = fp::floor(fp::div(y_end, consts::FP_TILE_SIZE)).to_i32();
	
	let tex_x = x_end.to_i32() & (consts::TILE_SIZE - 1);
	let tex_y = y_end.to_i32() & (consts::TILE_SIZE - 1);

	match scene.floor(x, y) {
		Tile::Surface(floor) => Some(Intersection::new(tex_x, tex_y, distance, floor.texture, 0, false)),
		_ => None,
	}
}

pub fn find_ceiling_intersection(origin_x: i32, origin_y: i32, direction: i32, row: i32, column: i32, scene: &Scene) -> Option<Intersection> {
	// convert to fixed point
	let player_height = consts::PLAYER_HEIGHT.to_fp(); 
	let pp_distance   = consts::DISTANCE_TO_PROJECTION_PLANE.to_fp();
	let wall_height   = consts::WALL_HEIGHT.to_fp();

	// adding 1 to the row exactly on the horizon avoids a division by one error
	// doubles up the texture at the vanishing point, but probably fine
	let row = if row == consts::PROJECTION_PLANE_HORIZON { (row + 1).to_fp() } else { row.to_fp() };

	let ratio = fp::div(fp::sub(wall_height, player_height), fp::sub(consts::PROJECTION_PLANE_HORIZON.to_fp(), row));

	let distance = fp::mul(fp::floor(fp::mul(pp_distance, ratio)), trig::fisheye_correction(column));

	let x_end = fp::floor(fp::mul(distance, trig::cos(direction)));
	let y_end = fp::floor(fp::mul(distance, trig::sin(direction)));

	let x_end = fp::add(origin_x, x_end);
	let y_end = fp::add(origin_y, y_end);
	
	let x = fp::floor(fp::div(x_end, consts::FP_TILE_SIZE)).to_i32();
	let y = fp::floor(fp::div(y_end, consts::FP_TILE_SIZE)).to_i32();
	
	let tex_x = x_end.to_i32() & (consts::TILE_SIZE - 1);
	let tex_y = y_end.to_i32() & (consts::TILE_SIZE - 1);

	match scene.ceiling(x, y) {
		Tile::Surface(ceiling) => Some(Intersection::new(tex_x, tex_y, distance, ceiling.texture, 0, false)),
		_ => None,
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use std::fs;
	use std::path::Path;
	use std::path::PathBuf;

	fn load_scene(fname: &PathBuf) -> Result<Scene, Box<dyn std::error::Error>> {
		let contents = fs::read_to_string(fname)?;
		let json: serde_json::Value = serde_json::from_str(contents.as_str())?;
		let scene = Scene::try_from(&json)?;
		Ok(scene)
	}

	#[test]
	fn test_facing_directly_right() {
		let fname = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("resources").join("test-scene-1.json");
		let scene = load_scene(&fname).expect("Failed to load scene for test");

		let intersections = find_wall_intersections(128.to_fp(), 128.to_fp(), trig::ANGLE_0, consts::PROJECTION_PLANE_WIDTH / 2, &scene);

		assert_eq!(1, intersections.len());

		let intersection = intersections[0];
		assert_eq!(128, intersection.dist.to_i32());
	}

	#[test]
	fn test_against_wall() {
		let fname = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("resources").join("test-scene-1.json");
		let scene = load_scene(&fname).expect("Failed to load scene for test");

		let intersections = find_wall_intersections(28.to_fp(), 28.to_fp(), trig::ANGLE_270, consts::PROJECTION_PLANE_WIDTH / 2, &scene);

		assert_eq!(1, intersections.len());

		let intersection = intersections[0];
		assert_eq!(28, intersection.dist.to_i32());
	}
}
