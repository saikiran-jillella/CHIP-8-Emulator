use std::env;
use std::fs;

use chip_8_emulator::{Chip8, Display};

fn main() {
    let args: Vec<String> = env::args().collect();

    let rom = fs::read(&args[1]).expect("ROM missing");
    let super_mode = args.iter().any(|a| a == "-s" || a == "--super");

    let mut display = Display::new(super_mode);
    let mut chip8 = Chip8::new(150);

    chip8.run(&rom, &mut display);
}
