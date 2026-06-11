//!
//! An example that measures the power consumption of an example with different
//! measurement rates.
//!

use ate_ppk2::{
    Rate, Setup,
    logic::When::*,
};
use std::{path::Path, process::Command, time::Duration};

const EXPERIMENT: &str = "pin_influence";
const PATH: &str = "../dut_nrf52840";
const WARMUP: Duration = Duration::from_secs(1);

fn main() {
    let mut setup = Setup::find();

    // Flash device
    setup.flash(
        Path::new(PATH),
        Command::new("cargo")
            .arg("flash")
            .arg("--chip")
            .arg("nRF52840_xxAA")
            .arg("--release")
            .arg("--bin")
            .arg(EXPERIMENT),
    );
    setup.power_enable();
    setup.wait_until(Time(WARMUP) & Logic(0.into()));

    // Run with sample sizes 10_000 to 100_000 with intervals of 1_000
    for i in 1..=10 {
        setup.rate = Rate::from_sps(i * 10_000);
        println!(
            "{}^^^^^^ RATE {}0_000 ^^^^^^\n",
            setup.measure(!Logic(0.into()), Logic(0.into())),
            i
        );
    }
}
