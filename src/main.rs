mod chip8;
mod display;
mod keyboard;

use chip8::{Chip8, Config};
use display::draw_pixels;

extern crate sdl2;
use sdl2::event::Event;

use std::io::ErrorKind;
use std::time::{Instant, Duration};
use std::thread::sleep;
use std::{fs, env};

// Nice: Static TV effect, pixel fading
// Needed: Key processing, sound beeps

fn main() -> Result<(), String> {
	let args: Vec<String> = env::args().collect();
	if args.len() <= 1 {
		return Err(String::from("No file argument provided"));
	}

	let file = fs::read(&args[1]);
	let file_bytes;
	match file {
		Ok(vec) => {
			file_bytes = vec; 
		},
		Err(err) => {
			return Err(
				match err.kind() {
					ErrorKind::NotFound => {
						format!("Could not find file `{}`", &args[1])
					},
					_ => String::from("Some other error occurred when loading this file")
				}
			);
		}
	}

	let config = Config::new(600, 20);

	// SDL initialization phase
	let _sdl = sdl2::init().unwrap();
	let video_subsystem = _sdl.video().unwrap();
	let video = video_subsystem
		.window("CHIP 8 Interpreter", 64 * config.screen_magnifier, 32 * config.screen_magnifier)
		.build()
		.unwrap();

	let mut canvas = video.into_canvas().build().unwrap();
	let mut event_pump = _sdl.event_pump().unwrap();

	let mut chip8 = Chip8::new();
	
	chip8.load_program(file_bytes, 0x200);

	let mut last_canvas_update = Instant::now();

	loop {		
		// Event loop
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit { .. } => {
					return Ok(());
				},
				
				// Register some new key presses.
				Event::KeyDown { scancode, .. } => {
					match scancode {
						Some(scancode) => {
							chip8.register_key_press(scancode);
						},
						_ => {},
					}
				}
				_ => {},
			}
		}
		
		let result = chip8.next_state(&mut event_pump);
		if matches![result, Err(_)] { return result; }

		draw_pixels(&mut canvas, &chip8.display, config);

		if last_canvas_update.elapsed().as_millis() > 16 {
			canvas.present();
			last_canvas_update = Instant::now();
		}

		sleep(Duration::from_millis((1000 / config.clock_hz).into()));
	}
}
