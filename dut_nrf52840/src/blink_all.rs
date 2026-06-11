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
    let mut pin1 = pin!(p.P1_01);
    let mut pin2 = pin!(p.P1_02);
    let mut pin3 = pin!(p.P1_03);
    let mut pin4 = pin!(p.P1_04);
    let mut pin5 = pin!(p.P1_05);
    let mut pin6 = pin!(p.P1_06);
    let mut pin7 = pin!(p.P1_07);
    let mut pin8 = pin!(p.P1_08);

    let duration = Duration::from_micros(10);
    let mut ticker = Ticker::every(duration);

    loop {
        pin1.toggle();
        pin2.toggle();
        pin3.toggle();
        pin4.toggle();
        pin5.toggle();
        pin6.toggle();
        pin7.toggle();
        pin8.toggle();
        ticker.next().await;
    }
}
