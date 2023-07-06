use base64::{Engine as _, engine::general_purpose};
use crate::{ Camera, RayCaster };
use crate::scene::{ Scene };
use crate::trig;
use serde_json;
use shared::consts;
use shared::fp;
use shared::fp::{ ToFixedPoint, FromFixedPoint };

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

	pub fn from_json(json: &serde_json::Value) -> Result<TextureMap, &'static str> {
		let width    = json["width"].as_u64().unwrap() as usize;
		let height   = json["height"].as_u64().unwrap() as usize;
		let byte_str = json["textures"].as_str().unwrap();
		let bytes: Vec<u8> = general_purpose::STANDARD_NO_PAD.decode(byte_str).expect("failed to decode textures");
		Ok(TextureMap::new(width, height, bytes))
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

	pub fn render_column(&self, buf: &mut[u8], column: i32, parameters: Vec<RenderParameters>) {
		let y_min = parameters[0].y_min;
		let y_max = parameters[0].y_max;

		for y in y_min..=y_max {
			let mut pixel = Colour::new(0, 0, 0, 0);
			
			let idx: usize = 4 * (column + y * consts::PROJECTION_PLANE_WIDTH) as usize;
			
			for intersect in parameters.iter() {
				if y < intersect.y_min || y > intersect.y_max { break; } // terminate early if either we're above/below the tallest wall
				if pixel.a >= 255 { break; }                             // or the pixel is solid
				let tex_y = intersect.tex_idx[y as usize];
				pixel = pixel.blend(&intersect.texture[tex_y]);
			}

			pixel = pixel.blend(&Colour::new(buf[idx + 0], buf[idx + 1], buf[idx + 2], buf[idx + 3]));
			(buf[idx + 0], buf[idx + 1], buf[idx + 2], buf[idx + 3]) = pixel.tuple();
		}
	}

	fn draw_background(&self, buf: &mut[u8]) {

		for y in 0..consts::PROJECTION_PLANE_HORIZON / 2 {
			for x in 0..consts::PROJECTION_PLANE_WIDTH {
				let idx: usize = 4 * (x + y * consts::PROJECTION_PLANE_WIDTH) as usize;
				buf[idx + 0] = 0x38;
				buf[idx + 1] = 0x38;
				buf[idx + 2] = 0x38;
				buf[idx + 3] = 0xFF; // alpha channel				
			}
		}

		for y in consts::PROJECTION_PLANE_HORIZON..consts::PROJECTION_PLANE_HEIGHT {
			for x in 0..consts::PROJECTION_PLANE_WIDTH {
				let idx: usize = 4 * (x + y * consts::PROJECTION_PLANE_WIDTH) as usize;
				buf[idx + 0] = 0x70;
				buf[idx + 1] = 0x70;
				buf[idx + 2] = 0x70;
				buf[idx + 3] = 0xFF; // alpha channel
			}
		}
	}

	pub fn render(&self, buf: &mut[u8], scene: &Scene, camera: &Camera) {
		self.draw_background(buf);
		
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
			
			// for each intersection, get a reference to its texture and figure out how
			// it should be drawn
			let parameters = intersects.iter().map(|intersect| {
				let dist        = fp::div(intersect.dist, trig::fisheye_correction(sweep)).to_i32();
				let wall_height = trig::wall_height(dist);
				let mid_height  = wall_height >> 1;
				let y_min       = std::cmp::max(0, camera.horizon() - mid_height);
				let y_max       = std::cmp::min(consts::PROJECTION_PLANE_HEIGHT - 1, camera.horizon() + mid_height);
				let tex_idx     = trig::wall_texture_index(wall_height);
				let texture     = self.textures.get(intersect.texture, intersect.texture_column, intersect.reverse);
				RenderParameters::new(texture, tex_idx, y_min, y_max)
			}).collect();

			self.render_column(buf, sweep, parameters);

			angle += 1;
			if angle >= trig::ANGLE_360 {
				angle -= trig::ANGLE_360;
			}
		}
	}

	pub fn from_json(json: &serde_json::Value) -> Result<Renderer, &'static str> {
		let raycaster = RayCaster::new();
		let textures  = TextureMap::from_json(&json["texture_map"]).ok().unwrap();
		Ok(Renderer::new(raycaster, textures))
	}
}
