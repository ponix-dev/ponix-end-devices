#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut led = Output::new(p.P1_03, Level::Low, OutputDrive::Standard);

    loop {
        defmt::info!("LED turning on");
        led.set_high();
        Timer::after_millis(1000).await;
        defmt::info!("LED turning off");
        led.set_low();
        Timer::after_millis(1000).await;
    }
}
