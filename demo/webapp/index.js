import * as wasm from "demo";
import { FourteenScrewsDemo } from "demo";

let level = require('./demo-level');

const demo = FourteenScrewsDemo.load_level(JSON.stringify(level));

let canvas   = document.getElementById("canvas");
let context  = canvas.getContext("2d", { willReadFrequently: true });
let keystate = {}

document.addEventListener('keydown', (event) => { keystate[event.code] = true; }, false);
document.addEventListener('keyup', (event) => { keystate[event.code] = false; }, false);

window.mobileAndTabletCheck = function() {
  let check = false;
  (function(a){if(/(android|bb\d+|meego).+mobile|avantgo|bada\/|blackberry|blazer|compal|elaine|fennec|hiptop|iemobile|ip(hone|od)|iris|kindle|lge |maemo|midp|mmp|mobile.+firefox|netfront|opera m(ob|in)i|palm( os)?|phone|p(ixi|re)\/|plucker|pocket|psp|series(4|6)0|symbian|treo|up\.(browser|link)|vodafone|wap|windows ce|xda|xiino|android|ipad|playbook|silk/i.test(a)||/1207|6310|6590|3gso|4thp|50[1-6]i|770s|802s|a wa|abac|ac(er|oo|s\-)|ai(ko|rn)|al(av|ca|co)|amoi|an(ex|ny|yw)|aptu|ar(ch|go)|as(te|us)|attw|au(di|\-m|r |s )|avan|be(ck|ll|nq)|bi(lb|rd)|bl(ac|az)|br(e|v)w|bumb|bw\-(n|u)|c55\/|capi|ccwa|cdm\-|cell|chtm|cldc|cmd\-|co(mp|nd)|craw|da(it|ll|ng)|dbte|dc\-s|devi|dica|dmob|do(c|p)o|ds(12|\-d)|el(49|ai)|em(l2|ul)|er(ic|k0)|esl8|ez([4-7]0|os|wa|ze)|fetc|fly(\-|_)|g1 u|g560|gene|gf\-5|g\-mo|go(\.w|od)|gr(ad|un)|haie|hcit|hd\-(m|p|t)|hei\-|hi(pt|ta)|hp( i|ip)|hs\-c|ht(c(\-| |_|a|g|p|s|t)|tp)|hu(aw|tc)|i\-(20|go|ma)|i230|iac( |\-|\/)|ibro|idea|ig01|ikom|im1k|inno|ipaq|iris|ja(t|v)a|jbro|jemu|jigs|kddi|keji|kgt( |\/)|klon|kpt |kwc\-|kyo(c|k)|le(no|xi)|lg( g|\/(k|l|u)|50|54|\-[a-w])|libw|lynx|m1\-w|m3ga|m50\/|ma(te|ui|xo)|mc(01|21|ca)|m\-cr|me(rc|ri)|mi(o8|oa|ts)|mmef|mo(01|02|bi|de|do|t(\-| |o|v)|zz)|mt(50|p1|v )|mwbp|mywa|n10[0-2]|n20[2-3]|n30(0|2)|n50(0|2|5)|n7(0(0|1)|10)|ne((c|m)\-|on|tf|wf|wg|wt)|nok(6|i)|nzph|o2im|op(ti|wv)|oran|owg1|p800|pan(a|d|t)|pdxg|pg(13|\-([1-8]|c))|phil|pire|pl(ay|uc)|pn\-2|po(ck|rt|se)|prox|psio|pt\-g|qa\-a|qc(07|12|21|32|60|\-[2-7]|i\-)|qtek|r380|r600|raks|rim9|ro(ve|zo)|s55\/|sa(ge|ma|mm|ms|ny|va)|sc(01|h\-|oo|p\-)|sdk\/|se(c(\-|0|1)|47|mc|nd|ri)|sgh\-|shar|sie(\-|m)|sk\-0|sl(45|id)|sm(al|ar|b3|it|t5)|so(ft|ny)|sp(01|h\-|v\-|v )|sy(01|mb)|t2(18|50)|t6(00|10|18)|ta(gt|lk)|tcl\-|tdg\-|tel(i|m)|tim\-|t\-mo|to(pl|sh)|ts(70|m\-|m3|m5)|tx\-9|up(\.b|g1|si)|utst|v400|v750|veri|vi(rg|te)|vk(40|5[0-3]|\-v)|vm40|voda|vulc|vx(52|53|60|61|70|80|81|83|85|98)|w3c(\-| )|webc|whit|wi(g |nc|nw)|wmlb|wonu|x700|yas\-|your|zeto|zte\-/i.test(a.substr(0,4))) check = true;})(navigator.userAgent||navigator.vendor||window.opera);
  return check;
};

if (window.mobileAndTabletCheck()) {
	let left_stick = new JoyStick('joystick-left', { "internalFillColor": "#CCCCCC", "internalStrokeColor": "#333333", "externalStrokeColor": "#333333" }, function(stickData) {
		keystate["joystick-left"] = stickData.cardinalDirection;    
	});

	let right_stick = new JoyStick('joystick-right', { "internalFillColor": "#CCCCCC", "internalStrokeColor": "#333333", "externalStrokeColor": "#333333" }, function(stickData) {
		keystate["joystick-right"] = stickData.cardinalDirection;    
	});
}

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
	demo.render(image.data);
	context.putImageData(image, 0, 0);
}

function events() {
	if (keystate['KeyW'] || [ "N", "NW", "NE" ].includes(keystate['joystick-left'])) {
		demo.player_forward();
	}

	if (keystate['KeyS'] || [ "S", "SW", "SE" ].includes(keystate['joystick-left'])) {
		demo.player_back();
	}

	if (keystate['KeyA'] || [ "W", "SW", "NW" ].includes(keystate['joystick-left'])) {
		demo.player_strafe_left();
	}

	if (keystate['KeyD'] || [ "E", "SE", "NE" ].includes(keystate['joystick-left'])) {
		demo.player_strafe_right();
	}

	// if (keystate['ArrowUp'] || [ "N", "NW", "NE" ].includes(keystate['joystick-right'])) {
	// 	demo.player_look_up();
	// }

	// if (keystate['ArrowDown'] || [ "S", "SW", "SE" ].includes(keystate['joystick-right'])) {
	// 	demo.player_look_down();
	// }

	if (keystate['ArrowLeft'] || [ "W", "SW", "NW" ].includes(keystate['joystick-right'])) {
		demo.player_turn_left();
	}

	if (keystate['ArrowRight'] || [ "E", "SE", "NE" ].includes(keystate['joystick-right'])) {
		demo.player_turn_right();
	}
}

function tick() {
	fps.render();
	events();
	render();	
	requestAnimationFrame(tick);	
}

requestAnimationFrame(tick);