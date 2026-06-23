#![doc = include_str!("../README.md")]

// Include datatypes from crate and expose them to lib users
// pub mod data;
pub mod data;
pub mod error;
pub mod logic;
pub mod plot;
pub mod timer;

use polars::frame::DataFrame;
use ppk2::measurement::Measurement;
use ppk2::measurement::MeasurementMatch;
use uom::ConstZero;
use uom::fmt::DisplayStyle::*;
use uom::si::electric_current::microampere;
use uom::si::f64::ElectricCurrent;
use uom::si::f64::Time;
use uom::si::time::microsecond;

// use audio_thread_priority::{
//     demote_current_thread_from_real_time, promote_current_thread_to_real_time,
// };

// We use the ppk2-rs library to interface with the Ppk2
use ppk2::{
    Ppk2, try_find_ppk2_port,
    types::{DevicePower, MeasurementMode},
};
use uom::si::time::millisecond;
use uom::si::time::second;

use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;
// Used for time management
use std::{
    env::{current_dir, set_current_dir},
    io::{self, Write},
    path::Path,
    process::{Command, Stdio},
};

// Local time for getting the current time as the std lib does not give a
// general way to get this.

use crate::data::Buffers;
use crate::data::Sample;
use crate::{
    logic::{MeasureStatus, When},
    timer::Timer,
};

/// Macro to help with [Setup::flash]
/// Gets a stream of child process and displays it in the parent stdout
/// in a given format
macro_rules! pipe_fmt {
    ($stream:expr, $format:expr) => {
        if let Some(stream) = $stream.take() {
            let reader = std::io::BufReader::new(stream);
            for line in std::io::BufRead::lines(reader).flatten() {
                println!($format, line);
            }
        }
    };
}

/// The experiment setup.
/// Create a new setup with [Setup::new]
/// Flash a device with a custom flash script with [Setup::flash]
/// Then measure with [Setup::measure] which returns a [Sections] object.
pub struct Setup {
    /// The ppk2 is wrapped in an Option type to keep it live during the lifetime
    /// of Setup. When Ppk2 is moved (in [Ppk2::start_measurement]) we take the
    /// value from [Setup::ppk2] leaving a None value. When the measurement is
    /// completed we put it back. This is done with the appropriatly named
    /// [Setup::take] and [Setup::put] functions.
    ppk2: Option<Ppk2>,
    /// The rate that will be measured with
    /// Will not update the rate of a measurement while mid measurement
    pub rate: Rate,
}

/// All functionality in one test to keep the lifetime of the ppk2 alive
/// Needed to make the ppk2 not shut off when borrowed
impl Setup {
    const TIMEOUT_DURATION: Duration = Duration::from_secs(2);

    /// Creates a new setup from a specified port with a [Rate::FINE] rate.

    pub fn new(ppk2_port: String) -> Setup {
        let mut ppk2 = Ppk2::new(ppk2_port, MeasurementMode::Ampere).unwrap();

        ppk2.set_device_power(DevicePower::Disabled).unwrap();

        Self::print_header();

        Setup {
            ppk2: Some(ppk2),
            rate: Rate::COARSE,
        }
    }

    /// Tries to find a ppk2_port and creates a new setup from it.
    pub fn find() -> Setup {
        Self::new(try_find_ppk2_port().unwrap())
    }

