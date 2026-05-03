### CHIP-8-Emulator
CHIP 8 Emulator in Rust

#### Components
* Memory: DMA 4 KB
* Display: 64 x 32 pixels monochrome, 128 x 64 for SUPER-CHIP
* PC: Program Counter
* Index Register: 16-bit index register called "I" points to locations in memory
* Stack: 16-bit addresses, which is used to call subroutines/functions and return from them
* Delay timer: 8-bit, decremented at a rate of 60 Hz until it reaches 0
* Sound Timer: 8-bit, functions as a delay timer, also gives off a beeping sound as long as it's not 0
* Variable Registers: 16 x 8-bit, general purpose, numbered 0 through F hexadecimal

