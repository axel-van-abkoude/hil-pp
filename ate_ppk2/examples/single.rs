//!
//! An example that measures the power consumption of an example with different
//! measurement rates.
//!

use ate_ppk2::{
    Setup,
    logic::{Pins, When::*},
};
use std::time::Duration;

const WARMUP: Duration = Duration::from_secs(1);

fn main() {
    let mut setup = Setup::find();
    setup.power_enable();

    let all_low = Pins::from(0u8);
    setup.wait_until(Time(WARMUP) & Logic(all_low));

    // Measure from a non 0 pin configuration until 0 has been found
    println!("{}", setup.measure(!Logic(all_low), Logic(all_low)));
}
