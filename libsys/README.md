# Libsys
A system library wrapper around the Kernel ABI definitions.

## Usage

```rs
// file: src/main.rs
#![no_std]
#![no_main]

#[libsys::entrypoint]
fn main() {
    let _ = libsys::core::get_abi_version();
}
```
