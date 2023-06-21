use crate::maths::trig;
use crate::maths::fp;
use crate::maths::fp::{ ToFixedPoint, FromFixedPoint };
use crate::consts;

#[derive(Debug, Copy, Clone)]
enum TextureCode {
	None,
	Wall(u8, i32, bool),
	Floor(u8, i32, i32),
	Ceiling(u8, i32, i32),
}

#[derive(Debug, Copy, Clone)]
struct Slice {
	pub texture: TextureCode,
	pub distance: i32,
}

impl Slice {
	fn new(texture: TextureCode, distance: i32) -> Slice {
		Slice { texture, distance }
	}
}

enum Tile {
	OutOfBounds,
	Wall(u8, i32),
	Floor,
	Ceiling,
}

struct Camera {
	x: i32,
	y: i32,
	angle: i32,
	horizon: i32,
}

impl Camera {
	pub fn new(x: i32, y: i32, angle: i32, horizon: i32) -> Camera {
		Camera { x: x.to_fp(), y: y.to_fp() , angle, horizon }
	}

	pub fn default() -> Camera {
		Camera::new(0, 0, 0, consts::PROJECTION_PLANE_HORIZON)
	}

	pub fn x(&self) -> i32 {
		self.x.to_i32()
	}

	pub fn set_x(&mut self, x: i32) {
		self.x = x.to_fp();
	}

	pub fn y(&self) -> i32 {
		self.y.to_i32()
	}

	pub fn set_y(&mut self, y: i32) {
		self.y = y.to_fp()
	}
}

struct Scene {
	width: i32,
	height: i32,
	y_walls: Vec<Tile>,
	x_walls: Vec<Tile>,
	floor: Vec<Tile>,
	ceiling: Vec<Tile>,
}

impl Scene {
	pub fn new(width: i32, height: i32) -> Result<Scene, &'static str> {
		if width < 0 || height < 0 {
			return Err("Width and height must be positive values");
		}

		let y_walls = Vec::new();
		let x_walls = Vec::new();
		let floor   = Vec::new();
		let ceiling = Vec::new();

		Ok(Scene { width, height, y_walls, x_walls, floor, ceiling })
	}

	pub fn is_within_bounds(&self, x: i32, y: i32) -> bool {
		x >= 0 && x < self.width && y >= 0 && y < self.height
	}

	pub fn y_wall(&self, x: i32, y: i32) -> &Tile {
		if !self.is_within_bounds(x, y) { return &Tile::OutOfBounds; }
		&self.y_walls[(x + y  * self.width) as usize]
	}

	pub fn x_wall(&self, x: i32, y: i32) -> &Tile {
		if !self.is_within_bounds(x, y) { return &Tile::OutOfBounds; }
		&self.x_walls[(x + y  * self.width) as usize]
	}
}

struct RenderConfig {

}

impl RenderConfig {
	pub fn new() -> RenderConfig {
		RenderConfig {}
	}

	pub fn default() -> RenderConfig {
		RenderConfig {}
	}
}

struct Renderer {

}

impl Renderer {
	pub fn new(config: &RenderConfig) -> Renderer {
		Renderer {}
	}

	fn draw_to_buffer(&self, buf: &mut[u8]) {

	}

	pub fn render(&self, buf: &mut[u8], scene: &Scene, camera: &Camera) {

		// theta is the direction player is facing
		// need to start out sweep 30 degrees to the left
		let mut angle = if camera.angle < trig::ANGLE_30 {
			camera.angle - trig::ANGLE_30 + trig::ANGLE_360
		} else {
			camera.angle - trig::ANGLE_30
		};

		// ray casting uses fixed point notation, so convert player coordinates to fixed point
		let origin_x = camera.x.to_fp();
		let origin_y = camera.y.to_fp();

		// sweep of the rays will be through 60 degrees
		for sweep in 0..trig::ANGLE_60 {
			let slices = self.find_wall_intersections(origin_x, origin_y, angle, scene);
		// 	if slices.len() <= 0 { continue; }
		// 	let mut parameters: Vec<ColumnRenderParameters> = Vec::new();
		// 	parameters.reserve(slices.len());

		// 	// for each slice, get a reference to its texture and figure out how
		// 	// it should be drawn
		// 	for slice in slices {
		// 		let dist = fp::div(slice.distance, trig::fisheye_correction(sweep)).to_i32();
		// 		let wall_height: i32 = trig::wall_height(dist);
		// 		let y_min = std::cmp::max(0, self.world.horizon() - wall_height / 2);
		// 		let y_max = std::cmp::min(consts::PROJECTION_PLANE_HEIGHT - 1, self.world.horizon() + wall_height / 2);
		// 		let step: f64 = consts::TEXTURE_HEIGHT as f64 / wall_height as f64;
				
		// 		if let raycast::TextureCode::Wall(code, texture_column, flipped) = slice.texture {
		// 			let texture = self.textures.get(code, texture_column, flipped);
		// 			let tex_pos: f64 = (y_min as f64 - *self.world.horizon() as f64 + wall_height as f64 / 2.0) * step;
		// 			parameters.push(ColumnRenderParameters::new(texture, step, wall_height, tex_pos, y_min, y_max))	
		// 		}
		// 	}

		// 	self.draw_to_buffer(buf, origin_x, origin_y, angle, sweep, &mut parameters);

		// 	angle += 1;
		// 	if angle >= trig::ANGLE_360 {
		// 		angle -= trig::ANGLE_360;
		// 	}
		}
	}

	fn find_horizontal_intersect(&self, origin_x: i32, origin_y: i32, direction: i32, scene: &Scene) -> Vec<Slice> {
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

		while scene.is_within_bounds(fp::div(x, consts::FP_TILE_SIZE).to_i32(), fp::div(y, consts::FP_TILE_SIZE).to_i32()) {
			if let Tile::Wall(texture, _) = scene.y_wall(fp::div(x, consts::FP_TILE_SIZE).to_i32(), fp::div(y, consts::FP_TILE_SIZE).to_i32()) {
				let slice = Slice::new(
					TextureCode::Wall(*texture, x.to_i32() & (consts::TILE_SIZE - 1), flipped),
					fp::mul(fp::sub(y, origin_y), trig::isin(direction)).abs(),					
				);
				slices.push(slice);
			}

			x = fp::add(x, step_x);
			y = fp::add(y, step_y);
		}

		slices
	}

	fn find_vertical_intersect(&self, origin_x: i32, origin_y: i32, direction: i32, scene: &Scene) -> Vec<Slice> {
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
		while scene.is_within_bounds(fp::div(x, consts::FP_TILE_SIZE).to_i32(), fp::div(y, consts::FP_TILE_SIZE).to_i32()) {
			if let Tile::Wall(texture, _) = scene.x_wall(fp::div(x, consts::FP_TILE_SIZE).to_i32(), fp::div(y, consts::FP_TILE_SIZE).to_i32()) {
				let slice = Slice::new(
					TextureCode::Wall(*texture, y.to_i32() & (consts::TILE_SIZE - 1), flipped),
					fp::mul(fp::sub(x, origin_x), trig::icos(direction)).abs()
				);

				slices.push(slice);
			}

			x = fp::add(x, step_x);
			y = fp::add(y, step_y);
		}

		slices
	}

	fn find_wall_intersections(&self, origin_x: i32, origin_y: i32, direction: i32, scene: &Scene) -> Vec<Slice> {
		let hslices = self.find_horizontal_intersect(origin_x, origin_y, direction, scene);
		let vslices = self.find_vertical_intersect(origin_x, origin_y, direction, scene);
		
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