# The Embedded OS
An experimental "from scratch" operating system project aimed at the RP2040/RP235x microcontroller.

## Goals
- Fully functional kernel - built on top of the embassy framework
- Provides a computer experience like the original MS-DOS
- Ability to load external programs from storage
- Can a full OS be written in Rust?

## Structure
- `kernel` - The Kernel
- `sdk` - Shared types and definitions for kernel + apps to link to.
- `libsys` - A friendly Rust interface for using the ABI
- `shell` - User control via CLI
- `apps/` - external apps

## Hardware Requirements
- RP2040 or RP235x microcontroller
- ST7920 display

## Contributing
This is just an personal experiment. Therefore this project is not open for contribution.

## License
Copyright (c) 2026 Leo Spratt. Licensed under [Apache License, Version 2.0](LICENSE-APACHE.txt).
