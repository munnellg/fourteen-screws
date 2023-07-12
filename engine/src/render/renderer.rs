use base64::{Engine as _, engine::general_purpose};
use crate::{ Camera };
use crate::scene::{ Scene };
use crate::trig;
use crate::render::raycast;
use serde_json;
use shared::consts;
use shared::fp::{ ToFixedPoint };

macro_rules! colour_to_buf {
	($colour:expr, $buf:expr, $idx:expr) => {
		($buf[$idx + 0], $buf[$idx + 1], $buf[$idx + 2], $buf[$idx + 3]) = $colour.tuple();
	}
}

macro_rules! blend_colour_to_buf {
	($colour:expr, $buf:expr, $idx:expr) => {
		let blended = $colour.blend(&Colour::new($buf[$idx + 0], $buf[$idx + 1], $buf[$idx + 2], $buf[$idx + 3]));
		colour_to_buf!(blended, $buf, $idx);
	}
}

macro_rules! screen_idx {
	($x:expr, $y:expr) => {
		4 * ($x + $y * consts::PROJECTION_PLANE_WIDTH) as usize
	}
}

macro_rules! put_surface_pixel {
	($intersect:expr, $buf:expr, $idx:expr, $textures:expr) => {
		if $intersect.is_some() {
			let intersect = $intersect.unwrap();
			let texture = $textures.get(intersect.texture, intersect.x, false);
			let pixel = &texture[intersect.y as usize];
			colour_to_buf!(pixel, $buf, $idx);	
		}
	}
}

macro_rules! blend_surface_pixel {
	($intersect:expr, $pixel: expr, $textures:expr) => {
		if $intersect.is_some() {
			let intersect = $intersect.unwrap();
			let texture = $textures.get(intersect.texture, intersect.x, false);
			let pixel = &texture[intersect.y as usize];
			$pixel.blend(pixel)	
		} else {
			$pixel
		}
	}
}

pub struct Colour {
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub a: u8,
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

pub struct RenderParameters<'a> {
	texture: &'a [Colour],
	tex_idx: &'a [usize],
	y_min: i32,
	y_max: i32,
}

impl RenderParameters<'_> {
	pub fn new<'a>(texture: &'a [Colour], tex_idx: &'a [usize], y_min: i32, y_max: i32,) -> RenderParameters<'a> {
		RenderParameters { texture, tex_idx, y_min, y_max }
	}
}

pub struct TextureMap {
	texture_width: usize,
	texture_height: usize,
	texture_size: usize,
	textures: Vec<Colour>
}

impl TextureMap {
	pub fn new(texture_width: usize, texture_height: usize, channels: Vec<u8>) -> TextureMap {
		let texture_size = texture_width * texture_height;
		
		let mut textures = Vec::new();
		textures.reserve(channels.len() / 4);
		
		for i in (0..channels.len()).step_by(4) {
			textures.push(Colour::new(channels[i], channels[i + 1], channels[i + 2], channels[i + 3]));
		}

		TextureMap { texture_width, texture_height, texture_size, textures }
	}

	pub fn empty() -> TextureMap {
		TextureMap { texture_width: 0, texture_height: 0, texture_size: 0, textures: vec![] }
	}

	pub fn get(&self, code: u32, column: i32, flipped: bool) -> &[Colour] {
		let column = if flipped { self.texture_width - 1 - column as usize } else { column as usize };
		let head: usize = (self.texture_size * code as usize + column as usize * self.texture_width) as usize;
		let tail: usize = head + self.texture_height;
		&self.textures[head..tail]
	}
}

impl TryFrom<&serde_json::Value> for TextureMap {
	type Error = &'static str;

	fn try_from(json: &serde_json::Value) -> Result<Self, Self::Error> {
		let width    = json["width"].as_u64().unwrap() as usize;
		let height   = json["height"].as_u64().unwrap() as usize;
		let byte_str = json["textures"].as_str().unwrap();
		let bytes: Vec<u8> = general_purpose::STANDARD_NO_PAD.decode(byte_str).expect("failed to decode textures");
		Ok(TextureMap::new(width, height, bytes))
	}
}

pub struct Renderer {
	textures: TextureMap,
}

impl Renderer {
	pub fn new(textures: TextureMap) -> Renderer {
		Renderer{ textures }
	}

