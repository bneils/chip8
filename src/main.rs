mod virt;
use virt::{Chip8, Config};

extern crate sdl2;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

// IDEAS: Static TV effect, pixel fading
// Needed: Key processing, sound beeps

fn main() -> Result<(), String> {
	let _sdl = sdl2::init().unwrap();
    let video_subsystem = _sdl.video().unwrap();
    let video = video_subsystem
        .window("CHIP 8 Interpreter", 700, 700)
        .build()
        .unwrap();

    let mut canvas = video.into_canvas().build().unwrap();
    let mut event_pump = _sdl.event_pump().unwrap();

	let mut chip8 = Chip8::new();
	
	'main: loop {
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit { .. } => {
					break 'main;
				},

				_ => {},
			}
		}
		canvas.set_draw_color(Color::RGB(255, 255, 255));
		match canvas.fill_rect(Rect::new(0, 0, 700, 700)) {
			Err(s) => {
				return Err(format!("error: {}", s));
			},
			_ => {},
		}
		canvas.present();
		
		match chip8.clock() {
			Err(s) => {
				return Err(format!("error: {}", s));
			},
			_ => {},
		}
	}

	Ok(())
}
