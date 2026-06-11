//!
//! An example that measures the power consumption of an example with different
//! measurement rates.
//!

use ate_ppk2::{
    Setup, data::Sections, logic::When::*, unit::{Micro, Unit}
};
use std::{fs::File, io::{Result, Write}, path::Path, process::Command, time::Duration};

const PATH: &str = "../dut_nrf52840";
const WARMUP: Duration = Duration::from_secs(1);
const DURATION: Duration = Duration::from_secs(10);

fn main() -> Result<()> {
    let single = run_experiment(PATH, "blink_single")?;
    let all = run_experiment(PATH, "blink_all")?;
    let wait = run_experiment(PATH, "wait")?;
    Ok(())
}

fn run_experiment(path: &'static str, experiment: &'static str) -> Result<Sections> {

    let mut file = File::create(format!("blinky_{}.txt", experiment))?;
    let mut setup = Setup::find();

    // Flash device
    setup.flash(
        Path::new(path),
        Command::new("cargo")
            .arg("flash")
            .arg("--chip")
            .arg("nRF52840_xxAA")
            .arg("--release")
            .arg("--bin")
            .arg(experiment),
    );
    setup.power_enable();

    let sections = setup.measure(Time(WARMUP), Time(DURATION));
    file.write_fmt(format_args!("{}", sections))?;

    println!("{}:\n {:?}\n {}", experiment, sections.total_duration(), sections.total_capacity().pretty::<Micro>());

    Ok(sections)
}
