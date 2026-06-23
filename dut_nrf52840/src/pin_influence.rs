#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Input, Output, Pull};
use embassy_time::{Duration, Timer};
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

// Naive, can probably be done better
fn set_section(pins: &mut [Output; 8], section: u8) {
    for i in 0..8 {
        match section & (1 << i) {
            0 => pins[i].set_low(),
            _ => pins[i].set_high(),
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_nrf::init(Default::default());
    let pins = &mut [
        pin!(p.P1_01),
        pin!(p.P1_02),
        pin!(p.P1_03),
        pin!(p.P1_04),
        pin!(p.P1_05),
        pin!(p.P1_06),
        pin!(p.P1_07),
        pin!(p.P1_08),
    ];

    let mut button1 = Input::new(p.P0_11, Pull::Up);
    let duration = Duration::from_millis(10);

    loop {
        // Waits for button to be released
        button1.wait_for_rising_edge().await;

        // Loops through all possible combinations waiting for a certain duration
        // between switching
        for i in 1..=255 {
            set_section(pins, i);
            match i {
                42 => Timer::after(Duration::from_millis(100)).await,
                23 => Timer::after(Duration::from_millis(50)).await,
                37 => Timer::after(Duration::from_millis(200)).await,
                _ => Timer::after(duration).await,
            }
        }

        // Resets to all 0s
        set_section(pins, 0);
    }
}
