use crate::trig;
use shared::consts;

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
}

impl TryFrom<&serde_json::Value> for Camera {
	type Error = &'static str;

	fn try_from(json: &serde_json::Value) -> Result<Self, Self::Error> {
		let x = json["x"].as_i64().unwrap() as i32;
		let y = json["y"].as_i64().unwrap() as i32;
		let a = json["angle"].as_i64().unwrap() as i32;
		let h = json["horizon"].as_i64().unwrap() as i32;
		Ok(Camera::new(x, y, a, h))
	}
}