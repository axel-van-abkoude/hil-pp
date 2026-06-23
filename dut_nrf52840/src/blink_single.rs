#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker};
use {defmt_rtt as _, panic_probe as _};

macro_rules! pin {
    ($pin:expr) => {
        embassy_nrf::gpio::Output::new(
            $pin,
            embassy_nrf::gpio::Level::Low,
            embassy_nrf::gpio::OutputDrive::Standard,
        )
    };
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_nrf::init(Default::default());
    let mut pin = pin!(p.P0_01);

    let duration = Duration::from_micros(10);
    let mut ticker = Ticker::every(duration);

    loop {
        pin.toggle();
        ticker.next().await;
    }
}
