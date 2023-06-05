import * as wasm from "fourteen-screws";

let canvas = document.getElementById("canvas");

if (canvas) {
	var context = canvas.getContext("2d");

	if (context) {
		var image = context.getImageData(0, 0, 320, 200);
		wasm.render(image.data);
		context.putImageData(image, 0, 0);	
	}
}
