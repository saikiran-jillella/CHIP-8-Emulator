use std::io::{self, Write};

pub struct Display {
    pixels: [[bool; 128]; 64],
    width: usize,
    height: usize,
}

impl Display {
    pub fn new(super_mode: bool) -> Self {
        let (width, height) = if super_mode { (128, 64) } else { (64, 32) };

        Self { pixels: [[false; 128]; 64], width, height }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn clear(&mut self) {
        self.pixels = [[false; 128]; 64];
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        self.pixels[y][x]
    }

    pub fn set(&mut self, x: usize, y: usize, v: bool) {
        self.pixels[y][x] = v;
    }

    //   pub fn draw(&self) {
    //      print!("\x1B[2J\x1B[H");
    //       for y in 0..self.height {
    //          for x in 0..self.width {
    //               print!("{}", if self.get(x, y) { "██" } else { "  " });
    //           }
    //           println!();
    //       }
    //  }

    pub fn draw(&self) {
        const FG: &str = "\x1B[38;2;255;204;0m";
        const BG: &str = "\x1B[48;2;153;102;0m";
        const RESET: &str = "\x1B[0m";

        let mut out = io::stdout().lock();

        write!(out, "\x1B[H").unwrap();

        for y in 0..self.height {
            for x in 0..self.width {
                if self.get(x, y) {
                    write!(out, "{FG}{BG}██{RESET}").unwrap();
                } else {
                    write!(out, "{BG}  {RESET}").unwrap();
                }
            }
            write!(out, "\r\n").unwrap();
        }

        out.flush().unwrap();
    }
}
