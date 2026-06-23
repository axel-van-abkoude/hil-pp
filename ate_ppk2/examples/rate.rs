//!
//! An example that measures the power consumption of an example with different
//! measurement rates.
//!

use ate_ppk2::{Rate, Setup, data::{Current, Timestamp, load_dataframe, store_dataframe}, logic::{Pins, When::*}, plot::Plot};
use plotters::style::BLUE;
use std::{path::Path, process::Command};

fn main() {
    let mut setup = Setup::find();

    // Flash device
    setup.flash(
        Path::new("../dut_nrf52840"),
        Command::new("cargo")
            .arg("flash")
            .arg("--chip")
            .arg("nRF52840_xxAA")
            .arg("--release")
            .arg("--bin")
            .arg("radio"),
    );

    // Run with sample sizes 10_000 to 100_000 with intervals of 1_000
    for i in 1..=10 {
        setup.rate = Rate::from_sps(i * 10_000);
        store_dataframe(
            &mut setup.measure(Logic(Pins::pin_low(0)), !Logic(Pins::pin_low(0))),
            Path::new(format!("data/rate_{}.parquet",i).as_str()),
        );
    }

    let df = load_dataframe(Path::new("data/rate_1.parquet"));
    let mut plot = Plot::<Timestamp, Current>::new(
        &df,
        "Trace".to_string(),
        Path::new("plots/rate.png"),
    );
    plot.draw_line(&df, BLUE, format!("10000").to_string());

    for i in 1..=10 {
        let df = load_dataframe(Path::new(format!("data/rate_{}.parquet",i).as_str()));
        plot.draw_line(&df, BLUE, format!("{}0000",i).to_string());
    }
    plot.present();
}
