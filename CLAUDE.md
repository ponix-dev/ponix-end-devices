# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build & Flash
- `just build` - Build the project in release mode
- `just flash` - Build and flash to device (runs `cargo run --release`)
- `cargo build` - Build in debug mode
- `cargo build --release` - Build in release mode
- `cargo run` - Build and flash debug build using probe-rs
- `cargo run --release` - Build and flash release build using probe-rs

### Development
- `cargo check` - Check code without building
- `cargo clippy` - Run linting
- `DEFMT_LOG=trace cargo run` - Run with trace-level logging (already set in .cargo/config.toml)

## Architecture Overview

This is an embedded Rust project for nRF52840 microcontrollers using the Embassy async framework. The codebase follows a no_std embedded architecture pattern.

### Key Components

1. **Embassy Framework** - Async embedded runtime and HAL
   - `embassy-executor` - Async task executor
   - `embassy-nrf` - Hardware abstraction for nRF52840
   - `embassy-time` - Async timers and delays
   - `embassy-net` - Networking stack (TCP/IP, DHCP)
   - `embassy-usb` - USB device support

2. **Memory Configuration**
   - Flash: 1MB starting at 0x00000000
   - RAM: 256KB starting at 0x20000000
   - Configured in `memory.x` (copied by build.rs)
   - Alternative configuration for SoftDevice S140 included but commented out

3. **Target Configuration**
   - Target: `thumbv7em-none-eabi` (Cortex-M4F)
   - Chip: nRF52840_xxAA
   - Runner: probe-rs for flashing and debugging
   - Configured in `.cargo/config.toml`

4. **Logging**
   - Uses `defmt` for efficient embedded logging
   - RTT (Real-Time Transfer) for debug output
   - Default log level: trace

### Project Structure

- `src/main.rs` - Entry point with async main task
- `memory.x` - Memory layout definition
- `build.rs` - Build script to handle memory.x and linker configuration
- `.cargo/config.toml` - Cargo configuration for target and runner
- `justfile` - Common commands shortcuts

### Key Development Patterns

1. **No Standard Library** - Uses `#![no_std]` and `#![no_main]`
2. **Async/Await** - Embassy provides async runtime for embedded
3. **Hardware Abstraction** - Embassy HAL provides safe abstractions over nRF peripherals
4. **Static Memory** - No heap allocation, uses `static_cell` for static allocations
5. **Panic Handling** - Uses `panic-probe` for debugging panics over RTT