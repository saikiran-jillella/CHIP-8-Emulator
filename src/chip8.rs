use rand::RngExt;

use crate::{
    Display,
    chip8::Mode::{Standard, Super},
};

const MEMORY_SIZE: usize = 4096;
const PROGRAM_START: usize = 0x200;
const FONT_START: usize = 0x50;

const FONT: [u8; 80] = [
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
    keys: [bool; 16],
    display: Display,
    memory: [u8; MEMORY_SIZE],
    pc: u16,
    i: u16,
    v: [u8; 16],
    stack: Vec<u16>,
    mode: Mode,
    delay_timer: u8,
    sound_timer: u8,
    waiting_for_key: Option<usize>,
}

#[derive(PartialEq)]
pub enum Mode {
    Standard,
    Super,
}

impl Chip8 {
    pub fn render(&self) {
        self.display.draw();
    }
    pub fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn sound_timer(&self) -> u8 {
        self.sound_timer
    }

    pub fn key_down(&mut self, key: usize) {
        self.keys[key] = true;
    }
    pub fn key_up(&mut self, key: usize) {
        self.keys[key] = false;
    }

    pub fn new(super_mode: bool) -> Self {
        let mut memory = [0u8; MEMORY_SIZE];
        memory[FONT_START..FONT_START + FONT.len()].copy_from_slice(&FONT);

        Self {
            keys: [false; 16],
            display: Display::new(super_mode),
            memory,
            pc: PROGRAM_START as u16,
            i: 0,
            v: [0; 16],
            stack: Vec::new(),
            mode: if super_mode { Super } else { Standard },
            delay_timer: 0,
            sound_timer: 0,
            waiting_for_key: None,
        }
    }

    pub fn fetch(&mut self) -> u16 {
        (self.memory[self.pc as usize] as u16) << 8 | self.memory[(self.pc + 1) as usize] as u16
    }

    pub fn exec(&mut self, opcode: u16) {
        let x = ((opcode >> 8) & 0x0F) as usize;
        let y = ((opcode >> 4) & 0x0F) as usize;
        let n = (opcode & 0x0F) as usize;
        let nn = (opcode & 0xFF) as u8;
        let nnn = opcode & 0x0FFF;

        match opcode & 0xF000 {
            0x0000 => match opcode & 0x00FF {
                // Clear screen
                0xE0 => self.display.clear(),
                // Return from Subroutine
                0xEE => self.pc = self.stack.pop().expect("stack underflow"),
                _ => {}
            },

            // Jump
            0x1000 => self.pc = nnn,

            // call Subroutine
            0x2000 => {
                self.stack.push(self.pc);
                self.pc = nnn;
            }

            // Jump
            0x3000 => {
                if self.v[x] == nn {
                    self.pc += 2;
                }
            }

            // Jump
            0x4000 => {
                if self.v[x] != nn {
                    self.pc += 2;
                }
            }

            // Jump
            0x5000 => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }

            // Set VX Register
            0x6000 => self.v[x] = nn,

            // Add NN to VX Register
            0x7000 => self.v[x] = self.v[x].wrapping_add(nn),

            // Logical & Arithmetic Instructions
            0x8000 => match opcode & 0x000F {
                0x0 => self.v[x] = self.v[y],
                0x1 => self.v[x] |= self.v[y],
                0x2 => self.v[x] &= self.v[y],
                0x3 => self.v[x] ^= self.v[y],
                0x4 => {
                    let (result, overflowed) = self.v[x].overflowing_add(self.v[y]);
                    self.v[x] = result;
                    self.v[0xF] = overflowed as u8;
                }
                0x5 => {
                    let (result, overflowed) = self.v[x].overflowing_sub(self.v[y]);
                    self.v[x] = result;
                    self.v[0xF] = (!overflowed) as u8;
                }
                0x6 => {
                    if self.mode == Mode::Standard {
                        self.v[x] = self.v[y]
                    }
                    self.v[0xF] = (self.v[x] >> 7) & 1;
                    self.v[x] <<= 1;
                }
                0x7 => {
                    let (result, overflowed) = self.v[y].overflowing_sub(self.v[x]);
                    self.v[x] = result;
                    self.v[0xF] = (!overflowed) as u8;
                }
                0xE => {
                    if self.mode == Mode::Standard {
                        self.v[x] = self.v[y]
                    }
                    self.v[0xF] = (self.v[x]) & 1;
                    self.v[x] >>= 1;
                }
                _ => {}
            },

            //Jump
            0x9000 => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }

