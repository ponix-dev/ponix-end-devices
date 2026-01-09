# blink-rp2350

Onboard LED blink example for Raspberry Pi Pico 2 W. The LED is controlled via the CYW43 WiFi chip (not a standard GPIO).

## Build & Run

Uses probe-rs for flashing and debugging.

### Using mise (from repo root)

```bash
mise run build blink-rp2350
mise run flash blink-rp2350
```

### Using cargo (from crate directory)

```bash
cargo build --release
cargo run --release
```

`cargo run` / `mise run flash` flashes via probe-rs and streams defmt logs over RTT.

### UF2 Flashing (alternative)

If probe-rs is unavailable:

```bash
mise run uf2 blink-rp2350
```

Or manually:

```bash
picotool uf2 convert target/thumbv8m.main-none-eabihf/release/blink-rp2350 -t elf blink-rp2350.uf2 --family rp2350-arm-s
```

Then copy the `.uf2` file to the Pico in BOOTSEL mode.

## Hardware

- **Board**: Raspberry Pi Pico 2 W (not compatible with non-W variant)
- **MCU**: RP2350 (Cortex-M33)
- **Target**: `thumbv8m.main-none-eabihf`
- **LED**: Onboard LED connected to CYW43 GPIO 0

The Pico 2 W's onboard LED is not directly connected to an RP2350 GPIO. Instead, it's connected to GPIO 0 of the CYW43 WiFi chip, which communicates with the RP2350 via PIO/SPI.

## Architecture

### Embassy Git Dependencies

Uses Embassy from git (latest main branch). All Embassy crates track the same revision.

### CYW43 Firmware

The `firmware/` directory contains required CYW43 WiFi chip firmware:
- `43439A0.bin` - Main firmware blob
- `43439A0_clm.bin` - Country locale matrix

These are included at compile time via `include_bytes!()`.

**Development tip**: For faster iteration, firmware can be flashed separately at fixed addresses:
```bash
probe-rs download firmware/43439A0.bin --binary-format bin --chip RP235x --base-address 0x10100000
probe-rs download firmware/43439A0_clm.bin --binary-format bin --chip RP235x --base-address 0x10140000
```

Then use `core::slice::from_raw_parts()` to reference them (see commented code in main.rs).

### Main Loop

1. Initialize CYW43 WiFi chip via PIO/SPI
2. Spawn background CYW43 runner task
3. Toggle LED via `control.gpio_set(0, true/false)` every 250ms

### Picotool Metadata

Includes binary info entries for `picotool info` to display program name, description, and version.

### Key Files

- `src/main.rs` - Application entry point and blink logic
- `memory.x` - RP2350 linker script (2MB flash, 512KB RAM)
- `build.rs` - Copies memory.x to output, sets linker args
- `.cargo/config.toml` - Target and probe-rs runner config
- `firmware/` - CYW43 firmware blobs

## Debugging

Uses defmt with RTT. Default log level is `debug` (set in `.cargo/config.toml`).

Logs will show "led on!" and "led off!" messages as the LED toggles.
