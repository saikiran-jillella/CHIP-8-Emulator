# CHIP-8 Emulator

A CHIP-8 and SUPER-CHIP emulator written in Rust that runs directly inside your terminal, rendering games with ANSI block characters in a retro amber palette.

## Features

- **Full CHIP-8 Instruction Set**: Complete opcode execution, stack, memory, and registers.
- **Super CHIP-8 Support**: 128×64 high-resolution display and SCHIP quirks with the `-s` / `--super` flag.
- **Terminal Rendering**: Direct terminal display using ANSI escape codes and block characters.
- **Audio Support**: Simple beep playback driven by the sound timer via `rodio`.
- **Linux Input**: Low-latency keyboard events via `evdev` with automatic device detection via `udev`.
- **Timing**: 700 Hz CPU clock with 60 Hz delay/sound timers and display refreshes.

## Controls

The classic 16-key CHIP-8 hex keypad is mapped to standard QWERTY keys:

```text
CHIP-8 Keypad            QWERTY Keyboard
+-+-+-+-+                +-+-+-+-+
| 1 | 2 | 3 | C |        | 1 | 2 | 3 | 4 |
+-+-+-+-+                +-+-+-+-+
| 4 | 5 | 6 | D |  --->  | Q | W | E | R |
+-+-+-+-+                +-+-+-+-+
| 7 | 8 | 9 | E |        | A | S | D | F |
+-+-+-+-+                +-+-+-+-+
| A | 0 | B | F |        | Z | X | C | V |
+-+-+-+-+                +-+-+-+-+
```

## Getting Started

### Prerequisites

- Rust (edition 2024 / recent stable toolchain)
- Linux with `evdev` support

> [!NOTE]
> `evdev` requires read access to `/dev/input/event*`. If you hit a permission error, either run the binary with `sudo`:
> ```bash
> sudo ./target/release/chip_8_emulator path/to/game.ch8
> ```
> or add your user to the `input` group:
> ```bash
> sudo usermod -aG input $USER
> ```
> *(Log out and back in for group changes to take effect.)*

### Build

```bash
cargo build --release
```

The compiled binary will be located at `./target/release/chip_8_emulator`.

### Run

Pass the path to your ROM file as the first argument:

```bash
# Standard CHIP-8 mode (64x32)
./target/release/chip_8_emulator path/to/game.ch8

# Super CHIP-8 mode (128x64)
./target/release/chip_8_emulator path/to/game.sc8 -s

# Or directly via Cargo
cargo run --release -- path/to/game.ch8
```

## Reference

Built following Tobias V. Langhoff's [Guide to making a CHIP-8 emulator](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/).

## License

This project is licensed under the [MIT License](LICENSE) © 2026 [Saikiran Jillella](https://github.com/saikiran-jillella).
