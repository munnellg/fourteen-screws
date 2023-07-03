use fourteen_screws::scene::Scene;
use fourteen_screws::render::Camera;
use serde_json;
use wasm_bindgen::prelude::*;
extern crate web_sys;

#[wasm_bindgen]
pub struct FourteenScrewsDemo {
	scene:  Scene,
	player: Camera,
}

#[wasm_bindgen]
impl FourteenScrewsDemo {
	pub fn load_level(json_str: &str) -> FourteenScrewsDemo {
		let json: serde_json::Value = serde_json::from_str(json_str).ok().unwrap();
		
		let scene  = Scene::from_json(&json["scene"]).ok().unwrap();
		let player = Camera::from_json(&json["camera"]).ok().unwrap();

		FourteenScrewsDemo { scene, player }
	}

	pub fn render(&mut self, buf: &mut[u8]) {

	}
}