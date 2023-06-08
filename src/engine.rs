use crate::consts;
use crate::raycast::{ Player, World };
use crate::trig;

pub struct Cluiche {
	world: World,
	player: Player,
}

impl Cluiche {
	pub fn new() -> Cluiche {
		let world = World::new(7, 7, "WHHHHHWVOOOOOVVOOOOOVVOOOOOVVOOOOOVVOOOOOVWHHHHHW").unwrap();
		let player = Player::new(128, 128, 0, 5, 5);
		Cluiche { world, player }
	}
	fn move_player(&mut self) {

	}

	fn draw_wall_column(&self, buf: &mut[u8], column: i32, dist: f64) {
		// get wall texture, draw into column
		let wall_height: i32 = consts::WALL_HEIGHT_SCALE_FACTOR / dist.max(1.0) as i32;

		let y_min = std::cmp::max(0, (200 - wall_height) / 2);
		let y_max = std::cmp::min(200 - 1, y_min + wall_height);

		for y in y_min..=y_max {
			let idx: usize = 4 * (column + y * consts::PROJECTION_PLANE_WIDTH) as usize;
			buf[idx + 0] = 0xFF;
			buf[idx + 1] = 0x00;
			buf[idx + 2] = 0x00;
			buf[idx + 3] = 0xFF; // alpha channel
		}
	}

	fn render(&self, buf: &mut[u8]) {
		// draw a grey background that will represent the ceiling and floor
		for x in &mut *buf { *x = 128; }

		// theta is the direction player is facing
		// need to start out sweep 30 degrees to the left
		let mut angle = if self.player.rotation < trig::ANGLE_30 {
			self.player.rotation - trig::ANGLE_30 + trig::ANGLE_360
		} else {
			self.player.rotation - trig::ANGLE_30
		};

		// sweep of the rays will be through 60 degrees
		for sweep in 0..trig::ANGLE_60 {
			let hdist = self.world.find_vertical_intersect(self.player.x, self.player.y, angle);		
			let vdist = self.world.find_horizontal_intersect(self.player.x, self.player.y, angle);
			let dist = hdist.min(vdist) / trig::fisheye_correction(sweep);

			self.draw_wall_column(buf, sweep, dist);

			angle += 1;
			if angle >= trig::ANGLE_360 {
				angle -= trig::ANGLE_360;
			}
		}
	}
}