use rand::{thread_rng, Rng, prelude::ThreadRng};
use sdl2::EventPump;
use sdl2::keyboard::Scancode;

use crate::keyboard;

#[derive(Copy, Clone)]
pub struct Config {
	pub clock_hz: u32,
	pub screen_magnifier: u32,	// How many screen pixels per console pixel
}

impl Config {
	pub fn new(clock_hz: u32, screen_magnifier: u32) -> Config {
		Config { clock_hz, screen_magnifier }
	}
}

pub struct Chip8 {
	pc: usize,
	
	memory: [u8; 4096],
	registers: [u8; 16],
	addr_register: u16, // "I" register
	addr_stack: Vec<u16>, // 16 ptrs
	
	delay_timer: u8,
	sound_timer: u8,

	pub display: [u64; 32],
	
	rand_thread: ThreadRng,

	pressed_scancodes: Vec<Scancode>,
}

//Sources:
//http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
//https://en.wikipedia.org/wiki/CHIP-8

impl Chip8 {
	pub fn new() -> Chip8 {
		let mut memory = [0; 4096];
		let sprites = [
			0xF0, 0x90, 0x90, 0x90, 0xF0, // 0x0
			0x20, 0x60, 0x20, 0x20, 0x70,
			0xF0, 0x10, 0xF0, 0x80, 0xF0,
			0xF0, 0x10, 0xF0, 0x10, 0xF0,
			0x90, 0x90, 0xF0, 0x10, 0x10,
			0xF0, 0x80, 0xF0, 0x10, 0xF0,
			0xF0, 0x80, 0xF0, 0x90, 0xF0,
			0xF0, 0x10, 0x20, 0x40, 0x40, // ...
			0xF0, 0x90, 0xF0, 0x90, 0xF0,
			0xF0, 0x90, 0xF0, 0x10, 0xF0,
			0xF0, 0x90, 0xF0, 0x90, 0x90,
			0xE0, 0x90, 0xE0, 0x90, 0xE0,
			0xF0, 0x80, 0x80, 0x80, 0xF0,
			0xE0, 0x90, 0x90, 0x90, 0xE0,
			0xF0, 0x80, 0xF0, 0x80, 0xF0,
			0xF0, 0x80, 0xF0, 0x80, 0x80, // 0xF
		];
		
		for i in 0..sprites.len() {
			memory[i] = sprites[i];
		}

		Chip8 {
			pc: 0x200,
			memory: memory,
			registers: [0; 16],
			addr_register: 0,
			addr_stack: Vec::with_capacity(16),
			delay_timer: 0,
			sound_timer: 0,
			display: [0; 32],
			rand_thread: thread_rng(),
			pressed_scancodes: Vec::new(),
		}
	}

	/// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
	/// If the sprite is positioned so part of it is outside the coordinates of the display, 
	/// it wraps around to the opposite side of the screen
	fn xor_sprite(&mut self, x: u8, y: u8, n: u8) {
		for i in 0..n {
			let sprite_row = (self.memory[self.addr_register as usize + i as usize] as u64) << (64 - 8);
			let rotated_row = sprite_row.rotate_right(self.registers[x as usize].into());
			let r = ((self.registers[y as usize] + i) % 32) as usize;
			let changes = self.display[r] & rotated_row;
			self.registers[0xF] = (changes != 0).into();
			self.display[r] ^= rotated_row;
		}
	}

	#[inline]
	pub fn register_key_press(&mut self, scancode: Scancode) {
		self.pressed_scancodes.push(scancode);
	}

	pub fn load_program(&mut self, memory: Vec<u8>, start_address: usize) {
		for (i, e) in memory.iter().enumerate() {
			self.memory[i + start_address] = *e;
		}
	}

