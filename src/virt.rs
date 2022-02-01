use rand::{thread_rng, Rng, prelude::ThreadRng};

//http://devernay.free.fr/hacks/chip8/C8TECH10.HTM

pub struct Config {
	// 150->(300)->500 hz is recommended
	clock_hz: u32, // the upperbound for how many instrs are done a second
	screen_magnifier: u32, // x1, x2, x4, etc.
}

/*
Keyboard layout:
1 2 3 C
4 5 6 D
7 8 9 E
A 0 B F
some instrs can halt on keyboard input, or just get the state of the keyboard.
*/

pub struct Chip8 {
	pc: usize,
	buf: [u8; 4096],
	registers: [u8; 16],
	addr_register: u16, // "I" register
	addr_stack: Vec<u16>, // no additional alloc is done if you specify .with_capacity()
	delay_timer: u8,
	sound_timer: u8,
	display: [u64; 32],
	rand_thread: ThreadRng,
}

impl Chip8 {
	pub fn new() -> Chip8 {
		let mut buf = [0; 4096];
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
			buf[i] = sprites[i];
		}

		Chip8 {
			pc: 0,
			buf,
			registers: [0; 16],
			addr_register: 0,
			addr_stack: Vec::with_capacity(16),
			delay_timer: 0,
			sound_timer: 0,
			display: [0; 32],
			rand_thread: thread_rng(),
		}
	}

	pub fn clock(&mut self) -> Result<(), &str> {
		let instr: u16 = 
			(self.buf[self.pc] as u16) << 8 | 
			 self.buf[self.pc + 1] as u16;
		
		self.pc += 2;
	
		let x = ((instr & 0x0F00) >> 8) as usize;
		let y = ((instr & 0x00F0) >> 4) as usize;

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
					None => return Err("attempted to exit from a subroutine when there was nothing to return to."),
				}
			},
			_ => {},
		}
		
		match instr & 0xF000 {
			// 1nnn: Jumps to address NNN.
			0x1000 => {
				self.pc = (instr & 0x0FFF) as usize;
			},
			// 2nnn: Calls subroutine at NNN
			0x2000 => {
				self.addr_stack.push(self.pc as u16);
				self.pc = (instr & 0x0FFF) as usize;
			},
			// 3xkk: Skips the next instruction if Vx equals kk.
			0x3000 => {
				if self.registers[x] == instr as u8 {
					self.pc += 2;
				}
			},
			// 4xkk: Skip next instruction if Vx != kk.
			0x4000 => {
				if self.registers[x] != instr as u8 {
					self.pc += 2;
				}
			},
			// 6xkk: The interpreter puts the value kk into register Vx.
			0x6000 => {
				self.registers[x] = instr as u8;
			},
			// 7xkk:  Adds the value kk to the value of register Vx, then stores the result in Vx.
			0x7000 => {
				self.registers[x] += instr as u8;
			},
			// Annn:  The value of register I is set to nnn.
			0xA000 => {
				self.addr_register = instr & 0x0FFF;
			},
			// Bnnn:  Jump to location nnn + V0.
			0xB000 => {
				self.pc = self.registers[0] as usize + (instr & 0x0FFF) as usize;
			},
			// Set Vx = random byte AND kk.
			0xC000 => {
				let b: u8 = self.rand_thread.gen();
				self.registers[x] = b & (instr as u8);
			},
			0xD000 => {
				panic!("Not implemented");
			},
			_ => {},
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
				self.registers[0xF] = overflow as u8;
			},
			// 8xy5: Set Vx = Vx - Vy, set VF = NOT borrow.
			0x8005 => {
				let (diff, underflow) = self.registers[x].overflowing_sub(self.registers[y]);
				self.registers[x] = diff;
				self.registers[0xF] = (!underflow) as u8;
			},
			// 8xy6: Set Vx = Vy = Vy SHR 1.
			0x8006 => {
				//https://www.reddit.com/r/EmuDev/comments/72dunw/chip8_8xy6_help/
				self.registers[0xF] = self.registers[y] & 1;
				self.registers[y] >>= 1;
				self.registers[x] = self.registers[y];
			},
			// 8xy7: Set Vx = Vy - Vx, set VF = NOT borrow.
			0x8007 => {
				//COPIED^^^^
				let (diff, underflow) = self.registers[y].overflowing_sub(self.registers[x]);
				self.registers[x] = diff;
				self.registers[0xF] = (!underflow) as u8;
			},
			// 8xyE: Set Vx = Vx SHL 1.
			0x800E => {
				// TODO: CLEAR CONFUSION AROUND 800E and 8006
				self.registers[0xF] = self.registers[x] >> 7;
				self.registers[y] <<= 1;
				self.registers[x] = self.registers[y];
			},
			// 9xy0: Skip next instruction if Vx != Vy.
			0x9000 => {
				if self.registers[x] != self.registers[y] {
					self.pc += 2;
				}
			},
			_ => {},
		}

		match instr & 0xF0FF {
			// Ex93: Skip next instruction if key with the value of Vx is pressed.
			0xE093 => {
				panic!("Not implemented");
			},
			// ExA1: Skip next instruction if key with the value of Vx is not pressed.
			0xE0A1 => {
				panic!("Not implemented");
			},
			// Fx07: Set Vx = delay timer value.
			0xF007 => {
				self.registers[x] = self.delay_timer;
			},
			// Fx0A: Wait for a key press, store the value of the key in Vx.
			0xF00A => {
				panic!("Not implemented");
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
				panic!("Not implemented");
			},
			// Fx33: Store BCD representation of Vx in memory locations I, I+1, and I+2.
			0xF033 => {
				self.buf[self.addr_register as usize] = self.registers[x] / 100;
				self.buf[self.addr_register as usize + 1] = self.registers[x] / 10 % 10;
				self.buf[self.addr_register as usize + 2] = self.registers[x] % 10;
			},
			// Fx55: Store registers V0 through Vx in memory starting at location I.
			0xF055 => {
				for i in 0..=x {
					self.buf[self.addr_register as usize + i] = self.registers[i];
				}
			},
			// Fx65: Read registers V0 through Vx from memory starting at location I.
			0xF065 => {
				for i in 0..=x {
					self.registers[i] = self.buf[self.addr_register as usize + i];
				}
			},
			_ => {},
		}

		Ok(())
	}
}