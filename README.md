# NtrixOS Embedded
An experimental "from scratch" operating system project aimed at the RP2040/RP235x microcontroller.

## Goals
- Run on real hardware, no VM/emulation
- Provide a single user, single process OS
- Inspired by DOS and other early OS (terminal based)
- Can a full OS be written in Rust?
- Ability to load external programs
- Create kernel on top of embassy framework
    - Removes the need to write a HAL
    - Can use pre-built state machine for kernel

## Structure
- `kernel` - The Kernel
- `sdk` - Shared types and definitions for kernel + apps to link to.
- `libsys` - A friendly Rust interface for using the ABI
- `libsys-macros` - Proc macros used by `libsys`
- `shell` - User control via CLI
- `apps/` - external apps

## Hardware Requirements
- RP2040 or RP235x microcontroller
- A [NtrixVDC](https://github.com/NtrixOS/ntrix-vdc), or protocol compatible alternative

## Contributing
This is just an personal experiment. Therefore this project is not open for contribution.

## License
Copyright (c) 2026 Leo Spratt. Licensed under [Apache License, Version 2.0](LICENSE-APACHE.txt).
