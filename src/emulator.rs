use crate::{Chip8, Input};
use std::{
    thread::sleep,
    time::{Duration, Instant},
};

pub struct Emulator {
    chip8: Chip8,
    cycle: Duration,
    frame_period: Duration,
    timer_tick: Duration,
}

impl Emulator {
    pub fn new(clock_speed: u64, fps: u64, timer_duration: u64, super_mode: bool) -> Self {
        Self {
            chip8: Chip8::new(super_mode),
            cycle: Duration::from_secs_f64(1.0 / clock_speed as f64),
            frame_period: Duration::from_secs_f64(1.0 / fps as f64),
            timer_tick: Duration::from_secs_f64(1.0 / timer_duration as f64),
        }
    }

    pub fn run<I: Input>(mut self, rom: &[u8], input: &mut I) {
        print!("\x1B[2J\x1B[H");

        self.chip8.load_rom(rom);

        let mut last = Instant::now();
        let mut cpu_accumulator = Duration::ZERO;
        let mut timer_accumulator = Duration::ZERO;
        let mut frame_accumulator = Duration::ZERO;

        loop {
            let now = Instant::now();
            let elapsed = now - last;
            last = now;

            cpu_accumulator += elapsed;
            timer_accumulator += elapsed;
            frame_accumulator += elapsed;

            input.update(&mut self.chip8);

            while cpu_accumulator >= self.cycle {
                self.chip8.cycle();
                cpu_accumulator -= self.cycle;
            }

            while timer_accumulator >= self.timer_tick {
                if self.chip8.sound_timer() > 0 {
                    print!("\x07");
                }

                self.chip8.decrement_timers();
                timer_accumulator -= self.timer_tick;
            }

            if frame_accumulator >= self.frame_period {
                self.chip8.render();
                frame_accumulator -= self.frame_period;
            }

            let until_cpu = self.cycle - cpu_accumulator;
            let until_timer = self.timer_tick - timer_accumulator;
            let until_frame = self.frame_period - frame_accumulator;

            sleep(until_cpu.min(until_timer).min(until_frame));
        }
    }
}
