#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::{Peripherals, gpio::{Input, Output, Pull}};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

const DURATION: Duration = Duration::from_secs(1);

macro_rules! pin {
    ($pin:expr) => {
        embassy_nrf::gpio::Output::new(
            $pin,
            embassy_nrf::gpio::Level::Low,
            embassy_nrf::gpio::OutputDrive::Standard,
        )
    };
}

static BUFSIZE : usize = 4;
static CH: Channel<CriticalSectionRawMutex, PinState, BUFSIZE> = Channel::new();

enum PinState {
    High(usize),
    Low(usize),
    Logic(u8),
}

#[embassy_executor::task]
async fn pin_task(p: Peripherals) {
//async fn pin_task(pins: &'static mut [Output<'static>; 8]) {
    loop {
        let state = CH.receive().await;

        use PinState::*;
        match state {
            High(i) => pins[i].set_high(),
            Low(i) => pins[i].set_low(),
            Logic(s) => {
                for i in 0..8 {
                    match s & (1 << i) {
                        0 => pins[i].set_low(),
                        _ => pins[i].set_high(),
                    }
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn a() {
    CH.send(PinState::High(1)).await;
    Timer::after(DURATION).await;
    CH.send(PinState::Low(1)).await;
}

#[embassy_executor::task]
async fn b() {
    CH.send(PinState::High(2)).await;
    Timer::after(DURATION).await;
    CH.send(PinState::Low(2)).await;
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
    _spawner.spawn(pin_task(pins.clone()));


    let mut button1 = Input::new(p.P0_11, Pull::Up);

    loop {
        // Waits for button to be released
        button1.wait_for_rising_edge().await;

        // Loops through all possible combinations waiting for a certain duration
        // between switching
        _spawner.spawn(a()).unwrap();
        _spawner.spawn(b()).unwrap();
    }
}
