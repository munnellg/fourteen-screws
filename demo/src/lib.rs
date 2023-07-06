use fourteen_screws::{ Camera, Scene, Renderer };
use serde_json;
use wasm_bindgen::prelude::*;
extern crate web_sys;

#[wasm_bindgen]
pub struct FourteenScrewsDemo {
	scene:  Scene,
	player: Camera,
	renderer: Renderer,
}

#[wasm_bindgen]
impl FourteenScrewsDemo {
	pub fn load_level(json_str: &str) -> FourteenScrewsDemo {
		let json: serde_json::Value = serde_json::from_str(json_str).ok().unwrap();
		
		let scene    = Scene::from_json(&json["scene"]).ok().unwrap();
		let player   = Camera::from_json(&json["camera"]).ok().unwrap();
		let renderer = Renderer::from_json(&json["renderer"]).ok().unwrap();

		FourteenScrewsDemo { scene, player, renderer }
	}

	pub fn render(&mut self, buf: &mut[u8]) {
		self.renderer.render(buf, &self.scene, &self.player);
		self.player.rotate(1);
	}
}