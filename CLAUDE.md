# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build & Flash
Each project is independent. Build from the project directory:

```bash
cd blink-rp2350 && cargo build --release
cd temp-humidity-rp2350 && cargo build --release
cd weather-nrf52840 && cargo build --release
```

### UF2 Conversion (for Pico boards)
```bash
picotool uf2 convert target/thumbv8m.main-none-eabihf/release/blink-rp2350 -t elf blink-rp2350.uf2 --family rp2350-arm-s
```

## Architecture Overview

This repository contains multiple independent embedded Rust projects using the Embassy async framework. Each project targets a specific microcontroller and is built independently (no Cargo workspace).

### Repository Structure

```
ponix-end-devices/
├── blink-rp2350/         # Pi Pico 2 W - onboard LED blink (CYW43)
├── temp-humidity-rp2350/ # Pi Pico 2 W - temp/humidity sensor
├── weather-nrf52840/     # RAK4631/nRF52840 - LoRaWAN weather sensor
└── .mise.toml            # Task runner configuration
```

### Naming Convention
Projects follow `feature-arch` pattern:
- `blink-rp2350` - blink feature on RP2350
- `temp-humidity-rp2350` - temp/humidity feature on RP2350
- `weather-nrf52840` - weather feature on nRF52840

### Project Details

#### blink-rp2350
- **Board**: Raspberry Pi Pico 2 W
- **Target**: `thumbv8m.main-none-eabihf` (Cortex-M33)
- **Features**: Onboard LED via CYW43 WiFi chip
- **Embassy**: Git version (for latest CYW43 API)

#### temp-humidity-rp2350
- **Board**: Raspberry Pi Pico 2 W
- **Target**: `thumbv8m.main-none-eabihf` (Cortex-M33)
- **Embassy**: Published crates.io versions

#### weather-nrf52840
- **Board**: RAK4631 WisBlock
- **Target**: `thumbv7em-none-eabi` (Cortex-M4F)
- **Features**: LoRaWAN (US915), SX1262 radio
- **Embassy**: Published crates.io versions (compatible with lora-rs)

### Key Components

1. **Embassy Framework** - Async embedded runtime
   - `embassy-executor` - Task executor
   - `embassy-nrf` - nRF52840 HAL
   - `embassy-rp` - RP2350 HAL
   - `embassy-time` - Async timers

2. **Per-Project Configuration**
   - Each project has its own `Cargo.toml` with all dependencies
   - Each project has its own `.cargo/config.toml` with target and runner
   - Each project has its own `memory.x` linker script
   - Each project has its own `Cargo.lock`

3. **Logging**
   - Uses `defmt` for efficient embedded logging
   - RTT (Real-Time Transfer) for debug output

### Key Development Patterns

1. **No Standard Library** - Uses `#![no_std]` and `#![no_main]`
2. **Async/Await** - Embassy provides async runtime for embedded
3. **Independent Projects** - Each project manages its own dependencies
4. **Per-Project Targets** - Each project builds for its specific MCU
5. **Static Memory** - No heap allocation, uses `static_cell` for static allocations
