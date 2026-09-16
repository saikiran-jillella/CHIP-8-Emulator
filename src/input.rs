use crate::Chip8;

pub trait Input {
    fn update(&mut self, chip8: &mut Chip8);
}
