use crate::scene::{ Tile, Scene };
use crate::trig;
use serde_json;
use shared::consts;
use shared::fp;
use shared::fp::{ ToFixedPoint, FromFixedPoint };

struct Colour {
	r: u8,
	g: u8,
	b: u8,
	a: u8,
}

impl Colour {
	pub fn new(r: u8, g: u8, b: u8, a: u8) -> Colour {
		Colour { r, g, b, a }
	}

	pub fn blend(self, other: &Colour) -> Colour {
		let (r, g, b, a) = Colour::blend_colours(self.r, self.g, self.b, self.a, other.r, other.g, other.b, other.a);
		Colour { r, g, b, a }
	}

	pub fn tuple(&self) -> (u8, u8, u8, u8) {
		(self.r, self.g, self.b, self.a)
	}

	fn alpha_blend(c1: f64, a1: f64, c2: f64, a2: f64, ao: f64) -> f64 {
		(c1 * a1 + c2 * a2 * (1.0 - a1)) / ao
	}

	fn blend_colours(r1: u8, g1: u8, b1: u8, a1: u8, r2:u8, g2:u8, b2:u8, a2:u8) -> (u8, u8, u8, u8) {
		let fa1 = a1 as f64 / 255.0;
		let fa2 = a2 as f64 / 255.0;
		let fao = Colour::alpha_blend(1.0, fa1, 1.0, fa2, 1.0);

		let r = Colour::alpha_blend(r1 as f64, fa1, r2 as f64, fa2, fao) as u8;
		let g = Colour::alpha_blend(g1 as f64, fa1, g2 as f64, fa2, fao) as u8;
		let b = Colour::alpha_blend(b1 as f64, fa1, b2 as f64, fa2, fao) as u8;
		let a = (255.0 * fao) as u8;

		(r, g, b, a)
	}
}

struct Intersection {
	x: i32,
	y: i32,
	dist: i32,
	texture: u32,
	reverse: bool,
}

impl Intersection {
	pub fn new(x: i32, y: i32, dist:i32, texture: u32, reverse: bool) -> Intersection {
		Intersection { x, y, dist, texture, reverse }
	}
}

pub struct Camera {
	x: i32,
	y: i32,
	angle: i32,
	horizon: i32,
}

impl Camera {
	pub fn new(x: i32, y: i32, angle: i32, horizon: i32) -> Camera {
		Camera { x, y, angle, horizon }
	}

	pub fn default() -> Camera {
		Camera::new(0, 0, 0, consts::PROJECTION_PLANE_HORIZON)
	}

	pub fn rotate(&mut self, angle: i32) {
		self.angle += angle;
		while self.angle >= trig::ANGLE_360 { self.angle -= trig::ANGLE_360; }
		while self.angle < trig::ANGLE_0    { self.angle += trig::ANGLE_360; }
	}

	pub fn pitch(&mut self, distance: i32) {
		self.horizon += distance;
		if self.horizon < 20  { self.horizon =  20; }
		if self.horizon > 180 { self.horizon = 180; }
	}

	pub fn move_to(&mut self, x: i32, y: i32) {
		self.set_x(x);
		self.set_y(y);
	}

	pub fn x(&self) -> i32 {
		self.x
	}

	pub fn set_x(&mut self, x: i32) {
		self.x = x;
	}

	pub fn y(&self) -> i32 {
		self.y
	}

	pub fn set_y(&mut self, y: i32) {
		self.y = y;
	}

	pub fn angle(&self) -> i32 {
		self.angle
	}

	pub fn horizon(&self) -> i32 {
		self.horizon
	}

	pub fn from_json(json: &serde_json::Value) -> Result<Camera, &'static str> {
		let x = json["x"].as_i64().unwrap() as i32;
		let y = json["y"].as_i64().unwrap() as i32;
		let a = json["angle"].as_i64().unwrap() as i32;
		let h = json["horizon"].as_i64().unwrap() as i32;
		Ok(Camera::new(x, y, a, h))
	}
}

