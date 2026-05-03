use chip_8_emulator::Memory;

fn main() {

    let memory = Memory::new();
    println!("{}", memory.access(0));
}
