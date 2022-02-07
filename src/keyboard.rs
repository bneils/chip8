use sdl2::{EventPump, keyboard::Scancode};

/*
Keyboard layout:
1 2 3 C
4 5 6 D
7 8 9 E
A 0 B F
some instrs can halt on keyboard input, or just get the state of the keyboard.
*/

/// Checks corresponding scancodes
pub fn is_hex_key_pressed(event_pump: &mut EventPump, hex: u8) -> bool {
	if hex > 0xF {
		return false;
	}

	let codes = [
		Scancode::X,
		Scancode::Num1,
		Scancode::Num2,
		Scancode::Num3,
		Scancode::Q,
		Scancode::W,
		Scancode::E,
		Scancode::A,
		Scancode::S,
		Scancode::D,
		Scancode::Z,
		Scancode::C,
		Scancode::Num4,
		Scancode::R,
		Scancode::F,
		Scancode::V,
	];

	let code: Scancode = codes[hex as usize];
	event_pump.keyboard_state().is_scancode_pressed(code)
}

pub fn scancode_to_value(scancode: Scancode) -> Option<u8> {
	match scancode {
		Scancode::Num1 => Some(1),
		Scancode::Num2 => Some(2),
		Scancode::Num3 => Some(3),
		Scancode::Num4 => Some(12),
		Scancode::Q => Some(4),
		Scancode::W => Some(5),
		Scancode::E => Some(6),
		Scancode::R => Some(13),
		Scancode::A => Some(7),
		Scancode::S => Some(8),
		Scancode::D => Some(9),
		Scancode::F => Some(14),
		Scancode::Z => Some(10),
		Scancode::X => Some(0),
		Scancode::C => Some(11),
		Scancode::V => Some(15),
		_ => None,
	}
}