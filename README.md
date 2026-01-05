# ponix-end-devices

Embedded Rust projects for ponix end devices using the Embassy async framework.

## Projects

```
├── blink-rp2350/         # Pico 2 W - onboard LED blink
├── temp-humidity-rp2350/ # Pico 2 W - temp/humidity sensor
└── weather-nrf52840/     # RAK4631 - LoRaWAN weather sensor
```

## Commands

```bash
mise run build <project>   # Build in release mode
mise run flash <project>   # Build and flash via probe-rs
mise run uf2 <project>     # Build and create UF2 for drag-and-drop
mise run check <project>   # Check without building
mise run clippy <project>  # Run linter
```

## Example

```bash
mise run uf2 blink-rp2350
# Creates blink-rp2350/blink-rp2350.uf2
```
