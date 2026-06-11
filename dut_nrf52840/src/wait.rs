#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let _ = embassy_nrf::init(Default::default());
    let duration = Duration::from_micros(10);
    let mut ticker = Ticker::every(duration);

    loop {
        ticker.next().await;
    }
}

