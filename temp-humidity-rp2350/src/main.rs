#![no_std]
#![no_main]

use aligned::{Aligned, A4};
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use defmt::*;
use embassy_executor::Spawner;
use embassy_net::{Config, StackResources};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::i2c::{self, Config as I2cConfig, I2c};
use embassy_rp::peripherals::{DMA_CH0, I2C0, PIO0};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_time::{Delay, Timer};
use embedded_aht20::Aht20;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// WiFi credentials (set via environment variables at build time)
const WIFI_NETWORK: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

bind_interrupts!(struct PioIrqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

bind_interrupts!(struct I2cIrqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, cyw43::SpiBus<Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Temperature/Humidity sensor with WiFi starting...");

    // ========================================
    // Initialize CYW43 WiFi chip FIRST
    // ========================================
    static FW: &Aligned<A4, [u8]> = &Aligned(*include_bytes!("../firmware/43439A0.bin"));
    static CLM: &Aligned<A4, [u8]> = &Aligned(*include_bytes!("../firmware/43439A0_clm.bin"));

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, PioIrqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        RM2_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, FW).await;
    spawner.spawn(unwrap!(cyw43_task(runner)));

    control.init(CLM).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;
    info!("CYW43 WiFi chip initialized!");

    // ========================================
    // Initialize I2C and AHT20 sensor
    // ========================================
    let i2c = I2c::new_async(p.I2C0, p.PIN_5, p.PIN_4, I2cIrqs, I2cConfig::default());

    let mut sensor = match Aht20::new(i2c, embedded_aht20::DEFAULT_I2C_ADDRESS, Delay).await {
        Ok(s) => {
            info!("AHT20 sensor initialized!");
            s
        }
        Err(e) => {
            match e {
                embedded_aht20::Error::I2c(i2c_err) => {
                    error!("Failed to initialize AHT20: I2C error: {:?}", i2c_err);
                }
                embedded_aht20::Error::InvalidCrc => {
                    error!("Failed to initialize AHT20: Invalid CRC");
                }
                embedded_aht20::Error::UnexpectedBusy => {
                    error!("Failed to initialize AHT20: Device unexpectedly busy");
                }
            }
            loop {
                Timer::after_secs(1).await;
            }
        }
    };

    // ========================================
    // Initialize Network Stack
    // ========================================
    let config = Config::dhcpv4(Default::default());

    // Seed for random number generation
    let seed = 0x0123_4567_89ab_cdef_u64;

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(
        net_device,
        config,
        RESOURCES.init(StackResources::new()),
        seed,
    );
    spawner.spawn(unwrap!(net_task(runner)));

    // ========================================
    // Connect to WiFi
    // ========================================
    info!("Connecting to WiFi network: {}", WIFI_NETWORK);

    loop {
        match control
            .join(WIFI_NETWORK, cyw43::JoinOptions::new(WIFI_PASSWORD.as_bytes()))
            .await
        {
            Ok(_) => {
                info!("WiFi connected!");
                break;
            }
            Err(e) => {
                warn!("WiFi join failed: {:?}, retrying in 5s...", e);
                Timer::after_secs(5).await;
            }
        }
    }

    // Wait for DHCP to assign an IP address
    info!("Waiting for DHCP...");
    stack.wait_config_up().await;

    if let Some(config) = stack.config_v4() {
        info!("IP address: {}", config.address);
        info!("Gateway: {:?}", config.gateway);
    }
    info!("Network ready!");

    // ========================================
    // Main sensor loop
    // ========================================
    loop {
        match sensor.measure().await {
            Ok(measurement) => {
                let fahrenheit = measurement.temperature.celsius() * 9.0 / 5.0 + 32.0;
                info!(
                    "Temperature: {} F, Humidity: {} %",
                    fahrenheit,
                    measurement.relative_humidity
                );
            }
            Err(_e) => {
                error!("Failed to read sensor");
            }
        }
        Timer::after_secs(2).await;
    }
}
