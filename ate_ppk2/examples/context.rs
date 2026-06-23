//!
//! An example that measures the power consumption of an example with different
//! measurement rates.
//!

use std::{path::Path, process::Command};

use ate_ppk2::{
    Rate, Setup,
    data::{Current, Latency, Metrics, Samples, Timestamp, store_dataframe},
    logic::When::*,
    plot::*,
};
use plotters::style::{full_palette::PURPLE, *};
use uom::si::{f64::Time, time::second};

fn main() {
    let mut setup = Setup::find();
    setup.rate = Rate::FINE;
    let one_sec: Time = Time::new::<second>(1.0f64);

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

    let mut df = setup.measure(Duration(one_sec), Duration(one_sec));

    store_dataframe(&mut df, Path::new("data/radio_one_sec.parquet"));

    let mut plot = Plot::<Timestamp, Current>::new(&df, "Measure radio for one second".to_string(), Path::new("plots/one_sec_no_poi.png"));
    plot.draw_line(&df, BLUE, "trace".to_string());
    plot.present();

    println!("METRICS PASSIVE {}", Metrics::<Current>::new(&df));


    let mut plot = Plot::<Timestamp, Current>::new(&df, "Measure radio for one second".to_string(), Path::new("plots/one_sec_with_poi.png"));
    plot.draw_line(&df, BLUE, "trace".to_string());
    plot.draw_poi_bounds(&df,RED,"D0".to_string(), "each".to_string());
    plot.draw_poi_bounds(&df,PURPLE,"D4".to_string(), "once".to_string());
    // plot.draw_poi_bounds(&df,YELLOW,"D7".to_string(), "base".to_string());
    plot.present();

    let mut plot = Plot::<Samples, Latency>::new(&df, "Latency of measurements".to_string(), Path::new("plots/latency.png"));
    plot.draw_line(&df, BLUE, "latency".to_string());
    plot.present();
}