	pub fn next_state(&mut self, event_pump: &mut EventPump) -> Result<(), String> {
		if self.pc >= self.memory.len() {
			return Err("program counter exceeded memory allocations".to_string());
		}
		let instr: u16 = 
			(self.memory[self.pc] as u16) << 8 | 
			 self.memory[self.pc + 1] as u16;
		//println!("pc={},instr={}",self.pc,instr);
		self.pc += 2;
	
		let x = ((instr & 0x0F00) >> 8) as usize;
		let y = ((instr & 0x00F0) >> 4) as usize;
		let nnn = instr & 0xFFF;
		let nn = (instr & 0xFF) as u8;

		let mut missed_matches = 0;

		match instr {
			// Clears the screen.
			0x00E0 => {
				self.display.fill(0);
			},
			// Returns from a subroutine.
			0x00EE => {
				match self.addr_stack.pop() {
					Some(addr) => {
						self.pc = addr as usize;
					},
					None => return Err(
						"attempted to exit from a subroutine when there was nothing to return to."
						.to_string()
					),
				}
			},
			_ => missed_matches += 1,
		}
		
		match instr & 0xF000 {
			// 1nnn: Jumps to address NNN.
			0x1000 => {
				self.pc = nnn.into();
			},
			// 2nnn: Calls subroutine at NNN
			0x2000 => {
				if self.addr_stack.len() == 16 {
					return Err(
						"maximum level of subroutines reached"
						.to_string()
					);
				}
				self.addr_stack.push(self.pc as u16);
				self.pc = nnn.into();
			},
			// 3xkk: Skips the next instruction if Vx equals kk.
			0x3000 => {
				if self.registers[x] == nn {
					self.pc += 2;
				}
			},
			// 4xkk: Skip next instruction if Vx != kk.
			0x4000 => {
				if self.registers[x] != nn {
					self.pc += 2;
				}
			},
			// 6xkk: The interpreter puts the value kk into register Vx.
			0x6000 => {
				self.registers[x] = nn;
			},
			// 7xkk:  Adds the value kk to the value of register Vx, then stores the result in Vx.
			0x7000 => {
				self.registers[x] = self.registers[x].wrapping_add(nn);
			},
			// Annn:  The value of register I is set to nnn.
			0xA000 => {
				self.addr_register = nnn;
			},
			// Bnnn:  Jump to location nnn + V0.
			0xB000 => {
				self.pc = self.registers[0] as usize + nnn as usize;
			},
			// Cxkk: Set Vx = random byte AND kk.
			0xC000 => {
				let b: u8 = self.rand_thread.gen();
				self.registers[x] = b & nn;
			},
			// Dxyn: Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
			0xD000 => {
				self.xor_sprite(x as u8, y as u8, nn & 0xF);
			},
			_ => missed_matches += 1,
		}

		match instr & 0xF00F {
			// 5xy0: Skip next instruction if Vx = Vy.
			0x5000 => {
				if self.registers[x] == self.registers[y] {
					self.pc += 2;
				}
			}
			// 8xy0: Stores the value of register Vy in register Vx.
			0x8000 => {
				self.registers[x] = self.registers[y];
			},
			// 8xy1: Set Vx = Vx OR Vy.
			0x8001 => {
				self.registers[x] |= self.registers[y];
			},
			// 8xy2: Set Vx = Vx AND Vy.
			0x8002 => {
				self.registers[x] &= self.registers[y];
			},
			// 8xy3: Set Vx = Vx XOR Vy.
			0x8003 => {
				self.registers[x] ^= self.registers[y];
			},
			// 8xy4: Set Vx = Vx + Vy, set VF = carry.
			0x8004 => {
				let (sum, overflow) = self.registers[x].overflowing_add(self.registers[y]);
				self.registers[x] = sum;
				self.registers[0xF] = overflow.into();
			},
			// 8xy5: Set Vx = Vx - Vy, set VF = NOT borrow.
			0x8005 => {
				self.registers[0xF] = (self.registers[x] >= self.registers[y]).into();
				self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]);
			},
			// 8xy6: Set Vx = Vx SHR 1.
			0x8006 => {
				// TODO: CLEAR CONFUSION AROUND 800E and 8006
				//https://www.reddit.com/r/EmuDev/comments/72dunw/chip8_8xy6_help/
				self.registers[0xF] = self.registers[x] & 1;
				self.registers[x] >>= 1;
			},
			// 8xy7: Set Vx = Vy - Vx, set VF = NOT borrow.
			0x8007 => {
				self.registers[0xF] = (self.registers[y] >= self.registers[x]).into();
				self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]);
			},
			// 8xyE: Set Vx = Vx SHL 1.
			0x800E => {
				self.registers[0xF] = self.registers[x] >> 7;
				self.registers[x] <<= 1;
			},
			// 9xy0: Skip next instruction if Vx != Vy.
			0x9000 => {
				if self.registers[x] != self.registers[y] {
					self.pc += 2;
				}
			},
			_ => missed_matches += 1,
		}

		match instr & 0xF0FF {
			// Ex93: Skip next instruction if key with the value of Vx is pressed.
			0xE093 => {
				if keyboard::is_hex_key_pressed(event_pump, self.registers[x]) {
					self.pc += 2;
				}
			},
			// ExA1: Skip next instruction if key with the value of Vx is not pressed.
			0xE0A1 => {
				if !keyboard::is_hex_key_pressed(event_pump, self.registers[x]) {
					self.pc += 2;
				}
			},
			// Fx07: Set Vx = delay timer value.
			0xF007 => {
				self.registers[x] = self.delay_timer;
			},
			// Fx0A: Wait for a key press, store the value of the key in Vx.
			0xF00A => {
				if self.pressed_scancodes.len() == 0 {
					self.pc -= 2; // Go back if nothing pressed
				} else {
					self.registers[x] = keyboard::scancode_to_value(self.pressed_scancodes[0]).unwrap();
					self.pressed_scancodes.clear();
				}
			},
			// Fx15: Set delay timer = Vx.
			0xF015 => {
				self.delay_timer = self.registers[x];
			},
			// Fx18: Set sound timer = Vx.
			0xF018 => {
				self.sound_timer = self.registers[x];
			},
			// Fx1E: Set I = I + Vx.
			0xF01E => {
				self.addr_register += self.registers[x] as u16;
			},
			// Fx29: Set I = location of sprite for digit Vx.
			0xF029 => {
				self.addr_register = 5 * self.registers[x] as u16;
			},
			// Fx33: Store BCD representation of Vx in memory locations I, I+1, and I+2.
			0xF033 => {
				self.memory[self.addr_register as usize] = self.registers[x] / 100;
				self.memory[self.addr_register as usize + 1] = self.registers[x] / 10 % 10;
				self.memory[self.addr_register as usize + 2] = self.registers[x] % 10;
			},
			// Fx55: Store registers V0 through Vx in memory starting at location I.
			0xF055 => {
				for i in 0..=x {
					self.memory[self.addr_register as usize + i] = self.registers[i];
				}
			},
			// Fx65: Read registers V0 through Vx from memory starting at location I.
			0xF065 => {
				for i in 0..=x {
					self.registers[i] = self.memory[self.addr_register as usize + i];
				}
			},
			_ => missed_matches += 1,
		}

		if missed_matches == 4 {
			Err(format!("Unrecognized instruction {} at pc={}", instr, self.pc - 2))
		} else {
			Ok(())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::Chip8;

	#[test]
	fn sprite_xor() {
		let mut chip8 = Chip8::new();
		chip8.xor_sprite(1, 1, 10);
		println!("{:?}", chip8.display);
	}
}