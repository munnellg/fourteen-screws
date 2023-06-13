use std::fs::File;

fn transform_tile(outbuf: &mut Vec<u8>, imgbuf: &[u8], tile_id: u32, tile_size: u32, img_width: u32) {
	let y_min = ((tile_id * tile_size) / img_width) as usize;
	let x_min = ((tile_id * tile_size) % img_width) as usize;
	let y_max = y_min + tile_size as usize;
	let x_max = x_min + tile_size as usize;

	for x in x_min..x_max {
		for y in y_min..y_max {
			let p = (x + y * img_width as usize) * 4;
			outbuf.push(imgbuf[p + 0]);
			outbuf.push(imgbuf[p + 1]);
			outbuf.push(imgbuf[p + 2]);
			outbuf.push(imgbuf[p + 3]);
		}
	}
}

pub fn convert_file(fname: &String, tile_size: u32) -> Vec<u8> {
	let file = File::open(fname).expect(format!("unable to open '{}'", fname).as_str());
	// The decoder is a build for reader and can be used to set various decoding options
	// via `Transformations`. The default output transformation is `Transformations::IDENTITY`.
	let mut decoder = png::Decoder::new(file);

	// validate
	let header = decoder.read_header_info().expect("problem decoding image header");

	let img_width = header.width;
	let img_height = header.height;
	assert!(img_width > 0 && img_height > 0, "image has zero width or height");
	assert!(img_width % tile_size == 0 && img_height % tile_size == 0, "tile sheet height and width must be even multiples of tile_size");
	assert!(header.color_type == png::ColorType::Rgba, "currently only support pngs in rgba format");
	assert!(header.bit_depth == png::BitDepth::Eight, "currently only support pngs with 8-bit depth");

	let num_tiles = (img_width * img_height) / (tile_size * tile_size);

	let mut reader = decoder.read_info().expect("problem decoding image data");

	// Allocate the output buffer.
	let mut buf = vec![0; reader.output_buffer_size()];

	// Read the next frame. An APNG might contain multiple frames.
	let info = reader.next_frame(&mut buf).expect("problem accessing next png frame");

	// Grab the bytes of the image.
	let bytes = &buf[..info.buffer_size()];
	let mut result = Vec::new();
	result.reserve(info.buffer_size());

	for tile_id in 0..num_tiles {
		transform_tile(&mut result, &bytes, tile_id, tile_size, img_width);
	}

	result
}
	