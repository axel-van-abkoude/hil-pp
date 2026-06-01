//! Datastructures that store the experiment data

use std::{
    array::from_fn,
    fmt::{self, Display, Formatter},
    ops::{Index, IndexMut},
    time::Duration,
};

use ppk2::measurement::Measurement;
use serde::{Deserialize, Serialize, ser::Serializer};

use crate::{
    logic::Pins,
    unit::{Ampere, AmpereHour},
};

/// DATATYPES

/// The data associated with a section
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Section {
    /// The total time spent in a section in the total timespan
    pub pins: Pins,
    /// The total time spent in a section in the total timespan
    pub total_duration: Duration,
    /// The total capacity of a section in the total timespan
    pub total_capacity: AmpereHour,
}

// IMPLS

impl From<Pins> for Section {
    fn from(value: Pins) -> Self {
        Section {
            pins: value,
            total_duration: Duration::ZERO,
            total_capacity: AmpereHour::ZERO,
        }
    }
}

impl Display for Section {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "| {:<3} | {:8} | {:>32} | {:>28} µAh |",
            u8::from(self.pins),
            self.pins.to_string(),
            self.total_duration.as_micros(),
            self.total_capacity.as_micros()
        )?;
        Ok(())
    }
}

impl IndexMut<Pins> for Sections {
    fn index_mut(&mut self, index: Pins) -> &mut Self::Output {
        &mut self.0[u8::from(index) as usize]
    }
}

impl Index<Pins> for Sections {
    type Output = Section;
    fn index(&self, index: Pins) -> &Self::Output {
        &self.0[u8::from(index) as usize]
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
/// The datatype that stores all sections
pub struct Sections([Section; 256]);

impl Display for Sections {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", "=".repeat(88))?;
        for section in self.0.iter() {
            match *section {
                Section {
                    pins: _,
                    total_duration: Duration::ZERO,
                    total_capacity: AmpereHour::ZERO,
                } => continue,
                s => {
                    write!(f, "{}", s)?;
                }
            }
        }
        writeln!(f, "|{}|", "-".repeat(86))?;
        writeln!(
            f,
            "| Total:         | {:>29} µs | {:>32} |",
            self.total_duration().as_micros(),
            self.total_capacity().pretty()
        )?;
        writeln!(f, "{}", "=".repeat(88))?;
        Ok(())
    }
}

impl Sections {
    /// Initializes all sections with the index mapped to a section
    pub fn new() -> Sections {
        Sections(from_fn(|i| Section::from(Pins::from(i as u8))))
    }

    /// Returns the total capacity of all sections combined
    pub fn total_capacity(mut self) -> AmpereHour {
        self.0
            .iter_mut()
            .reduce(|acc, section| {
                acc.total_capacity += section.total_capacity;
                acc
            })
            .unwrap()
            .total_capacity
    }

    /// Returns the total duration of all sections combined
    pub fn total_duration(mut self) -> Duration {
        self.0
            .iter_mut()
            .reduce(|acc, section| {
                acc.total_duration += section.total_duration;
                acc
            })
            .unwrap()
            .total_duration
    }

    /// Update the Sections with a measurement and the duration of that measurement
    pub fn update_with(&mut self, measurement: Measurement, duration: Duration) {
        let section = &mut self[Pins::from(measurement.pins)];

        section.total_capacity +=
            AmpereHour::from(Ampere::from_micros(measurement.micro_amps), duration);
        section.total_duration += duration;
    }
}

/// One Sample collected by the ppk2 containing:
/// * the timestamp in the measurement
/// * the duration of the sample itself
/// * the average current of the sample
/// * the most prevalent pin configuration of the sample
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Sample {
    #[serde(rename = "Timestamp In Measurement (μs)")]
    #[serde(serialize_with = "ser_duration_micros")]
    #[allow(missing_docs)]
    pub timestamp: Duration,
    #[serde(rename = "Duration Sample (μs)")]
    #[serde(serialize_with = "ser_duration_micros")]
    #[allow(missing_docs)]
    pub duration: Duration,
    #[serde(rename = "Current Sample (μA)")]
    #[serde(serialize_with = "ser_ampere_micros")]
    #[allow(missing_docs)]
    pub current: Ampere,
    #[serde(rename = "Logic Pins Sample (D0-D7)")]
    #[serde(serialize_with = "ser_pins_str")]
    #[allow(missing_docs)]
    pub pins: Pins,
}

fn ser_duration_micros<S>(duration: &Duration, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_f32(duration.as_micros() as f32)
}

fn ser_ampere_micros<S>(current: &Ampere, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_f32(current.as_micros())
}

fn ser_pins_str<S>(pins: &Pins, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&pins.to_string())
}
