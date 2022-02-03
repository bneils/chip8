mod virt;

use virt::Chip8;

extern crate sdl2;
use sdl2::{EventPump, render::Canvas, video::Window, event::Event, pixels::Color, rect::Rect};

use tokio::time::sleep;
use futures::{FutureExt, select, pin_mut};

use std::cell::RefCell;
use std::io::ErrorKind;
use std::time::Duration;
use std::env;

// Nice: Static TV effect, pixel fading
// Needed: Key processing, sound beeps

const MAGNIFIER: u32 = 20;

async fn frame_loop(canvas: &mut Canvas<Window>, chip8: &RefCell<Chip8>, event_pump: &RefCell<EventPump>) -> Result<(), String> {
	loop {
		for event in event_pump.borrow_mut().poll_iter() {
			match event {
				Event::Quit { .. } => {
					return Ok(());
				},
	
				_ => {},
			}
		}

		for y in 0..32 {
			for x in 0..64 {
				let val = (chip8.borrow().display[y as usize] >> (63 - x)) & 1;
				canvas.set_draw_color(
					if val != 0 { Color::RGB(0, 255, 0) }
					else { Color::RGB(0, 0, 0) }
				);
				
				canvas.fill_rect(Rect::new(
					x * MAGNIFIER as i32, 
					y * MAGNIFIER as i32, 
					64 * MAGNIFIER, 32 * MAGNIFIER)
				)?;
			}
		}
		
		canvas.present();
		sleep(Duration::from_millis(1000 / 30)).await
	}
}

async fn cpu_loop(event_pump: &RefCell<EventPump>, chip8: &RefCell<Chip8>) -> Result<(), String> {
	loop {
		match chip8.borrow_mut().next_state(&mut event_pump.borrow_mut()) {
			Err(s) => { return Err(s); },
			_ => {},
		};
		sleep(Duration::from_nanos(1_000_000_000 / 500)).await;
	}
}

async fn async_main(canvas: &mut Canvas<Window>, chip8: RefCell<Chip8>, event_pump: RefCell<EventPump>) -> Result<(), String> {
	chip8.borrow_mut().load_program(vec![0, 224, 162, 42, 96, 12, 97, 8, 208, 31, 112, 9, 162, 57, 208, 31, 162, 72, 112, 8, 208, 31, 112, 4, 162, 87, 208, 31, 112, 8, 162, 102, 208, 31, 112, 8, 162, 117, 208, 31, 18, 40, 255, 0, 255, 0, 60, 0, 60, 0, 60, 0, 60, 0, 255, 0, 255, 255, 0, 255, 0, 56, 0, 63, 0, 63, 0, 56, 0, 255, 0, 255, 128, 0, 224, 0, 224, 0, 128, 0, 128, 0, 224, 0, 224, 0, 128, 248, 0, 252, 0, 62, 0, 63, 0, 59, 0, 57, 0, 248, 0, 248, 3, 0, 7, 0, 15, 0, 191, 0, 251, 0, 243, 0, 227, 0, 67, 224, 0, 224, 0, 128, 0, 128, 0, 128, 0, 128, 0, 224, 0, 224]);

	let future1 = cpu_loop(&event_pump, &chip8).fuse();
	let future2 = frame_loop(canvas, &chip8, &event_pump).fuse();

	pin_mut!(future1, future2); // idk

	select! {
		result1 = future1 => result1,
		result2 = future2 => result2,
	}
}

fn main() -> Result<(), String> {
	let args: Vec<String> = env::args().collect();
	if args.len() <= 1 {
		return Err(String::from("No file argument provided"));
	}

	let file = std::fs::read(&args[1]);
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

	// SDL initialization phase
	let _sdl = sdl2::init().unwrap();
	let video_subsystem = _sdl.video().unwrap();
	let video = video_subsystem
		.window("CHIP 8 Interpreter", 64 * MAGNIFIER, 32 * MAGNIFIER)
		.build()
		.unwrap();

	let mut canvas = video.into_canvas().build().unwrap();
	let event_pump = _sdl.event_pump().unwrap();

	// I have this async because it's easier to have more than one thing running concurrently
	// using this. Otherwise I'd have to simulate my own runtime
	let rt = tokio::runtime::Runtime::new().unwrap();
	let chip8 = RefCell::new(Chip8::new());
	chip8.borrow_mut().load_program(file_bytes);
	rt.block_on(async_main(&mut canvas, chip8, RefCell::new(event_pump)))
}
