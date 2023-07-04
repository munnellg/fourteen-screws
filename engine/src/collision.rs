use crate::maths::trig;

#[derive(PartialEq)]
enum HitResult {
	Nothing,
	SlideX,
	SlideY,
	WallX,
	WallY,
}

struct BoundingBox {
	margin: i32
}

impl BoundingBox {
	fn move_player(&mut self, mut direction: i32, amount: i32) -> HitResult {
		while direction >= trig::ANGLE_360 { direction -= trig::ANGLE_360; }
		while direction < trig::ANGLE_0    { direction += trig::ANGLE_360; }

		let xp = self.player.x;
		let yp = self.player.y;

		// get bounds of the tile player currently occupies
		let x_left   = xp & 0xFFC0;
		let y_top    = yp & 0xFFC0;
		let x_right  = x_left + consts::TILE_SIZE;
		let y_bottom = y_top + consts::TILE_SIZE;

		let mut hit_result = HitResult::Nothing;

		let mut x1 = xp + fp::mul(trig::cos(direction), amount.to_fp()).to_i32();
		let mut y1 = yp + fp::mul(trig::sin(direction), amount.to_fp()).to_i32();
		
		let grid_x = x_left / consts::TILE_SIZE;
		let grid_y = y_top / consts::TILE_SIZE;

		if x1 < xp { // are we moving left
			if let raycast::Tile::Wall(_, passable) = self.world.x_wall(grid_x, grid_y) {
				if !passable && (x1 < x_left || (x1 - x_left).abs() < 28) { // we crossed the wall or we're too close
					x1 = xp;
					hit_result = HitResult::SlideX;
				}
			}
		}

		if x1 > xp { // are we moving right
			if let raycast::Tile::Wall(_, passable) = self.world.x_wall(grid_x + 1, grid_y) { // wall found in current square (right edge)
				if !passable && (x1 > x_right || (x_right - x1).abs() < 28) { // we crossed the wall or we're too close
					x1 = xp;
					hit_result = HitResult::SlideX;
				}
			}
		}

		if y1 < yp { // are we moving up			
			if let raycast::Tile::Wall(_, passable) = self.world.y_wall(grid_x, grid_y) {
				if !passable && (y1 < y_top || (y1 - y_top).abs() < 28) {
					y1 = yp;
					hit_result = HitResult::SlideY;
				}
			}
		}

		if y1 > yp { // are we moving down
			if let raycast::Tile::Wall(_, passable) = self.world.y_wall(grid_x, grid_y + 1) {
				if !passable && (y1 > y_bottom || (y_bottom - y1).abs() < 28) {
					y1 = yp;
					hit_result = HitResult::SlideY;
				}
			}
		}

		// A wall or object hasn't been hit yet. We must look further.
		// The current grid square will be divided into four regions:
		// A = top left; B = top right; C = bottom left; D = bottom right
		// Each of these regions will be checked to see if the player's new position
		// (x1, y1) is close to a wall or object that borders one of these regions.
		// Each grid square is 64x64 units, so each region to check is 32x32 units


		if hit_result == HitResult::Nothing {
			if y1 < (y_top + 32) {    // new y position falls in top half
				
				// check region A-top left area of grid
				if x1 < x_left + 32 { // new x position falls in left half

					// check adjacent x wall (to left)
					if let raycast::Tile::Wall(_, x_passable) = self.world.x_wall(grid_x, grid_y - 1) { 
						if !x_passable && y1 < (y_top + 28) { // adjacent x wall found and new y coord is within 28 units
							if x1 < x_left + 28 {
								if xp > x_left + 27 {
									x1 = xp;
									hit_result = HitResult::SlideX;
								} else {
									y1 = yp;
									hit_result = HitResult::SlideY;
								}
							}
						}
					}

					// check adjacent y wall (above)
					if let raycast::Tile::Wall(_, y_passable) = self.world.y_wall(grid_x - 1, grid_y) {
						if !y_passable && x1 < x_left + 28 {
							if y1 < y_top + 28 {
								if yp > y_top + 27 {
									y1 = yp;
									hit_result = HitResult::SlideY;
								} else {
									x1 = xp;
									hit_result = HitResult::SlideX;
								}
							}
						}
					}
				}

				// check region B-top right area
				if x1 > x_right - 32 && hit_result == HitResult::Nothing {
					
					// check adjacent x wall (to right)
					if let raycast::Tile::Wall(_, x_passable) = self.world.x_wall(grid_x + 1, grid_y - 1) {
						if !x_passable && y1 < y_top + 28 {
							if x1 > x_right - 28 {
								if xp < x_right - 27 {
									x1 = xp;
									hit_result = HitResult::SlideX;
								} else {
									y1 = yp;
									hit_result = HitResult::SlideY;
								}
							}
						}
					}

					// check adjacent y wall (above)
					if let raycast::Tile::Wall(_, y_passable) = self.world.y_wall(grid_x + 1, grid_y) {
						if !y_passable && x1 > x_right - 28 {
							if y1 < y_top + 28 {
								if yp < y_top + 27 {
									y1 = yp;
									hit_result = HitResult::SlideY;
								} else {
									x1 = xp;
									hit_result = HitResult::SlideX;
								}
							}
						}
					}
				}
			}

			// check region C-bottom left area
			if y1 > y_top + 32 && hit_result == HitResult::Nothing {
				if x1 < x_left + 32 {					
					
					// check adjacent x wall (to left)
					if let raycast::Tile::Wall(_, x_passable) = self.world.x_wall(grid_x, grid_y + 1) {
						if !x_passable && y1 > y_bottom - 28 {
							if x1 < x_left + 28 {
								if xp > x_left + 27 {
									x1 = xp;
									hit_result = HitResult::SlideX;
								} else {
									y1 = yp;
									hit_result = HitResult::SlideY;
								}
							}
						}
					}
					
					// check adjacent y wall (below)
					if let raycast::Tile::Wall(_, y_passable) = self.world.y_wall(grid_x - 1, grid_y + 1) {
						if !y_passable && x1 < x_left + 28 {
							if y1 > y_bottom - 28 {
								if yp < y_bottom - 27 {
									y1 = yp;
									hit_result = HitResult::SlideY;
								} else {
									x1 = xp;
									hit_result = HitResult::SlideX;
								}
							}
						}
					}
				}

				// check region D-bottom right area
				if x1 > x_right - 32 && hit_result == HitResult::Nothing {
					
					// check adjacent x wall (to right)
					if let raycast::Tile::Wall(_, x_passable) = self.world.x_wall(grid_x + 1, grid_y + 1) {
						if !x_passable && y1 > y_bottom - 28 {
							if x1 > x_right - 28 {
								if xp < x_right - 27 {
									x1 = xp;
									hit_result = HitResult::SlideX;
								} else {
									y1 = yp;
									hit_result = HitResult::SlideY;
								}
							}
						}
					}

					// check adjacent y wall (below)
					if let raycast::Tile::Wall(_, y_passable) = self.world.y_wall(grid_x + 1, grid_y + 1) {
						if !y_passable && x1 > x_right - 28 {
							if y1 > y_bottom - 28 {
								if yp < y_bottom - 27 {
									y1 = yp;
									hit_result = HitResult::SlideY;
								} else {
									x1 = xp;
									hit_result = HitResult::SlideX;
								}
							}
						}
					}
				}
			}
		}
		
		if hit_result == HitResult::SlideX && y1 == yp {
			hit_result = HitResult::WallX;
		}

		if hit_result == HitResult::SlideY && x1 == xp {
			hit_result = HitResult::WallY;
		}

		self.player.pos(x1, y1);

		hit_result
	}
}