            // Set Index Register
            0xA000 => self.i = nnn,

            // Jump with Offset
            0xB000 => {
                self.pc = nnn + if self.mode == Mode::Super { self.v[x] as u16 } else { self.v[0x0] as u16 };
            }

            // Random
            0xC000 => {
                self.v[x] = nn & rand::rng().random::<u8>();
            }

            // Display
            0xD000 => {
                let vx = (self.v[x] as usize) % self.display.width();
                let vy = (self.v[y] as usize) % self.display.height();

                self.v[0xF] = 0;

                for row in 0..n {
                    let py = vy + row;
                    if py >= self.display.height() {
                        break;
                    }

                    let sprite = self.memory[self.i as usize + row];

                    for col in 0..8 {
                        let px = vx + col;
                        if px >= self.display.width() {
                            continue;
                        }

                        if (sprite >> (7 - col)) & 1 != 0 {
                            let prev = self.display.get(px, py);
                            if prev {
                                self.v[0xF] = 1;
                            }
                            self.display.set(px, py, !prev);
                        }
                    }
                }
            }

            // Skip if key
            0xE000 => match opcode & 0x00FF {
                0x9E if self.keys[self.v[x] as usize] => {
                    self.pc += 2;
                }
                0xA1 if !self.keys[self.v[x] as usize] => {
                    self.pc += 2;
                }
                _ => {}
            },

            0xF000 => {
                // Timers
                match opcode & 0x00FF {
                    0x07 => self.v[x] = self.delay_timer,
                    0x15 => self.delay_timer = self.v[x],
                    0x18 => self.sound_timer = self.v[x],

                    // Add to index
                    0x1E => {
                        let (result, overflow) = self.i.overflowing_add(self.v[x] as u16);
                        self.i = result;
                        self.v[0xF] = overflow as u8;
                    }

                    // Get key
                    0x0A => match self.waiting_for_key {
                        Some(key) if !self.keys[key] => {
                            self.v[x] = key as u8;
                            self.waiting_for_key = None;
                        }

                        Some(_) => {
                            self.pc -= 2;
                        }

                        None => {
                            if let Some(key) = self.keys.iter().position(|&pressed| pressed) {
                                self.waiting_for_key = Some(key);
                            }

                            self.pc -= 2;
                        }
                    },

                    // Font Character
                    0x29 => {
                        let digit = (self.v[x] & 0x0F) as u16;
                        self.i = FONT_START as u16 + digit * 5;
                    }

                    // Decimal Conversion
                    0x33 => {
                        let vx = self.v[x];
                        let addr = self.i as usize;
                        self.memory[addr] = vx / 100;
                        self.memory[addr + 1] = (vx / 10) % 10;
                        self.memory[addr + 2] = vx % 10;
                    }

                    // Store
                    0x55 => {
                        let addr = self.i as usize;
                        self.memory[addr..=addr + x].copy_from_slice(&self.v[0..=x]);
                        if self.mode == Standard {
                            self.i += x as u16 + 1;
                        }
                    }

                    // Load
                    0x65 => {
                        let addr = self.i as usize;
                        self.v[0..=x].copy_from_slice(&self.memory[addr..=addr + x]);
                        if self.mode == Standard {
                            self.i += x as u16 + 1;
                        }
                    }
                    _ => {}
                }
            }

            _ => {}
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.memory[PROGRAM_START..PROGRAM_START + rom.len()].copy_from_slice(rom);
    }

    pub fn cycle(&mut self) {
        let opcode = self.fetch();
        self.pc += 2;
        self.exec(opcode);
    }
}
