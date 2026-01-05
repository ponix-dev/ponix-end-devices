//! This example runs on the RAK4631 WisBlock, which has an nRF52840 MCU and Semtech Sx126x radio.
//! Other nrf/sx126x combinations may work with appropriate pin modifications.
//! It demonstrates LoRaWAN join functionality.
#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::rng::Rng;
use embassy_nrf::{bind_interrupts, mode, peripherals, rng, spim};
use embassy_time::{Delay, Duration, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use lora_phy::iv::GenericSx126xInterfaceVariant;
use lora_phy::lorawan_radio::LorawanRadio;
use lora_phy::sx126x::{self, Sx1262, Sx126x, TcxoCtrlVoltage};
use lora_phy::LoRa;
use lorawan_device::async_device::{region, Device, EmbassyTimer, JoinMode, JoinResponse};
use lorawan_device::region::Subband;
use lorawan_device::{AppEui, AppKey, DevEui};
use rand::Rng as _;
use {defmt_rtt as _, panic_probe as _};

const MAX_TX_POWER: u8 = 14;

// TODO: Replace these with your actual LoRaWAN credentials from your network server (TTN, Chirpstack, etc.)
// You can find these in your network server's device configuration
const DEVEUI: [u8; 8] = [0x02, 0x4B, 0x00, 0xD8, 0x7E, 0xD5, 0xB3, 0x70]; // Device EUI (unique device identifier) - LSB format
const APPEUI: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // Application EUI (aka JoinEUI)
const APPKEY: [u8; 16] = [
    0x5F, 0xB7, 0xF2, 0xED, 0xD3, 0xF9, 0x66, 0x0F, 0xC0, 0x0B, 0xBE, 0x8F, 0xB9, 0xBE, 0x29, 0x4A,
];

bind_interrupts!(struct Irqs {
    TWISPI1 => spim::InterruptHandler<peripherals::TWISPI1>;
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

// Generate "jittered" delay for retry attempts up to maximum of 1 hour
pub fn generate_delay(rng: &mut Rng<'static, peripherals::RNG, mode::Async>, retries: u16) -> u16 {
    let base = core::cmp::min(10 + (10 * retries), 3600);
    let jitter = base / 5;
    (base - jitter).saturating_add(rng.random_range(jitter..=2 * jitter))
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let nss = Output::new(p.P1_10, Level::High, OutputDrive::Standard);
    let reset = Output::new(p.P1_06, Level::High, OutputDrive::Standard);
    let dio1 = Input::new(p.P1_15, Pull::Down);
    let busy = Input::new(p.P1_14, Pull::None);
    let rf_switch_rx = Output::new(p.P1_05, Level::Low, OutputDrive::Standard);
    let rf_switch_tx = Output::new(p.P1_07, Level::Low, OutputDrive::Standard);

    let mut spi_config = spim::Config::default();
    spi_config.frequency = spim::Frequency::M16;
    let spim = spim::Spim::new(p.TWISPI1, Irqs, p.P1_11, p.P1_13, p.P1_12, spi_config);
    let spi = ExclusiveDevice::new(spim, nss, Delay);

    let config = sx126x::Config {
        chip: Sx1262,
        tcxo_ctrl: Some(TcxoCtrlVoltage::Ctrl1V7),
        use_dcdc: true,
        rx_boost: false,
    };
    let iv = GenericSx126xInterfaceVariant::new(
        reset,
        dio1,
        busy,
        Some(rf_switch_rx),
        Some(rf_switch_tx),
    )
    .unwrap();
    let lora = LoRa::new(Sx126x::new(spi, iv, config), true, Delay)
        .await
        .unwrap();

    let mut radio: LorawanRadio<_, _, MAX_TX_POWER> = lora.into();
    // Increase RX window timing to improve join accept reception
    // Default is 50ms, increase for better reception in noisy environments
    radio.set_rx_window_lead_time(100);
    radio.set_rx_window_buffer(200);

    // Configure US915 sub-band 2 (channels 8-15) - most common for US gateways
    let mut us915 = lorawan_device::region::US915::new();
    // Sub-band 2 is most common (TTN, Helium, etc.)
    // Change to different sub-band if your gateway uses a different one
    // Most US networks (TTN, Helium) use sub-band 2
    // Try sub-band 2 first, then 1, then others if needed
    us915.set_join_bias(Subband::_2);
    let region = region::Configuration::from(us915);
    let mut device: Device<_, _, _, 256, 1> =
        Device::new(region, radio, EmbassyTimer::new(), Rng::new(p.RNG, Irqs));

    defmt::info!("Joining LoRaWAN network (US915 sub-band 2)");
    defmt::info!("DevEUI: {:02x}", DEVEUI);
    defmt::info!("AppEUI: {:02x}", APPEUI);
    defmt::info!("AppKey: {:02x}", APPKEY);
    defmt::info!("Max TX Power: {} dBm", MAX_TX_POWER);
    defmt::info!("RX window lead time: 100ms, buffer: 200ms");

    let join_mode = JoinMode::OTAA {
        deveui: DevEui::from(DEVEUI),
        appeui: AppEui::from(APPEUI),
        appkey: AppKey::from(APPKEY),
    };

    let mut retries = 0;

    loop {
        let join_result = device.join(&join_mode).await;
        match join_result {
            Ok(JoinResponse::JoinSuccess) => {
                info!("LoRaWAN network joined");
                break;
            }
            Ok(other) => {
                error!(
                    "Join response was not success: {:?}",
                    defmt::Debug2Format(&other)
                );
                // Add more specific error handling
                match other {
                    JoinResponse::NoJoinAccept => {
                        error!("No join accept received - check RX windows, gateway downlink, or network server response");
                    }
                    _ => {}
                }
            }
            Err(e) => {
                error!("Join failed with error: {:?}", defmt::Debug2Format(&e));
            }
        }
        let delay = generate_delay(&mut device.rng, retries);

        info!("Retrying join in {} seconds..", delay);
        Timer::after(Duration::from_secs(delay.into())).await;
        retries += 1;
    }

    info!("LoRaWAN network joined!");

    // Send uplink messages
    loop {
        // Prepare your message payload
        let mut buffer = [0u8; 64];
        let message = b"Hello from nRF52840!";
        buffer[..message.len()].copy_from_slice(message);

        info!("Sending uplink: {:?}", &buffer[..message.len()]);

        // Send unconfirmed uplink on port 1
        // Port numbers 1-223 are available for application use
        // Use send_recv for confirmed uplinks (requires downlink ACK)
        match device.send(&buffer[..message.len()], 1, false).await {
            Ok(_) => {
                info!("Uplink sent successfully");
            }
            Err(e) => {
                error!("Failed to send uplink: {:?}", defmt::Debug2Format(&e));
            }
        }

        // Wait before sending next message (respect duty cycle)
        // LoRaWAN has duty cycle limitations - don't send too frequently
        info!("Waiting 60 seconds before next uplink...");
        Timer::after(Duration::from_secs(60)).await;
    }
}
