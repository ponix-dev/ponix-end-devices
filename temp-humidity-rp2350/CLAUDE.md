# temp-humidity-rp2350

Temperature and humidity sensor project for Raspberry Pi Pico 2 W using the AHT20 sensor with WiFi connectivity.

## Build & Run

WiFi credentials are required at build time (set in `.env` or inline). Uses probe-rs for flashing and debugging.

### Using mise (from repo root)

```bash
mise run build temp-humidity-rp2350
mise run flash temp-humidity-rp2350
```

### Using cargo (from crate directory)

```bash
WIFI_SSID="your_network" WIFI_PASSWORD="your_password" cargo build --release
WIFI_SSID="your_network" WIFI_PASSWORD="your_password" cargo run --release
```

`cargo run` / `mise run flash` flashes via probe-rs and streams defmt logs over RTT.

### UF2 Flashing (alternative)

If probe-rs is unavailable:

```bash
mise run uf2 temp-humidity-rp2350
```

Or manually:

```bash
picotool uf2 convert target/thumbv8m.main-none-eabihf/release/temp-humidity-rp2350 -t elf temp-humidity-rp2350.uf2 --family rp2350-arm-s
```

Then copy the `.uf2` file to the Pico in BOOTSEL mode.

## Hardware

- **Board**: Raspberry Pi Pico 2 W
- **MCU**: RP2350 (Cortex-M33)
- **Target**: `thumbv8m.main-none-eabihf`
- **Sensor**: AHT20 (I2C temperature/humidity sensor)
- **WiFi**: CYW43 onboard

### Pin Connections

| Function | Pin |
|----------|-----|
| I2C0 SDA | GPIO 4 |
| I2C0 SCL | GPIO 5 |

CYW43 WiFi uses dedicated pins (23, 24, 25, 29) managed internally.

## Architecture

### Embassy Git Dependencies

This project uses Embassy from git (not crates.io) pinned to commit `a6705c550836fc23ef52dcbbe6c26c9fd6f45c98`. All Embassy crates must use the same revision.

### CYW43 Firmware

The `firmware/` directory contains required CYW43 WiFi chip firmware:
- `43439A0.bin` - Main firmware blob
- `43439A0_clm.bin` - Country locale matrix

These are included at compile time via `include_bytes!()`.

### Main Components

1. **CYW43 WiFi initialization** - Must initialize first, spawns background task
2. **I2C + AHT20 sensor** - Async I2C on I2C0
3. **Network stack** - DHCPv4 configuration
4. **Sensor loop** - Reads every 2 seconds, logs temperature (Fahrenheit) and humidity

### Key Files

- `src/main.rs` - Application entry point and all logic
- `memory.x` - RP2350 linker script (2MB flash, 512KB RAM)
- `build.rs` - Copies memory.x to output, sets linker args
- `.cargo/config.toml` - Target and probe-rs runner config
- `firmware/` - CYW43 firmware blobs

## Debugging

Uses defmt with RTT. Default log level is `debug` (set in `.cargo/config.toml`).

```bash
WIFI_SSID="test" WIFI_PASSWORD="test" cargo run --release
```

probe-rs will display defmt logs automatically.
