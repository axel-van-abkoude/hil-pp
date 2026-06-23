//!
//! An example that measures the power consumption of an example with different
//! measurement rates.
//!

use ate_ppk2::{Setup, data::*, logic::When::*, plot::Plot};
use std::{path::Path, process::Command};
use uom::si::time::{Time, millisecond};

fn main() {
    let mut setup = Setup::find();
    let experiment = "wait";

    // Flash device
    setup.flash(
        Path::new("../dut_nrf52840"),
        Command::new("cargo")
            .arg("flash")
            .arg("--chip")
            .arg("nRF52840_xxAA")
            .arg("--release")
            .arg("--bin")
            .arg(experiment),
    );

    let df = setup.measure(
        LatencyGt(Time::new::<millisecond>(1.0f64)),
        LatencyGt(Time::new::<millisecond>(1.0f64)),
    );
    println!("{:?}", df);
    let mut plot = Plot::<Samples, Latency>::new(&df, Path::new("plots/latency.png"));
    plot.draw(0.into());
    plot.present();
}
