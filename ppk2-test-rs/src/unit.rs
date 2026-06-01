//! Types that store SI units which ensures we keep data correct through conversions.

use std::{
    ops::{Add, AddAssign},
    time::Duration,
};

use serde::{Deserialize, Serialize};

/// Current Capacity in Ampere Hour
/// Stored as pico Ampere Hour (pAh = (µA * µs)/3_600) to minimize data loss.
/// [f32::MAX] > 10^{38}, which means that we can store up to 10^{26}
/// This is plenty for this unit
#[derive(Debug, Copy, PartialEq, Clone, Serialize, Deserialize)]
pub struct AmpereHour(f32);

impl AmpereHour {
    /// Zero capacity
    pub const ZERO: AmpereHour = AmpereHour(0f32);

    const RATIO_SEC_HOUR: f32 = 3_600.0;
    const RATIO_PICO_MILLI: f32 = 1_000_000_000.0;
    const RATIO_PICO_MICRO: f32 = 1_000_000.0;
    const RATIO_PICO_NANO: f32 = 1_000.0;

    /// Current Capacity stored as pAh
    pub fn from(ampere: Ampere, duration: Duration) -> Self {
        // It is safe to convert duration to f32 as we only expect durations
        // smaller then [Setup::TIMEOUT_DURATION].
        Self((ampere.as_micros() * duration.as_micros() as f32) / Self::RATIO_SEC_HOUR)
    }

    /// Capacity returned as mAh
    pub fn as_millis(self) -> f32 {
        self.0 / Self::RATIO_PICO_MILLI
    }

    /// Capacity returned as µAh
    pub fn as_micros(self) -> f32 {
        self.0 / Self::RATIO_PICO_MICRO
    }

    /// Capacity returned as nAh
    pub fn as_nanos(self) -> f32 {
        self.0 / Self::RATIO_PICO_NANO
    }

    /// Capacity returned as pAh
    pub fn as_picos(self) -> f32 {
        self.0
    }
    /// Pretty print with unit
    pub fn pretty(self) -> String {
        match self.0 {
            x if x > Self::RATIO_PICO_MILLI => format!("{:.3} mAh", self.as_millis()),
            x if x > Self::RATIO_PICO_MICRO => format!("{:.3} µAh", self.as_micros()),
            x if x > Self::RATIO_PICO_NANO => format!("{:.3} nAh", self.as_nanos()),
            _ => format!("{:.3} pAh", self.as_picos()),
        }
    }
}

impl Add for AmpereHour {
    type Output = AmpereHour;
    fn add(mut self, rhs: Self) -> Self::Output {
        self.0 = self.0 + rhs.0;
        self
    }
}

impl AddAssign<AmpereHour> for AmpereHour {
    fn add_assign(&mut self, rhs: AmpereHour) {
        *self = *self + rhs;
    }
}

/// Current in Ampere stored as μA
/// Storage in f32, [f32::MAX] is in the range of 10^{38}.
/// This is more than sufficient for our use case.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Ampere(f32);

impl Ampere {
    #[allow(missing_docs)]
    pub fn from_micros(micros: f32) -> Self {
        Self(micros)
    }

    #[allow(missing_docs)]
    pub fn as_micros(&self) -> f32 {
        self.0
    }
}

impl PartialOrd for Ampere {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