struct RayCaster {}

impl RayCaster {
	fn new() -> RayCaster {
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
			
			match scene.y_wall(grid_x, grid_y) {
				Tile::Wall(wall) => {
					let world_x  = x.to_i32() & (consts::TILE_SIZE - 1);
					let world_y  = y.to_i32() & (consts::TILE_SIZE - 1);
					let distance = fp::mul(fp::sub(y, origin_y), trig::isin(direction)).abs();
					let texture  = wall.texture;
					let intersection = Intersection::new(world_x, world_y, distance, texture, flipped);
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
					let world_x  = x.to_i32() & (consts::TILE_SIZE - 1);
					let world_y  = y.to_i32() & (consts::TILE_SIZE - 1);
					let distance = fp::mul(fp::sub(x, origin_x), trig::icos(direction)).abs();
					let texture  = wall.texture;
					let intersection = Intersection::new(world_x, world_y, distance, texture, flipped);
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

	fn find_wall_intersections(&self, origin_x: i32, origin_y: i32, direction: i32, scene: &Scene) -> Vec<Intersection> {
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

	pub fn get(&self, code: u32, column: i32, flipped: bool) -> &[u8] {
		let column = if flipped { self.texture_width - 1 - column as usize } else { column as usize };
		let pos: usize = (self.texture_size * code as usize + column as usize * self.texture_width) * 4 as usize;
		&self.textures[pos..pos + self.texture_size]
	}
}

struct RenderParameters<'a> {
	texture: &'a [u8],
	step: f64,
	wall_height: i32,
	tex_pos: f64,
	y_min: i32,
	y_max: i32,
}

impl RenderParameters<'_> {
	pub fn new(texture: &[u8], step: f64, wall_height: i32, tex_pos: f64, y_min: i32, y_max: i32,) -> RenderParameters {
		RenderParameters { texture, step, wall_height, tex_pos, y_min, y_max }
	}
}

pub struct Renderer {
	raycaster: RayCaster,
	textures: TextureMap,
}

impl Renderer {
	pub fn new(raycaster: RayCaster, textures: TextureMap) -> Renderer {
		Renderer{ raycaster, textures }
	}

	pub fn render_column(&self, buf: &mut[u8], column: i32, parameters: &Vec<RenderParameters>) {
		let y_min = parameters[0].y_min;
		let y_max = parameters[0].y_max;

		for y in y_min..=y_max {
			let pixel = Colour::new(r, g, b, a);
			
			let idx: usize = 4 * (column + y * consts::PROJECTION_PLANE_WIDTH) as usize;

			for intersect in parameters.iter_mut() {
				if y < intersect.y_min || y > intersect.y_max { break; }
				let tex_y = (intersect.tex_pos.clamp(0.0, 63.0) as usize) * 4;
				intersect.step();
				if a >= 255 { continue; }
				(r, g, b, a) = blend_colours(r, g, b, a, intersect.texture[tex_y + 0], intersect.texture[tex_y + 1], intersect.texture[tex_y + 2], intersect.texture[tex_y + 3]);
			}

			(buf[idx + 0], buf[idx + 1], buf[idx + 2], buf[idx + 3]) = blend_colours(r, g, b, a, buf[idx + 0], buf[idx + 1], buf[idx + 2], buf[idx + 3]);
		}
	}

	pub fn render(&self, buf: &mut[u8], scene: &Scene, camera: &Camera) {
		// angle is the direction camera is facing
		// need to start out sweep 30 degrees to the left
		let mut angle = if camera.angle() < trig::ANGLE_30 {
			camera.angle() - trig::ANGLE_30 + trig::ANGLE_360
		} else {
			camera.angle() - trig::ANGLE_30
		};

		// ray casting uses fixed point notation, so convert camera coordinates to fixed point
		let origin_x = camera.x().to_fp();
		let origin_y = camera.y().to_fp();

		// sweep of the rays will be through 60 degrees
		for sweep in 0..trig::ANGLE_60 {
			let intersects = self.raycaster.find_wall_intersections(origin_x, origin_y, angle, scene);
			if intersects.len() <= 0 { continue; }
			let mut parameters: Vec<RenderParameters> = Vec::new();
			parameters.reserve(intersects.len());

			// for each slice, get a reference to its texture and figure out how
			// it should be drawn
			let parameters = intersects.iter().map(|intersect| {
				let dist = fp::div(intersect.dist, trig::fisheye_correction(sweep)).to_i32();
				let wall_height: i32 = trig::wall_height(dist);
				let y_min = std::cmp::max(0, camera.horizon() - wall_height / 2);
				let y_max = std::cmp::min(consts::PROJECTION_PLANE_HEIGHT - 1, camera.horizon() + wall_height / 2);
				let step: f64 = consts::TEXTURE_HEIGHT as f64 / wall_height as f64;
				let texture = self.textures.get(intersect.texture, 0, intersect.reverse);
				let tex_pos: f64 = (y_min as f64 - camera.horizon() as f64 + wall_height as f64 / 2.0) * step;
				RenderParameters::new(texture, step, wall_height, tex_pos, y_min, y_max)
			}).collect();

			self.render_column(buf, sweep, &mut parameters);

			angle += 1;
			if angle >= trig::ANGLE_360 {
				angle -= trig::ANGLE_360;
			}
		}
	}

	// pub fn from_json(_json: &serde_json::Value) -> Result<Renderer, &'static str> {
	// 	Ok(Renderer::new())
	// }
}

// struct Colour {
// 	r: u8,
// 	g: u8,
// 	b: u8,
// 	a: u8,
// }

// impl Colour {
// 	pub fn new(r: u8, g: u8, b: u8, a: u8) -> Colour {
// 		Colour { r, g, b, a }
// 	}

// 	pub fn blend(self, other: &Colour) -> Colour {
// 		let (r, g, b, a) = Colour::blend_colours(self.r, self.g, self.b, self.a, other.r, other.g, other.b, other.a);
// 		Colour { r, g, b, a }
// 	}

// 	pub fn tuple(&self) -> (u8, u8, u8, u8) {
// 		(self.r, self.g, self.b, self.a)
// 	}

// 	fn alpha_blend(c1: f64, a1: f64, c2: f64, a2: f64, ao: f64) -> f64 {
// 		(c1 * a1 + c2 * a2 * (1.0 - a1)) / ao
// 	}

// 	fn blend_colours(r1: u8, g1: u8, b1: u8, a1: u8, r2:u8, g2:u8, b2:u8, a2:u8) -> (u8, u8, u8, u8) {
// 		let fa1 = a1 as f64 / 255.0;
// 		let fa2 = a2 as f64 / 255.0;
// 		let fao = Colour::alpha_blend(1.0, fa1, 1.0, fa2, 1.0);

// 		let r = Colour::alpha_blend(r1 as f64, fa1, r2 as f64, fa2, fao) as u8;
// 		let g = Colour::alpha_blend(g1 as f64, fa1, g2 as f64, fa2, fao) as u8;
// 		let b = Colour::alpha_blend(b1 as f64, fa1, b2 as f64, fa2, fao) as u8;
// 		let a = (255.0 * fao) as u8;

// 		(r, g, b, a)
// 	}
// }

// struct TextureFloorRenderer {
// 	default_colour: Colour,
// }

// impl TextureFloorRenderer {
// 	pub fn new() -> TextureFloorRenderer {
// 		TextureFloorRenderer { default_colour: Colour::new(0x70, 0x70, 0x70, 0xFF) }
// 	}

// 	pub fn render(&self, column: i32, y_min: i32, y_max: i32, camera: &Camera, scene: &Scene) {
// 		for y in y_min..y_max {
// 			let floor = self.find_floor_intersection(y, column, camera, scene);
// 			let idx: usize = 4 * (column + y * consts::PROJECTION_PLANE_WIDTH) as usize;

// 			if let RayCastResult::Surface(intersection) = floor {
// 				let texture = self.textures.get(code, x, false);
// 				let tex_y = (y * 4) as usize;

// 				(buf[idx + 0], buf[idx + 1], buf[idx + 2], buf[idx + 3]) = (texture[tex_y + 0], texture[tex_y + 1], texture[tex_y + 2], texture[tex_y + 3]);
// 			} else {
// 				(buf[idx + 0], buf[idx + 1], buf[idx + 2], buf[idx + 3]) = self.default_colour.tuple();
// 			}
// 		}
// 	}

// 	fn find_floor_intersection(&self, row: i32, column: i32, camera: &Camera, scene: &Scene) -> RayCastResult {
// 		// convert to fixed point
// 		let player_height = consts::PLAYER_HEIGHT.to_fp(); 
// 		let pp_distance   = consts::DISTANCE_TO_PROJECTION_PLANE.to_fp();

// 		// adding 1 to the row exactly on the horizon avoids a division by one error
// 		// doubles up the texture at the vanishing point, but probably fine
// 		let row = if row == camera.horizon { (row + 1).to_fp() } else { row.to_fp() };

// 		let ratio = fp::div(player_height, fp::sub(row, camera.horizon.to_fp()));

// 		let diagonal_distance = fp::mul(fp::floor(fp::mul(pp_distance, ratio)), trig::fisheye_correction(column));

// 		let x_end = fp::floor(fp::mul(diagonal_distance, trig::cos(camera.angle)));
// 		let y_end = fp::floor(fp::mul(diagonal_distance, trig::sin(camera.angle)));

// 		let x_end = fp::add(camera.x(), x_end);
// 		let y_end = fp::add(camera.y(), y_end);
		
// 		let x = fp::floor(fp::div(x_end, consts::FP_TILE_SIZE)).to_i32();
// 		let y = fp::floor(fp::div(y_end, consts::FP_TILE_SIZE)).to_i32();
		
// 		if !scene.is_within_bounds(x, y) {
// 			return RayCastResult::OutOfBounds;
// 		}

// 		let texture_col = x_end.to_i32() & (consts::TILE_SIZE - 1);
// 		let texture_row = y_end.to_i32() & (consts::TILE_SIZE - 1);

// 		let intersection = Intersection::new(texture_col, texture_row, diagonal_distance, 42, false);
// 		RayCastResult::Surface(intersection)
// 	}
// }

// struct TextureCeilingRenderer {
// 	default_colour: Colour;
// }

// impl TextureCeilingRenderer {
// 	pub fn new() -> TextureFloorRenderer {
// 		TextureFloorRenderer { Colour::new(0x50, 0x50, 0x50, 0xFF) }
// 	}

// 	pub fn render(&self, column: i32, y_min: i32, y_max: i32, camera: &Camera, scene: &Scene) {
// 		for y in y_min..y_max {
// 			let ceiling = self.find_ceiling_intersection(y, column, camera, scene);
// 			let idx: usize = 4 * (column + y * consts::PROJECTION_PLANE_WIDTH) as usize;

// 			if let RayCastResult::Surface(intersection) = floor {
// 				let texture = self.textures.get(code, x, false);
// 				let tex_y = (y * 4) as usize;

// 				(buf[idx + 0], buf[idx + 1], buf[idx + 2], buf[idx + 3]) = (texture[tex_y + 0], texture[tex_y + 1], texture[tex_y + 2], texture[tex_y + 3]);
// 			} else {
// 				(buf[idx + 0], buf[idx + 1], buf[idx + 2], buf[idx + 3]) = self.default_colour.tuple();
// 			}
// 		}
// 	}

// 	pub fn find_ceiling_intersection(&self, origin_x: i32, origin_y: i32, direction: i32, row: i32, column: i32) -> TextureCode {
// 		// convert to fixed point
// 		let player_height = consts::PLAYER_HEIGHT.to_fp(); 
// 		let pp_distance   = consts::DISTANCE_TO_PROJECTION_PLANE.to_fp();
// 		let wall_height   = consts::WALL_HEIGHT.to_fp();

// 		// adding 1 to the row exactly on the horizon avoids a division by one error
// 		// doubles up the texture at the vanishing point, but probably fine
// 		let row = if row == self.horizon { (row + 1).to_fp() } else { row.to_fp() };

// 		let ratio = fp::div(fp::sub(wall_height, player_height), fp::sub(self.fp_horizon, row));

// 		let diagonal_distance = fp::mul(fp::floor(fp::mul(pp_distance, ratio)), trig::fisheye_correction(column));

// 		let x_end = fp::floor(fp::mul(diagonal_distance, trig::cos(direction)));
// 		let y_end = fp::floor(fp::mul(diagonal_distance, trig::sin(direction)));

// 		let x_end = fp::add(origin_x, x_end);
// 		let y_end = fp::add(origin_y, y_end);
		
// 		let x = fp::floor(fp::div(x_end, consts::FP_TILE_SIZE)).to_i32();
// 		let y = fp::floor(fp::div(y_end, consts::FP_TILE_SIZE)).to_i32();
		
// 		if !self.is_within_bounds(x, y) {
// 			return TextureCode::None;
// 		}

// 		TextureCode::Ceiling(23, x_end.to_i32() & (consts::TILE_SIZE - 1), y_end.to_i32() & (consts::TILE_SIZE - 1))
// 	}
// }

// struct RenderConfig {

// }

// impl RenderConfig {
// 	pub fn new() -> RenderConfig {
// 		RenderConfig {}
// 	}

// 	pub fn default() -> RenderConfig {
// 		RenderConfig {}
// 	}
// }

// struct RaycastRenderer {
// 	floor_renderer: TextureFloorRenderer
// }

// impl RaycastRenderer {
// 	pub fn new(config: &RenderConfig) -> RaycastRenderer {
// 		Renderer { floor_renderer : TextureFloorRenderer::new() }
// 	}

// 	fn draw_wall_column(&self, buf: &mut[u8], origin_x: i32, origin_y: i32, direction: i32, column: i32, parameters: &mut Vec<ColumnRenderParameters>) {
// 		let y_min = parameters[0].y_min;
// 		let y_max = parameters[0].y_max;

// 		for y in y_min..=y_max {
// 			let mut r: u8 = 0;
// 			let mut g: u8 = 0;
// 			let mut b: u8 = 0;
// 			let mut a: u8 = 0;
			
// 			let idx: usize = 4 * (column + y * consts::PROJECTION_PLANE_WIDTH) as usize;

// 			for slice in parameters.iter_mut() {
// 				if y < slice.y_min || y > slice.y_max { break; }
// 				let tex_y = (slice.tex_pos.clamp(0.0, 63.0) as usize) * 4;
// 				slice.step();
// 				if a >= 255 { continue; }
// 				(r, g, b, a) = blend_colours(r, g, b, a, slice.texture[tex_y + 0], slice.texture[tex_y + 1], slice.texture[tex_y + 2], slice.texture[tex_y + 3]);
// 			}

// 			if a < 255 {
// 				if y >= *self.world.horizon() {
// 					let floor = self.world.find_floor_intersection(origin_x, origin_y, direction, y, column);

// 					if let raycast::TextureCode::Floor(code, x, y) = floor {
// 						let texture = self.textures.get(code, x, false);
// 						let tex_y = (y * 4) as usize;
// 						(r, g, b, a) = blend_colours(r, g, b, a, texture[tex_y + 0], texture[tex_y + 1], texture[tex_y + 2], texture[tex_y + 3]);
// 					} else {
// 						(r, g, b, a) = blend_colours(r, g, b, a, 0x70, 0x70, 0x70, 0xFF);
// 					}
// 				} else {
// 					let ceiling = self.world.find_ceiling_intersection(origin_x, origin_y, direction, y, column);

// 					if let raycast::TextureCode::Ceiling(code, x, y) = ceiling {
// 						let texture = self.textures.get(code, x, false);
// 						let tex_y = (y * 4) as usize;
// 						(r, g, b, a) = blend_colours(r, g, b, a, texture[tex_y + 0], texture[tex_y + 1], texture[tex_y + 2], texture[tex_y + 3]);
// 					} else {
// 						(r, g, b, a) = blend_colours(r, g, b, a, 0x70, 0x70, 0x70, 0xFF);
// 					}
// 				}
// 			}

// 			(buf[idx + 0], buf[idx + 1], buf[idx + 2], buf[idx + 3]) = blend_colours(r, g, b, a, buf[idx + 0], buf[idx + 1], buf[idx + 2], buf[idx + 3]);
// 		}

// 		// texture the ceiling
// 		for y in 0..(y_min) {
// 			let ceiling = self.world.find_ceiling_intersection(origin_x, origin_y, direction, y, column);
// 			let idx: usize = 4 * (column + y * consts::PROJECTION_PLANE_WIDTH) as usize;

// 			if let raycast::TextureCode::Ceiling(code, x, y) = ceiling {
// 				let texture = self.textures.get(code, x, false);
// 				let tex_y = (y * 4) as usize;

// 				(buf[idx + 0], buf[idx + 1], buf[idx + 2], buf[idx + 3]) = (texture[tex_y + 0], texture[tex_y + 1], texture[tex_y + 2], texture[tex_y + 3]);
// 			} else {
// 				(buf[idx + 0], buf[idx + 1], buf[idx + 2], buf[idx + 3]) = (0x70, 0x70, 0x70, 0xFF);
// 			}
// 		}

// 		// texture the floor
// 		self.floor_renderer.render(buf, column, y_max + 1, consts::PROJECTION_PLANE_HEIGHT, camera, scene);
// 	}

// 	pub fn render(&self, buf: &mut[u8], scene: &Scene, camera: &Camera) {

// 		// theta is the direction player is facing
// 		// need to start out sweep 30 degrees to the left
// 		let mut angle = if camera.angle < trig::ANGLE_30 {
// 			camera.angle - trig::ANGLE_30 + trig::ANGLE_360
// 		} else {
// 			camera.angle - trig::ANGLE_30
// 		};

// 		// ray casting uses fixed point notation, so convert player coordinates to fixed point
// 		let origin_x = camera.x.to_fp();
// 		let origin_y = camera.y.to_fp();

// 		// sweep of the rays will be through 60 degrees
// 		for sweep in 0..trig::ANGLE_60 {
// 			let slices = self.find_wall_intersections(origin_x, origin_y, angle, scene);
// 		// 	if slices.len() <= 0 { continue; }
// 		// 	let mut parameters: Vec<ColumnRenderParameters> = Vec::new();
// 		// 	parameters.reserve(slices.len());

// 		// 	// for each slice, get a reference to its texture and figure out how
// 		// 	// it should be drawn
// 		// 	for slice in slices {
// 		// 		let dist = fp::div(slice.distance, trig::fisheye_correction(sweep)).to_i32();
// 		// 		let wall_height: i32 = trig::wall_height(dist);
// 		// 		let y_min = std::cmp::max(0, self.world.horizon() - wall_height / 2);
// 		// 		let y_max = std::cmp::min(consts::PROJECTION_PLANE_HEIGHT - 1, self.world.horizon() + wall_height / 2);
// 		// 		let step: f64 = consts::TEXTURE_HEIGHT as f64 / wall_height as f64;
				
// 		// 		if let raycast::TextureCode::Wall(code, texture_column, flipped) = slice.texture {
// 		// 			let texture = self.textures.get(code, texture_column, flipped);
// 		// 			let tex_pos: f64 = (y_min as f64 - *self.world.horizon() as f64 + wall_height as f64 / 2.0) * step;
// 		// 			parameters.push(ColumnRenderParameters::new(texture, step, wall_height, tex_pos, y_min, y_max))	
// 		// 		}
// 		// 	}

// 		// 	self.draw_to_buffer(buf, origin_x, origin_y, angle, sweep, &mut parameters);

// 		// 	angle += 1;
// 		// 	if angle >= trig::ANGLE_360 {
// 		// 		angle -= trig::ANGLE_360;
// 		// 	}
// 		}
// 	}

// 	fn find_horizontal_intersect(&self, origin_x: i32, origin_y: i32, direction: i32, scene: &Scene) -> Vec<Intersection> {
// 		let step_x: i32; // distance to next vertical intersect
// 		let step_y: i32; // distance to next horizontal intersect
// 		let mut x: i32;  // x coordinate of current ray intersect
// 		let mut y: i32;  // y coordinate of current ray intersect
// 		let flipped: bool;

// 		let mut slices = Vec::new();

// 		// determine if looking up or down and find horizontal intersection
// 		if direction > trig::ANGLE_0 && direction < trig::ANGLE_180 { // looking down
// 			step_x = trig::xstep(direction);
// 			step_y = consts::FP_TILE_SIZE;

// 			y = ((origin_y.to_i32() / consts::TILE_SIZE) * consts::TILE_SIZE + consts::TILE_SIZE).to_fp();
// 			x = fp::add(origin_x, fp::mul(fp::sub(y, origin_y), trig::itan(direction)));
// 			flipped = true;
// 		} else {                     // looking up
// 			step_x = trig::xstep(direction);
// 			step_y = -consts::FP_TILE_SIZE;

// 			y = ((origin_y.to_i32() / consts::TILE_SIZE) * consts::TILE_SIZE).to_fp();
// 			x = fp::add(origin_x, fp::mul(fp::sub(y, origin_y), trig::itan(direction)));
// 			flipped = false;
// 		}

// 		if direction == trig::ANGLE_0 || direction == trig::ANGLE_180 {
// 			return slices;
// 		}

// 		// Cast x axis intersect rays, build up xSlice

// 		while scene.is_within_bounds(fp::div(x, consts::FP_TILE_SIZE).to_i32(), fp::div(y, consts::FP_TILE_SIZE).to_i32()) {
// 			if let Tile::Wall(texture, _) = scene.y_wall(fp::div(x, consts::FP_TILE_SIZE).to_i32(), fp::div(y, consts::FP_TILE_SIZE).to_i32()) {
// 				let slice = Slice::new(
// 					TextureCode::Wall(*texture, x.to_i32() & (consts::TILE_SIZE - 1), flipped),
// 					fp::mul(fp::sub(y, origin_y), trig::isin(direction)).abs(),					
// 				);
// 				slices.push(slice);
// 			}

// 			x = fp::add(x, step_x);
// 			y = fp::add(y, step_y);
// 		}

// 		slices
// 	}

// 	fn find_vertical_intersect(&self, origin_x: i32, origin_y: i32, direction: i32, scene: &Scene) -> Vec<Intersection> {
// 		let step_x: i32; // distance to next vertical intersect
// 		let step_y: i32; // distance to next horizontal intersect
// 		let mut x: i32;  // x coordinate of current ray intersect
// 		let mut y: i32;  // y coordinate of current ray intersect
// 		let flipped: bool;

// 		let mut slices = Vec::new();

// 		// determine if looking left or right and find vertical intersection
// 		if direction <= trig::ANGLE_90 || direction > trig::ANGLE_270 { // looking right
// 			step_x = consts::FP_TILE_SIZE;
// 			step_y = trig::ystep(direction);
			
// 			x = ((origin_x.to_i32() / consts::TILE_SIZE) * consts::TILE_SIZE + consts::TILE_SIZE).to_fp();
// 			y = fp::add(origin_y, fp::mul(fp::sub(x, origin_x), trig::tan(direction)));
			
// 			flipped = false;
// 		} else {
// 			step_x = -consts::FP_TILE_SIZE;
// 			step_y = trig::ystep(direction);
			
// 			x = (((origin_x.to_i32() / consts::TILE_SIZE) * consts::TILE_SIZE)).to_fp();
// 			y = fp::add(origin_y, fp::mul(fp::sub(x, origin_x), trig::tan(direction)));
			
// 			flipped = true;
// 		};

// 		if direction == trig::ANGLE_90 || direction == trig::ANGLE_270 {
// 			return slices;
// 		}

// 		// Cast y axis intersect rays, build up ySlice
// 		while scene.is_within_bounds(fp::div(x, consts::FP_TILE_SIZE).to_i32(), fp::div(y, consts::FP_TILE_SIZE).to_i32()) {
// 			if let Tile::Wall(texture, _) = scene.x_wall(fp::div(x, consts::FP_TILE_SIZE).to_i32(), fp::div(y, consts::FP_TILE_SIZE).to_i32()) {
// 				let slice = Slice::new(
// 					TextureCode::Wall(*texture, y.to_i32() & (consts::TILE_SIZE - 1), flipped),
// 					fp::mul(fp::sub(x, origin_x), trig::icos(direction)).abs()
// 				);

// 				slices.push(slice);
// 			}

// 			x = fp::add(x, step_x);
// 			y = fp::add(y, step_y);
// 		}

// 		slices
// 	}

// 	fn find_wall_intersections(&self, origin_x: i32, origin_y: i32, direction: i32, scene: &Scene) -> Vec<Slice> {
// 		let hslices = self.find_horizontal_intersect(origin_x, origin_y, direction, scene);
// 		let vslices = self.find_vertical_intersect(origin_x, origin_y, direction, scene);
		
// 		let mut slices = Vec::new();
// 		slices.reserve(hslices.len() + vslices.len());

// 		let mut i = 0;
// 		let mut j = 0;

// 		while i < hslices.len() && j < vslices.len() {
// 			if hslices[i].distance < vslices[j].distance {
// 				slices.push(hslices[i]);
// 				i += 1;
// 			} else {
// 				slices.push(vslices[j]);
// 				j += 1;
// 			}
// 		}

// 		while i < hslices.len() {			
// 			slices.push(hslices[i]);
// 			i += 1;
// 		}

// 		while j < vslices.len() {
// 			slices.push(vslices[j]);
// 			j += 1;
// 		}

// 		slices
// 	}

// 	pub fn find_ceiling_intersection(&self, origin_x: i32, origin_y: i32, direction: i32, row: i32, column: i32) -> TextureCode {
// 		// convert to fixed point
// 		let player_height = consts::PLAYER_HEIGHT.to_fp(); 
// 		let pp_distance   = consts::DISTANCE_TO_PROJECTION_PLANE.to_fp();
// 		let wall_height   = consts::WALL_HEIGHT.to_fp();

// 		// adding 1 to the row exactly on the horizon avoids a division by one error
// 		// doubles up the texture at the vanishing point, but probably fine
// 		let row = if row == self.horizon { (row + 1).to_fp() } else { row.to_fp() };

// 		let ratio = fp::div(fp::sub(wall_height, player_height), fp::sub(self.fp_horizon, row));

// 		let diagonal_distance = fp::mul(fp::floor(fp::mul(pp_distance, ratio)), trig::fisheye_correction(column));

// 		let x_end = fp::floor(fp::mul(diagonal_distance, trig::cos(direction)));
// 		let y_end = fp::floor(fp::mul(diagonal_distance, trig::sin(direction)));

// 		let x_end = fp::add(origin_x, x_end);
// 		let y_end = fp::add(origin_y, y_end);
		
// 		let x = fp::floor(fp::div(x_end, consts::FP_TILE_SIZE)).to_i32();
// 		let y = fp::floor(fp::div(y_end, consts::FP_TILE_SIZE)).to_i32();
		
// 		if !self.is_within_bounds(x, y) {
// 			return TextureCode::None;
// 		}

// 		TextureCode::Ceiling(23, x_end.to_i32() & (consts::TILE_SIZE - 1), y_end.to_i32() & (consts::TILE_SIZE - 1))
// 	}
// }