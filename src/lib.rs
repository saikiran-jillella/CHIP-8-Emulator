use std::thread::sleep;
use std::time::{Duration, Instant};

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
    memory: [u8; MEMORY_SIZE],
    pc: u16,
    i: u16,
    v: [u8; 16],
    stack: Vec<u16>,
    clock_speed: u32,
}

impl Chip8 {
    pub fn new(clock_speed: u32) -> Self {
        let mut memory = [0u8; MEMORY_SIZE];
        memory[FONT_START..FONT_START + FONT.len()].copy_from_slice(&FONT);

        Self {
            memory,
            pc: PROGRAM_START as u16,
            i: 0,
            v: [0; 16],
            stack: Vec::new(),
            clock_speed,
        }
    }

    fn fetch(&mut self) -> u16 {
        (self.memory[self.pc as usize] as u16) << 8 | self.memory[(self.pc + 1) as usize] as u16
    }

    fn exec(&mut self, opcode: u16, display: &mut Display) {
        let x = ((opcode >> 8) & 0x0F) as usize;
        let y = ((opcode >> 4) & 0x0F) as usize;
        let n = (opcode & 0x0F) as usize;
        let nn = (opcode & 0xFF) as u8;
        let nnn = opcode & 0x0FFF;

        match opcode & 0xF000 {
            0x0000 => match opcode & 0x00FF {
                0xE0 => display.clear(),
                0xEE => self.pc = self.stack.pop().expect("stack underflow"),
                _ => {}
            },

            0x1000 => self.pc = nnn,

            0x2000 => {
                self.stack.push(self.pc);
                self.pc = nnn;
            }

            0x6000 => self.v[x] = nn,

            0x7000 => self.v[x] = self.v[x].wrapping_add(nn),

            0xA000 => self.i = nnn,

            0xD000 => {
                let vx = self.v[x] as usize;
                let vy = self.v[y] as usize;

                self.v[0xF] = 0;

                for row in 0..n {
                    let py = vy + row;
                    if py >= display.height() {
                        break;
                    }

                    let sprite = self.memory[self.i as usize + row];

                    for col in 0..8 {
                        let px = vx + col;
                        if px >= display.width() {
                            continue;
                        }

                        if (sprite >> (7 - col)) & 1 != 0 {
                            let prev = display.get(px, py);
                            if prev {
                                self.v[0xF] = 1;
                            }
                            display.set(px, py, !prev);
                        }
                    }
                }

                display.draw();
            }

            _ => {}
        }
    }

    pub fn run(&mut self, program: &[u8], display: &mut Display) {
        self.memory[PROGRAM_START..PROGRAM_START + program.len()]
            .copy_from_slice(program);

        let cycle = Duration::from_secs_f64(1.0 / self.clock_speed as f64);

        loop {
            let start = Instant::now();

            let opcode = self.fetch();
            self.pc += 2;

            self.exec(opcode, display);

            if let Some(remaining) = cycle.checked_sub(start.elapsed()) {
                sleep(remaining);
            }
        }
    }
}
