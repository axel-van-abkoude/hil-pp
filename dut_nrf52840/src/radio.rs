#![no_std]
#![no_main]
/// Based on example/nrf52840/ieee802154_send.rs
use embassy_executor::Spawner;
use embassy_nrf::config::{Config, HfclkSource};
use embassy_nrf::pac::RADIO;
use embassy_nrf::radio::ieee802154::Packet;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

macro_rules! pin {
    ($pin:expr) => {
        embassy_nrf::gpio::Output::new(
            $pin,
            embassy_nrf::gpio::Level::High,
            embassy_nrf::gpio::OutputDrive::Standard,
        )
    };
}

macro_rules! poi {
    ($pin:expr, $code:expr) => {
        $pin.set_low();
        $code;
        $pin.set_high();
    };
}

async fn clear() {
    RADIO.events_ready().write_value(0);
    RADIO.events_end().write_value(0);
    RADIO.events_disabled().write_value(0);
}

async fn on() {
    RADIO.tasks_txen().write_value(1);
    // Wait until radio is ready
    while RADIO.events_ready().read() == 0 {}
}
async fn off() {
    RADIO.tasks_disable().write_value(1);
    while RADIO.events_disabled().read() == 0 {}
}
async fn send() {
    RADIO.tasks_start().write_value(1);
    // Wait untill sent
    while RADIO.events_end().read() == 0 {}
}

async fn sleep(wait: Duration) {
    Timer::after(wait).await;
}

// Amount of messages to be sent
const N: usize = 1000;
// The duration we wait between messages
const T: Duration = Duration::from_micros(50);
// The message
const M: &'static str = "foo";

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.hfclk_source = HfclkSource::ExternalXtal;

    let p = embassy_nrf::init(config);
    let mut d0 = pin!(p.P1_01);
    let mut d1 = pin!(p.P1_02);
    let mut d2 = pin!(p.P1_03);
    let mut _d3 = pin!(p.P1_04);
    let mut d4 = pin!(p.P1_05);
    let mut d5 = pin!(p.P1_06);
    let mut d6 = pin!(p.P1_07);
    let mut _d7 = pin!(p.P1_08);

    let warmup = Duration::from_millis(50);

    let mut packet = Packet::new();
    packet.copy_from_slice(M.as_bytes());
    RADIO.packetptr().write_value(packet.as_ptr() as u32);
    sleep(warmup).await;

    loop {
        clear().await;
        sleep(warmup).await;

        // For each
        poi!(d0,
            for _ in 0..N {
                poi!(d1, {
                    on().await;
                    poi!(d2, send().await);
                    off().await;
                });
                sleep(T).await;
        });

        clear().await;
        sleep(warmup).await;

        // Once
        poi!(d4, {
            poi!(d5, {
                on().await;
                for _ in 0..N {
                    poi!(d6, send().await);
                    sleep(T).await;
                }
                off().await;
            });
        });

        // clear().await;
        // sleep(warmup).await;

        // // Wait
        // poi!(d7, {
        //     for _ in 0..N {
        //         sleep(T).await;
        //     }
        // });
    }
}
