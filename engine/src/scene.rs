use serde_json;

pub struct TextureTile {
	pub texture: u32,
	pub passable: bool,
}

pub enum Tile {
	OutOfBounds,
	Empty,
	Wall(TextureTile),
}

pub struct Scene {
	width: i32,
	height: i32,
	y_walls: Vec<Tile>,
	x_walls: Vec<Tile>,
	floor: Vec<Tile>,
	ceiling: Vec<Tile>,
}

impl Scene {
	pub fn new(width: i32, height: i32, y_walls: Vec<Tile>, x_walls: Vec<Tile>, floor: Vec<Tile>, ceiling: Vec<Tile>) -> Result<Scene, &'static str> {
		if width < 0 || height < 0 {
			return Err("Width and height must be positive values");
		}

		Ok(Scene { width, height, y_walls, x_walls, floor, ceiling })
	}

	pub fn is_within_bounds(&self, x: i32, y: i32) -> bool {
		x >= 0 && x < self.width && y >= 0 && y < self.height
	}

	pub fn x_obstructed(&self, x: i32, y: i32) -> bool {
		let tile = self.x_wall(x, y);
		match tile {
			Tile::Wall(wall) => !wall.passable,
			Tile::OutOfBounds => true,
			_ => false
		}
	}

	pub fn y_obstructed(&self, x: i32, y: i32) -> bool {
		let tile = self.y_wall(x, y);
		match tile {
			Tile::Wall(wall) => !wall.passable,
			Tile::OutOfBounds => true,
			_ => false
		}
	}

	pub fn y_wall(&self, x: i32, y: i32) -> &Tile {
		if !self.is_within_bounds(x, y) { return &Tile::OutOfBounds; }
		&self.y_walls[(x + y  * self.width) as usize]
	}

	pub fn x_wall(&self, x: i32, y: i32) -> &Tile {
		if !self.is_within_bounds(x, y) { return &Tile::OutOfBounds; }
		&self.x_walls[(x + y  * self.width) as usize]
	}

	pub fn ceiling(&self, x: i32, y: i32) -> &Tile {
		if !self.is_within_bounds(x, y) { return &Tile::OutOfBounds; }
		&self.ceiling[(x + y  * self.width) as usize]
	}

	pub fn floor(&self, x: i32, y: i32) -> &Tile {
		if !self.is_within_bounds(x, y) { return &Tile::OutOfBounds; }
		&self.floor[(x + y  * self.width) as usize]
	}

	pub fn from_json(json: &serde_json::Value) -> Result<Scene, &'static str> {
		let width  = json["width"].as_i64().unwrap() as i32;
		let height = json["height"].as_i64().unwrap() as i32;
		
		let x_walls = json["x_walls"].as_array().unwrap().iter()
			.map(|value|   { value.as_i64().unwrap() as u32 })
			.map(|texture| { if texture > 0 { Tile::Wall(TextureTile { texture: texture - 1, passable: false }) } else { Tile::Empty } })
			.collect();

		let y_walls = json["y_walls"].as_array().unwrap().iter()
			.map(|value|   { value.as_i64().unwrap() as u32 })
			.map(|texture| { if texture > 0 { Tile::Wall(TextureTile { texture: texture - 1, passable: false }) } else { Tile::Empty } })
			.collect();

		let floor = json["floor"].as_array().unwrap().iter()
			.map(|value|   { value.as_i64().unwrap() as u32 })
			.map(|texture| { if texture > 0 { Tile::Wall(TextureTile { texture: texture - 1, passable: false }) } else { Tile::Empty } })
			.collect();

		let ceiling = json["ceiling"].as_array().unwrap().iter()
			.map(|value|   { value.as_i64().unwrap() as u32 })
			.map(|texture| { if texture > 0 { Tile::Wall(TextureTile { texture: texture - 1, passable: false }) } else { Tile::Empty } })
			.collect();

		Scene::new(width, height, x_walls, y_walls, floor, ceiling)
	}
}
