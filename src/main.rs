use chip_8_emulator::{DefaultSpeaker, Emulator, Keyboard};

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    let rom = fs::read(&args[1]).expect("ROM missing");
    let super_mode = args.iter().any(|a| a == "-s" || a == "--super");

    let mut keyboard: Keyboard = Keyboard::new();
    let mut speaker: DefaultSpeaker = DefaultSpeaker::new().unwrap();
    let emulator: Emulator = Emulator::new(700, 200, 60, super_mode);

    emulator.run(&rom, &mut keyboard, &mut speaker);
}
