#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{self, Config, I2c};
use embassy_rp::peripherals::I2C0;
use embassy_time::{Delay, Timer};
use embedded_aht20::Aht20;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Temperature/Humidity sensor starting...");

    // Initialize I2C0 on GP5 (SCL) and GP4 (SDA)
    let i2c = I2c::new_async(p.I2C0, p.PIN_5, p.PIN_4, Irqs, Config::default());

    // Initialize AHT20 sensor
    let mut sensor = match Aht20::new(i2c, embedded_aht20::DEFAULT_I2C_ADDRESS, Delay).await {
        Ok(s) => {
            info!("AHT20 sensor initialized!");
            s
        }
        Err(_e) => {
            error!("Failed to initialize AHT20 sensor");
            loop {
                Timer::after_secs(1).await;
            }
        }
    };

    loop {
        match sensor.measure().await {
            Ok(measurement) => {
                info!(
                    "Temperature: {} C, Humidity: {} %",
                    measurement.temperature.celsius(),
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
