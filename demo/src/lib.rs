use fourteen_screws::{ Camera, Scene, Renderer };
use fourteen_screws::trig;

use serde_json;
use wasm_bindgen::prelude::*;
extern crate web_sys;

mod player;

const PLAYER_MARGIN: i32     = 28;
const PLAYER_MOVE_SPEED: i32 = 8;
const PLAYER_TURN_SPEED: i32 = trig::ANGLE_5;

#[wasm_bindgen]
pub struct FourteenScrewsDemo {
	scene:  Scene,
	player: player::Player,
	renderer: Renderer,
}

#[wasm_bindgen]
impl FourteenScrewsDemo {
	pub fn player_forward(&mut self) {
		self.player.forward(&self.scene);
	}

	pub fn player_back(&mut self) {
		self.player.back(&self.scene);
	}

	pub fn player_strafe_left(&mut self) {
		self.player.strafe_left(&self.scene);
	}

	pub fn player_strafe_right(&mut self) {
		self.player.strafe_right(&self.scene);
	}

	pub fn player_turn_left(&mut self) {
		self.player.turn_left();
	}

	pub fn player_turn_right(&mut self) {
		self.player.turn_right();
	}

	pub fn load_level(json_str: &str) -> FourteenScrewsDemo {
		let json: serde_json::Value = serde_json::from_str(json_str).ok().unwrap();
		
		let scene    = Scene::try_from(&json["scene"]).ok().unwrap();
		let camera   = Camera::try_from(&json["camera"]).ok().unwrap();
		let renderer = Renderer::try_from(&json["renderer"]).ok().unwrap();

		let player = player::Player::new(camera, PLAYER_MOVE_SPEED, PLAYER_TURN_SPEED, PLAYER_MARGIN);

		FourteenScrewsDemo { scene, player, renderer }
	}

	pub fn render(&mut self, buf: &mut[u8]) {
		self.renderer.render(buf, &self.scene, &self.player.camera);
	}
}