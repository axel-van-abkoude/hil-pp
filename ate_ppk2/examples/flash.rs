//!
//! An example that flashes the device with an experiment.
//!

use ate_ppk2::Setup;
use std::{path::Path, process::Command};

const EXPERIMENT: &str = "pin_influence";
const PATH: &str = "../dut_nrf52840";

fn main() {
    let mut setup = Setup::find();
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
}
