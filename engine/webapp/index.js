import * as wasm from "fourteen-screws";
import { Cluiche } from "fourteen-screws";

let demo_level = require('./demo-level');
let textures = demo_level.textures;

const cluiche = Cluiche.new();
cluiche.load_textures(textures);

let canvas   = document.getElementById("canvas");
let context  = canvas.getContext("2d", { willReadFrequently: true });
let keystate = {}

document.addEventListener('keydown', (event) => { keystate[event.key] = true; }, false);
document.addEventListener('keyup', (event) => { keystate[event.key] = false; }, false);

const fps = new class {
	constructor() {
		this.fps = document.getElementById("fps");
		this.frames = [];
		this.lastFrameTimestamp = performance.now();
	}

	render () {
		// Convert the delta time since the last frame render into a measure
		// of frames per second
		const now = performance.now();
		const delta = now - this.lastFrameTimeStamp;
		this.lastFrameTimeStamp = now;
		const fps = 1 / delta * 1000;

		// Save only the latest 100 timings.
		this.frames.push(fps);
		if (this.frames.length > 100) {
			this.frames.shift();
		}

		// find the max, min, and mean of our 100 latest timings
		let min = Infinity;
		let max = -Infinity;
		let sum = 0;

		for (let i = 0; i < this.frames.length; i++) {
			sum += this.frames[i];
			min = Math.min(this.frames[i], min);
			max = Math.max(this.frames[i], max);
		}

		let mean = sum / this.frames.length;

		this.fps.textContent = `
Frames per Second:
	     latest = ${Math.round(fps)} |
avg of last 100 = ${Math.round(mean)} |
min of last 100 = ${Math.round(min)} |
max of last 100 = ${Math.round(max)}
`.trim();
	}
}

function render() {
	context.clearRect(0, 0, 320, 200);
	var image = context.getImageData(0, 0, 320, 200);
	cluiche.render(image.data);
	context.putImageData(image, 0, 0);
}

function events() {
	if (keystate['w'] || keystate['ArrowUp']) {
		cluiche.player_forward();
	}

	if (keystate['s'] || keystate['ArrowDown']) {
		cluiche.player_back();
	}

	if (keystate['a']) {
		cluiche.player_strafe_left();
	}

	if (keystate['d']) {
		cluiche.player_strafe_right();
	}

	if (keystate['ArrowLeft']) {
		cluiche.player_turn_left();
	}

	if (keystate['ArrowRight']) {
		cluiche.player_turn_right();
	}
}

function tick() {
	fps.render();
	events();
	render();	
	requestAnimationFrame(tick);	
}

requestAnimationFrame(tick);