    /// Flashes the device with a given flash command from a specified path.
    ///
    /// Flashing while the ppk2 is connected without providing power will
    /// soft brick the target. As providing power is not instant this function
    /// will wait until it detects power.
    ///
    /// For the nRF52840 measuring a current greater than 0 is enough to detect
    /// if enough power is provided to flash. When the target device is soft
    /// bricked due to not having power one can look to use wait_until on a
    /// greater current.
    pub fn flash(&mut self, target_dir: &Path, flash_command: &mut Command) {
        self.power_enable();

        // We wait until we actually measure some power to continue.
        //
        // In the case of the nRF52840 we measure negative current
        // when the power is not provided to the board.
        let _ = self.measure(
            When::CurrentGt(ElectricCurrent::ZERO),
            When::Duration(Time::new::<millisecond>(1.0)),
        );

        // We flash the device from the target directory and pipe stdout and
        // stderr of the child to capture it in the terminal.
        let original_dir = current_dir().unwrap();
        set_current_dir(target_dir).unwrap();
        let mut child = flash_command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawning flash_command");
        set_current_dir(original_dir).unwrap();

        println!("\n==============COMMAND OUTPUT================");

        pipe_fmt!(child.stderr, "[stderr] {}");
        pipe_fmt!(child.stdout, "[stdout] {}");

        // Wait for the child process to finish and give feedback on the code
        let exit_child = child.wait().unwrap();

        match exit_child.code() {
            Some(code) => println!("==============EXIT CODE {:<3}=================\n", code),
            None => println!("==============TERMINATED====================\n"),
        }

        let _ = self.measure(
            When::CurrentGt(ElectricCurrent::ZERO),
            When::Duration(Time::new::<second>(2.0)),
        );

        self.power_disable();
    }

    // #[allow(missing_docs)]
    // pub fn promoted_measure(&mut self, start: When, stop: When) -> DataFrame {
    //     // ... on a thread that will compute audio and has to be real-time:
    //     match promote_current_thread_to_real_time(512, self.rate.as_u32()) {
    //         Ok(h) => {
    //             println!("this thread is now bumped to real-time priority.");

    //             let ret = self.measure(start, stop);

    //             match demote_current_thread_from_real_time(h) {
    //                 Ok(_) => {
    //                     println!("this thread is now bumped back to normal.");
    //                     return ret;
    //                 }
    //                 Err(_) => {
    //                     println!("Could not bring the thread back to normal priority.");
    //                     panic!()
    //                 }
    //             };
    //         }
    //         Err(e) => {
    //             eprintln!("Error promoting thread to real-time: {}", e);
    //             panic!()
    //         }
    //     }
    // }

    #[allow(missing_docs)]
    pub fn measure(&mut self, start: When, stop: When) -> DataFrame {
        self.power_enable();
        let ppk2 = self.take();
        let (rcv, stop_ppk2) = ppk2.start_measurement(self.rate.as_usize()).unwrap();

        use MeasureStatus::*;
        let mut status = Waiting;
        let mut buffers = Buffers::new(1024);

        let mut timestamp_prev = uom::si::time::Time::new::<microsecond>(0.0f64);
        let mut timestamp_timer = Timer::start().unwrap();

        loop {
            use MeasurementMatch::*;
            use RecvTimeoutError::*;
            match rcv.recv_timeout(Self::TIMEOUT_DURATION) {
                Ok(Match(Measurement { micro_amps, pins })) => {
                    let timestamp_now = timestamp_timer.elapsed().unwrap();
                    let sample = &mut Sample {
                        timestamp: timestamp_prev,
                        latency: timestamp_now - timestamp_prev,
                        current: ElectricCurrent::new::<microampere>(micro_amps as f64),
                        pins: pins.into(),
                    };
                    timestamp_prev = timestamp_now;

                    match status {
                        Waiting
                            if (When::CurrentGt(ElectricCurrent::ZERO)
                                & When::Duration(Time::new::<millisecond>(1.0)))
                            .eval(sample)
                                && start.eval(sample) =>
                        {
                            sample.timestamp = uom::si::time::Time::new::<microsecond>(0.0f64);
                            buffers.push(sample);

                            status = Measuring;

                            timestamp_timer.reset().unwrap();
                            timestamp_prev = uom::si::time::Time::new::<microsecond>(0.0f64);

                            println!("\nStart: {:?}", start);
                        }
                        Waiting => Self::print_status(&status, sample),
                        Measuring if stop.eval(sample) => {
                            println!("Stop:  {:?}", stop);
                            self.stop_and_put(stop_ppk2);
                            return buffers.finish();
                        }
                        Measuring => buffers.push(sample),
                    }
                }
                Err(Disconnected) => todo!("Disconnected"),
                _ => todo!(),
            }
        }
    }

