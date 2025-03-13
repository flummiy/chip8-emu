use drivers::display_driver::WINDOW_HEIGHT;
use drivers::display_driver::WINDOW_WIDTH;
use drivers::display_driver::draw_screen;
use rand::Rng;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use std::fs;
use std::io;
use std::time::Duration;

use sdl3;

pub mod drivers;

use drivers::input_driver::process_input;

const START_ADDRESS: usize = 0x200;
const FONTSET_SIZE: usize = 80;
const FONTSET_START_ADDRESS: usize = 0x50;

pub const CHIP8_WIDTH: usize = 64;
pub const CHIP8_HEIGHT: usize = 32;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8 {
    pub registers: [u8; 16],
    pub memory: [u8; 4096],
    pub index: u16,
    pub pc: u16,
    pub stack: [u16; 16],
    pub sp: u8,
    pub dtimer: u8,
    pub stimer: u8,
    pub keypad: [bool; 16],
    pub video: [bool; 64 * 32],
    pub opcode: u16,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut new_chip8 = Self {
            pc: START_ADDRESS as u16,
            memory: [0; 4096],
            video: [false; 64 * 32],
            registers: [0; 16],
            index: 0,
            sp: 0,
            stack: [0; 16],
            keypad: [false; 16],
            dtimer: 0,
            stimer: 0,
            opcode: 0,
        };

        new_chip8.memory[FONTSET_START_ADDRESS..FONTSET_START_ADDRESS + FONTSET_SIZE]
            .copy_from_slice(&FONTSET);

        new_chip8
    }

    pub fn run(&mut self, rom: &str, ticks_per_frame: usize) {
        let sdl_context = sdl3::init().unwrap();

        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("Chip8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas();
        canvas.clear();
        canvas.present();

        let mut event_pump = sdl_context.event_pump().unwrap();

        self.load_rom(rom).unwrap();

        let target_frame_duration = Duration::from_secs_f64(1.0 / 60.0);

        'gameloop: loop {
            let frame_start = std::time::Instant::now();

            for evt in event_pump.poll_iter() {
                match evt {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        break 'gameloop;
                    }
                    Event::KeyDown {
                        keycode: Some(key), ..
                    } => {
                        if let Some(k) = process_input(key) {
                            self.keypress(k, true);
                        }
                    }
                    Event::KeyUp {
                        keycode: Some(key), ..
                    } => {
                        if let Some(k) = process_input(key) {
                            self.keypress(k, false);
                        }
                    }
                    _ => (),
                }
            }

            for _ in 0..ticks_per_frame {
                self.tick();
            }
            self.tick_timers();
            draw_screen(&self, &mut canvas);

            let elapsed = frame_start.elapsed();
            if elapsed < target_frame_duration {
                let sleep_time = target_frame_duration - elapsed;
                std::thread::sleep(sleep_time);
            }
        }
    }

    pub fn load_rom(&mut self, filename: &str) -> io::Result<()> {
        let rom_data = fs::read(filename)?;

        let load_range = START_ADDRESS..START_ADDRESS + rom_data.len();

        if load_range.end > self.memory.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "ROM too large to fit in memory",
            ));
        }

        self.memory[load_range].copy_from_slice(&rom_data);

        Ok(())
    }

    pub fn get_display(&self) -> &[bool] {
        &self.video
    }

    pub fn fetch(&mut self) -> u16 {
        let higher_byte = self.memory[self.pc as usize] as u16;
        let lower_byte = self.memory[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    pub fn tick(&mut self) {
        let op = self.fetch();

        self.execute(op);
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keypad[idx] = pressed;
    }

    pub fn tick_timers(&mut self) {
        if self.dtimer > 0 {
            self.dtimer -= 1;
        }

        if self.stimer > 0 {
            self.stimer -= 1;
        }
    }

    pub fn execute(&mut self, opcode: u16) {
        let nibbles = (
            (opcode & 0xF000) >> 12, // First Digit
            (opcode & 0x0F00) >> 8,  // Second Digit
            (opcode & 0x00F0) >> 4,  // Third Digit
            (opcode & 0x000F),       // Fourth Digit
        );

        match nibbles {
            // NOP
            (0, 0, 0, 0) => return,
            // CLS
            (0, 0, 0xE, 0) => self.video = [false; 64 * 32],
            // RET
            (0, 0, 0xE, 0xE) => {
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize]
            }
            // JP addr
            (1, _, _, _) => {
                let address = opcode & 0x0FFF;

                self.pc = address;
            }
            // CALL addr
            (2, _, _, _) => {
                let address = opcode & 0x0FFF;

                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = address;
            }
            // SE Vx, byte
            (3, _, _, _) => {
                let vx = nibbles.1 as usize;
                let byte = opcode & 0x00FF;

                if self.registers[vx] == byte as u8 {
                    self.pc += 2;
                }
            }
            // SNE Vx, byte
            (4, _, _, _) => {
                let vx = nibbles.1 as usize;
                let byte = opcode & 0x00FF;

                if self.registers[vx] != byte as u8 {
                    self.pc += 2;
                }
            }
            // SE Vx, Vy
            (5, _, _, _) => {
                let vx = nibbles.1 as usize;
                let vy = nibbles.2 as usize;

                if self.registers[vx] == self.registers[vy] {
                    self.pc += 2;
                }
            }
            // LD Vx, byte
            (6, _, _, _) => {
                let vx = nibbles.1 as usize;
                let byte = opcode & 0x00FF;

                self.registers[vx] = byte as u8;
            }
            // ADD Vx, byte
            (7, _, _, _) => {
                let vx = nibbles.1 as usize;
                let byte = opcode & 0x00FF;

                self.registers[vx] = self.registers[vx].wrapping_add(byte as u8);
            }
            // LD Vx, Vy
            (8, _, _, 0) => {
                let vx = nibbles.1 as usize;
                let vy = nibbles.2 as usize;

                self.registers[vx] = self.registers[vy];
            }
            // OR Vx, Vy
            (8, _, _, 1) => {
                let vx = nibbles.1 as usize;
                let vy = nibbles.2 as usize;

                self.registers[vx] |= self.registers[vy];
            }
            // AND Vx, Vy
            (8, _, _, 2) => {
                let vx = nibbles.1 as usize;
                let vy = nibbles.2 as usize;

                self.registers[vx] &= self.registers[vy];
            }
            // XOR Vx, Vy
            (8, _, _, 3) => {
                let vx = nibbles.1 as usize;
                let vy = nibbles.2 as usize;

                self.registers[vx] ^= self.registers[vy];
            }
            // ADD Vx, Vy
            (8, _, _, 4) => {
                let vx = nibbles.1 as usize;
                let vy = nibbles.2 as usize;

                let (new_vx, carry) = self.registers[vx].overflowing_add(self.registers[vy]);
                let new_vf = if carry { 1 } else { 0 };

                self.registers[vx] = new_vx;
                self.registers[0xF] = new_vf;
            }
            // SUB Vx, Vy
            (8, _, _, 5) => {
                let vx = nibbles.1 as usize;
                let vy = nibbles.2 as usize;

                let (new_vx, borrow) = self.registers[vx].overflowing_sub(self.registers[vy]);
                let new_vf = if borrow { 0 } else { 1 };

                self.registers[vx] = new_vx;
                self.registers[0xF] = new_vf;
            }
            // SHR Vx
            (8, _, _, 6) => {
                let vx = nibbles.1 as usize;

                // Save LSB in VF
                self.registers[0xF] = self.registers[vx] & 0x1;

                self.registers[vx] >>= 1;
            }
            // SUBN Vx, Vy
            (8, _, _, 7) => {
                let vx = nibbles.1 as usize;
                let vy = nibbles.2 as usize;

                if self.registers[vy] > self.registers[vx] {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }

                self.registers[vx] = self.registers[vy] - self.registers[vx];
            }
            // SHL Vx {, Vy}
            (8, _, _, 0xE) => {
                let vx = nibbles.1 as usize;

                // Save MSB in VF
                self.registers[0xF] = (self.registers[vx] & 0x80) >> 7;

                self.registers[vx] <<= 1;
            }
            // SNE Vx, Vy
            (9, _, _, 0) => {
                let vx = nibbles.1 as usize;
                let vy = nibbles.2 as usize;

                if self.registers[vx] != self.registers[vy] {
                    self.pc += 2;
                }
            }
            // LD I, addr
            (0xA, _, _, _) => {
                let address = opcode & 0x0FFF;

                self.index = address;
            }
            // JP V0, addr
            (0xB, _, _, _) => {
                let address = opcode & 0x0FFF;

                self.pc = self.registers[0] as u16 + address;
            }
            // RND Vx, byte
            (0xC, _, _, _) => {
                let vx = nibbles.1 as usize;
                let byte = opcode & 0x00FF;
                let rng: u8 = rand::rng().random();

                self.registers[vx] = rng & byte as u8;
            }
            // DRW Vx, Vy, nibble
            (0xD, _, _, _) => {
                let x_coord = self.registers[nibbles.1 as usize] as u16;
                let y_coord = self.registers[nibbles.2 as usize] as u16;
                let num_rows = nibbles.3;

                let mut flipped = false;

                for y_line in 0..num_rows {
                    let addr = self.index + y_line as u16;
                    let pixels = self.memory[addr as usize];

                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % CHIP8_WIDTH;
                            let y = (y_coord + y_line) as usize % CHIP8_HEIGHT;

                            let idx = x + CHIP8_WIDTH * y;
                            flipped |= self.video[idx];
                            self.video[idx] ^= true;
                        }
                    }
                }
                if flipped {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
            }
            // SKP Vx
            (0xE, _, 9, 0xE) => {
                let vx = nibbles.1 as usize;
                let key = self.registers[vx];

                if self.keypad[key as usize] {
                    self.pc += 2;
                }
            }
            // SKNP Vx
            (0xE, _, 0xA, 1) => {
                let vx = nibbles.1 as usize;
                let key = self.registers[vx];

                if !self.keypad[key as usize] {
                    self.pc += 2;
                }
            }
            // LD Vx, DT
            (0xF, _, 0, 7) => {
                let vx = nibbles.1 as usize;

                self.registers[vx] = self.dtimer;
            }
            // LD Vx, K
            (0xF, _, 0, 0xA) => {
                let vx = nibbles.1 as usize;
                let mut pressed = false;

                for i in 0..self.keypad.len() {
                    if self.keypad[i] {
                        self.registers[vx] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    self.pc -= 2;
                }
            }
            // LD DT, Vx
            (0xF, _, 1, 5) => {
                let vx = nibbles.1 as usize;

                self.dtimer = self.registers[vx];
            }
            // LD ST, Vx
            (0xF, _, 1, 8) => {
                let vx = nibbles.1 as usize;

                self.stimer = self.registers[vx];
            }
            // ADD I, Vx
            (0xF, _, 1_, 0xE) => {
                let vx = nibbles.1 as usize;
                let x = self.registers[vx] as u16;

                self.index = self.index.wrapping_add(x);
            }
            // LD F, Vx
            (0xF, _, 2, 9) => {
                let vx = nibbles.1 as usize;
                let digit = self.registers[vx] as u16;

                self.index = FONTSET_START_ADDRESS as u16 + (5 * digit);
            }
            // LD B, Vx
            (0xF, _, 3, 3) => {
                let vx = nibbles.1 as usize;
                let value = self.registers[vx] as f32;

                let hundreds = (value / 100.0).floor() as u8;
                let tens = ((value / 10.0) % 10.0).floor() as u8;
                let ones = (value % 10.0) as u8;

                self.memory[self.index as usize] = hundreds;
                self.memory[(self.index + 1) as usize] = tens;
                self.memory[(self.index + 2) as usize] = ones;
            }
            // LD [I], Vx
            (0xF, _, 5, 5) => {
                let vx = nibbles.1 as usize;
                let i = self.index as usize;
                for idx in 0..=vx {
                    self.memory[i + idx] = self.registers[idx];
                }
            }
            // LD Vx, [I]
            (0xF, _, 6, 5) => {
                let vx = nibbles.1 as usize;
                let i = self.index as usize;
                for idx in 0..=vx {
                    self.registers[idx] = self.memory[i + idx];
                }
            }
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {:#04x}", opcode),
        }
    }
}
