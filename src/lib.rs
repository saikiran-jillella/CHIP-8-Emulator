use std::thread::sleep;
use std::time::{Duration, Instant};

use rand::RngExt;

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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub enum Display {
    Standard([[bool; 64]; 32]),
    Super([[bool; 128]; 64]),
}

impl Display {
    pub fn new(super_mode: bool) -> Self {
        match super_mode {
            false => Self::Standard([[false; 64]; 32]),
            true => Self::Super([[false; 128]; 64]),
        }
    }

    #[inline]
    pub fn width(&self) -> usize {
        match self {
            Self::Standard(_) => 64,
            Self::Super(_) => 128,
        }
    }

    #[inline]
    pub fn height(&self) -> usize {
        match self {
            Self::Standard(_) => 32,
            Self::Super(_) => 64,
        }
    }

    pub fn clear(&mut self) {
        match self {
            Self::Standard(p) => *p = [[false; 64]; 32],
            Self::Super(p) => *p = [[false; 128]; 64],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        match self {
            Self::Standard(p) => p[y][x],
            Self::Super(p) => p[y][x],
        }
    }

    pub fn set(&mut self, x: usize, y: usize, v: bool) {
        match self {
            Self::Standard(p) => p[y][x] = v,
            Self::Super(p) => p[y][x] = v,
        }
    }

    pub fn draw(&self) {
        print!("\x1B[2J\x1B[H");
        for y in 0..self.height() {
            for x in 0..self.width() {
                print!("{}", if self.get(x, y) { '█' } else { ' ' });
            }
            println!();
        }
    }
}

pub struct Chip8 {
    display: Display,
    memory: [u8; MEMORY_SIZE],
    pc: u16,
    i: u16,
    v: [u8; 16],
    stack: Vec<u16>,
    clock_speed: u32,
    mode: Mode,
}

#[derive(PartialEq)]
pub enum Mode {
    Standard,
    Super,
}

impl Chip8 {
    pub fn new(clock_speed: u32, super_mode: bool) -> Self {

        let mut memory = [0u8; MEMORY_SIZE];
        memory[FONT_START..FONT_START + FONT.len()].copy_from_slice(&FONT);

        Self {
            display: Display::new(super_mode),
            memory,
            pc: PROGRAM_START as u16,
            i: 0,
            v: [0; 16],
            stack: Vec::new(),
            clock_speed,
            mode: if super_mode { Mode::Super } else { Mode::Standard },
        }
    }

    fn fetch(&mut self) -> u16 {
        (self.memory[self.pc as usize] as u16) << 8 | self.memory[(self.pc + 1) as usize] as u16
    }

    fn exec(&mut self, opcode: u16) {
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
            },

            // Jump
            0x3000 => {
                if self.v[x] == nn {
                    self.pc += 2;
                }
            },

            // Jump
            0x4000 => {
                if self.v[x] != nn {
                    self.pc += 2;
                }
            },

            // Jump
            0x5000 => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            },

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
                },
                0x5 => {
                    let (result, overflowed) = self.v[x].overflowing_sub(self.v[y]); 
                    self.v[x] = result;
                    self.v[0xF] = (!overflowed) as u8;
                },
                0x6 => {
                    if self.mode == Mode::Standard {
                        self.v[x] = self.v[y]
                    }
                    self.v[0xF] = (self.v[x] >> 7) & 1;
                    self.v[x] <<= 1;
                },
                0x7 => {
                    let (result, overflowed) = self.v[y].overflowing_sub(self.v[x]); 
                    self.v[x] = result;
                    self.v[0xF] = (!overflowed) as u8;
                },
                0xE => {
                    if self.mode == Mode::Standard {
                        self.v[x] = self.v[y]
                    }
                    self.v[0xF] = (self.v[x]) & 1;
                    self.v[x] >>= 1;
                },
                _ => {}
            }

            //Jump
            0x9000 => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            },


            // Set Index Register
            0xA000 => self.i = nnn,

            // Jump with Offset
            0xB000 => {
                self.pc = nnn + if self.mode == Mode::Super { self.v[x] as u16 } else { self.v[0x0] as u16 };
            },

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

                self.display.draw();
            }

            _ => {}
        }
    }

    pub fn run(&mut self, program: &[u8]) {
        self.memory[PROGRAM_START..PROGRAM_START + program.len()]
            .copy_from_slice(program);

        let cycle = Duration::from_secs_f64(1.0 / self.clock_speed as f64);

        loop {
            let start = Instant::now();

            let opcode = self.fetch();
            self.pc += 2;

            self.exec(opcode);

            if let Some(remaining) = cycle.checked_sub(start.elapsed()) {
                sleep(remaining);
            }
        }
    }
}