    /// Enables the power on the ppk2 device
    /// This does not have an immediate effect on the target board
    pub fn power_enable(&mut self) {
        let mut ppk2 = self.take();
        ppk2.set_device_power(DevicePower::Enabled).unwrap();
        self.put(ppk2);
    }

    /// Disables the power on the ppk2 device
    /// This does not have an immediate effect on the target board
    pub fn power_disable(&mut self) {
        let mut ppk2 = self.take();
        ppk2.set_device_power(DevicePower::Disabled).unwrap();
        self.put(ppk2);
    }

    fn take(&mut self) -> Ppk2 {
        self.ppk2.take().unwrap()
    }

    fn put(&mut self, ppk2: Ppk2) {
        self.ppk2 = Some(ppk2);
    }

    fn stop_and_put(&mut self, stop_ppk2: impl FnOnce() -> Result<Ppk2, ppk2::Error>) {
        self.put(stop_ppk2().unwrap());
    }

    fn print_header() {
        println!(
            "\n|   Pins   |    Timestamp (s) | Latency (µs/smp) |   Current (µA)   |  Status  |"
        );
        println!(
            "|==========|==================|==================|==================|==========|"
        );
    }

    fn print_status(
        status: &MeasureStatus,
        Sample {
            timestamp,
            latency,
            current,
            pins,
        }: &Sample,
    ) {
        let spinner = ['|', '/', '-', '\\'];
        print!(
            "\r| {:>8} | {:>16} | {:>16} | {:>16} | {:8} | [{}]",
            pins.to_string(),
            format!(
                "{:.3}",
                Time::format_args(second, Abbreviation).with(*timestamp)
            )
            .as_str(),
            format!(
                "{:.3}",
                Time::format_args(microsecond, Abbreviation).with(*latency)
            )
            .as_str(),
            format!(
                "{:.3}",
                ElectricCurrent::format_args(microampere, Abbreviation).with(*current)
            )
            .as_str(),
            format!("{:?}", status),
            spinner[timestamp.get::<second>() as usize % spinner.len()]
        );
        io::stdout().flush().unwrap();
    }
}

/// The rate of samples of the ppk2 in samples per second
/// Ranges between [Rate::MIN_SPS] and [Rate::MAX_SPS].
#[derive(Copy, Clone)]
pub struct Rate(u32);

impl Rate {
    /// Constant value which represents the *minimum* samples per second that can
    /// be passed to the ppk2.
    pub const MIN_SPS: u32 = 1;

    /// Constant value which represents the *maximum* samples per second that can
    /// be passed to the ppk2.
    pub const MAX_SPS: u32 = 100_000;

    /// Rate which results in a fine granularity in measurements
    ///
    /// (+) More accurate
    /// (+) Can spot outliers with effects on powerconsumption > 10 µseconds
    /// (-) Higher storage usage
    /// (-) Outliers can skew metrics like averages
    pub const FINE: Rate = Rate(Rate::MAX_SPS);

    /// Rate which results in a coarse granularity in measurements
    ///
    /// (-) Less accurate
    /// (-) It is harder to spot single instruction outliers
    /// (+) Lower storage usage
    /// (+) Good for comparing average loads
    pub const COARSE: Rate = Rate(10_000);

    /// Rate data constructor
    /// Rejects values that lie outside of the range
    /// [Rate::MIN_SPS] ..= [Rate::MAX_SPS]
    pub fn from_sps(sps: u32) -> Rate {
        match sps {
            Rate::MIN_SPS..=Rate::MAX_SPS => Rate(sps),
            x => todo!("sample size out of bounds: {}", x),
        }
    }

    /// Returns the rate as samples per second in u32
    pub fn as_u32(self) -> u32 {
        self.0
    }

    /// Returns the rate as samples per second in usize
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}
