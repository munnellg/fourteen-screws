import * as wasm from "fourteen-screws";

let angle = 0;

function render() {

	let canvas = document.getElementById("canvas");

	if (canvas) {
		var context = canvas.getContext("2d");

		if (context) {
			context.clearRect(0, 0, 320, 200);
			var image = context.getImageData(0, 0, 320, 200);
			wasm.render(image.data, angle);
			context.putImageData(image, 0, 0);	
		}
	}
	angle++;
	if ( angle >= 1920 ) { angle = 0; }
	requestAnimationFrame(render);	
}

requestAnimationFrame(render);