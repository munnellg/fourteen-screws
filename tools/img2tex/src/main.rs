use base64::{Engine as _, engine::general_purpose};
use clap::Parser;
use img2tex::convert_file;
use std::fs;

#[derive(Parser)]
#[command(name = "img2tex")]
#[command(version)]
#[command(about = "Convert tile sheets to be compatible with fourteen screws engine", long_about = None)]
struct Cli {
	/// size of a single tile in the sheet
	#[arg(short, long, default_value_t = 64)]
	tile_size: u32,

	/// output file
	#[arg(short, long, default_value_t = String::from("out.base64"))]
	output: String,

	#[arg(required=true)]
	image_file: String,
}

fn base64_encode(img: &Vec<u8>) -> String {
	general_purpose::STANDARD_NO_PAD.encode(img)
}

fn main() {
	let args: Cli = Cli::parse();
	let image_data = convert_file(&args.image_file, args.tile_size);
	let encoded: String = base64_encode(&image_data);
	fs::write(&args.output, encoded).expect(format!("unable to write output to '{}'", &args.output).as_str());
}