	pub fn render_column(&self, buf: &mut[u8], origin_x: i32, origin_y: i32, angle: i32, column: i32, camera: &Camera, scene: &Scene) {
			
		let parameters = self.intersect_to_render_params(origin_x, origin_y, angle, column, camera, scene);

		let y_min = parameters[0].y_min;
		let y_max = parameters[0].y_max;

		// draw ceiling
		for y in 0..y_min {
			let intersect = raycast::find_ceiling_intersection(origin_x, origin_y, angle, y, column, scene);
			put_surface_pixel!(intersect, buf, screen_idx!(column, y), self.textures);
		}

		// draw walls		
		for y in y_min..=y_max {
			let mut pixel = Colour::new(0, 0, 0, 0);
			
			let idx: usize = screen_idx!(column, y);
			
			for intersect in parameters.iter() {
				if y < intersect.y_min || y > intersect.y_max { break; } // terminate early if either we're above/below the tallest wall
				if pixel.a >= 255 { break; }                             // or the pixel is solid
				let tex_y = intersect.tex_idx[y as usize];
				pixel = pixel.blend(&intersect.texture[tex_y]);
			}
			
			// blend in the floor or ceiling through transparent areas if necessary
			if pixel.a < 255 {
				let intersect = if y > camera.horizon() {
					raycast::find_floor_intersection(origin_x, origin_y, angle, y, column, scene)
				} else {
					raycast::find_ceiling_intersection(origin_x, origin_y, angle, y, column, scene)
				};

				pixel = blend_surface_pixel!(intersect, pixel, self.textures);
			}

			blend_colour_to_buf!(pixel, buf, idx);
		}

		// draw floor
		for y in y_max..consts::PROJECTION_PLANE_HEIGHT {
			let intersect = raycast::find_floor_intersection(origin_x, origin_y, angle, y, column, scene);
			put_surface_pixel!(intersect, buf, screen_idx!(column, y), self.textures);
		}
	}

	pub fn render(&self, buf: &mut[u8], scene: &Scene, camera: &Camera) {
		self.render_background(buf);
		
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

			self.render_column(buf, origin_x, origin_y, angle, sweep, &camera, &scene);

			angle += 1;
			if angle >= trig::ANGLE_360 {
				angle -= trig::ANGLE_360;
			}
		}
	}

	fn render_background(&self, buf: &mut[u8]) {
		let ceiling = Colour::new(0x38, 0x38,  0x38, 0xFF);
		let floor   = Colour::new(0x70, 0x70,  0x70, 0xFF);

		for y in 0..consts::PROJECTION_PLANE_HORIZON {
			for x in 0..consts::PROJECTION_PLANE_WIDTH {
				colour_to_buf!(ceiling, buf, screen_idx!(x, y));
			}
		}

		for y in consts::PROJECTION_PLANE_HORIZON..consts::PROJECTION_PLANE_HEIGHT {
			for x in 0..consts::PROJECTION_PLANE_WIDTH {
				colour_to_buf!(floor, buf, screen_idx!(x, y));
			}
		}
	}

	fn intersect_to_render_params(&self, origin_x: i32, origin_y: i32, angle: i32, column: i32, camera: &Camera, scene: &Scene) -> Vec<RenderParameters> {
		let intersects = raycast::find_wall_intersections(origin_x, origin_y, angle, column, scene);

		// for each intersection, get a reference to its texture and figure out how
		// it should be drawn
		return intersects.iter().map(|intersect| {
			let dist        = intersect.dist;
			let wall_height = trig::wall_height(dist);
			let mid_height  = wall_height >> 1;
			let y_min       = std::cmp::max(0, camera.horizon() - mid_height);
			let y_max       = std::cmp::min(consts::PROJECTION_PLANE_HEIGHT - 1, camera.horizon() + mid_height);
			let tex_idx     = trig::wall_texture_index(wall_height);
			let texture     = self.textures.get(intersect.texture, intersect.texture_column, intersect.reverse);
			RenderParameters::new(texture, tex_idx, y_min, y_max)
		}).collect();
	}
}

impl TryFrom<&serde_json::Value> for Renderer {
	type Error = &'static str;

	fn try_from(json: &serde_json::Value) -> Result<Self, Self::Error> {
		let textures  = TextureMap::try_from(&json["texture_map"]).ok().unwrap();
		Ok(Renderer::new(textures))
	}
}