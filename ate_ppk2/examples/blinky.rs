//!
//! An example that measures the power consumption of an example with different
//! measurement rates.
//!

use ate_ppk2::{Setup, data::*, logic::When::*, plot::Plot};
use std::{io::Result, path::Path, process::Command};
use uom::si::time::{Time, millisecond, second};

const PATH: &str = "../dut_nrf52840";

fn main() -> Result<()> {
    let all = run_experiment("blink_all")?;
    let single = run_experiment("blink_single")?;
    let wait = run_experiment("wait")?;
    println!("{}", all);
    println!("{}", single);
    println!("{}", wait);
    Ok(())
}

fn run_experiment(experiment: &'static str) -> Result<Metrics<Current>> {
    let path = format!("plots/{}.png", experiment);
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
            .arg(experiment),
    );

    let df = setup.measure(
        Duration(Time::new::<millisecond>(1.0f64)),
        Duration(Time::new::<second>(1.0f64)),
    );

    let mut plot = Plot::<Timestamp, Current>::new(&df, Path::new(path.as_str()));
    for i in 0u8..=255u8 {
        plot.draw(i);
    }
    plot.present();
    Ok(Metrics::<Current>::new(&df))
}
