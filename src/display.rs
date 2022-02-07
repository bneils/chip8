use crate::chip8::Config;

use sdl2::{render::Canvas, video::Window, pixels::Color, rect::Rect};

pub fn draw_pixels(canvas: &mut Canvas<Window>, pixels: &[u64; 32], config: Config) {
	canvas.set_draw_color(Color::RGB(0, 0, 0));
	canvas.clear();
	canvas.set_draw_color(Color::RGB(0, 255, 0));
	for y in 0..32 {
		for x in 0..64 {
			if (pixels[y as usize] >> (63 - x)) & 1 != 0 {
				let m = config.screen_magnifier;
				canvas.fill_rect(Rect::new(x * m as i32, y * m as i32, m, m))
					.expect("Canvas couldn't write");
			}
		}
	}
}