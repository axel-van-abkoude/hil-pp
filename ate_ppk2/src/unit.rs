//! Types that store SI units which ensures we keep data correct through conversions.

use std::{
    ops::{Add, AddAssign, Sub, SubAssign},
    time::Duration,
};

#[cfg(target_os = "linux")]
use libc::{CLOCK_MONOTONIC_RAW, clock_gettime, timespec};

use serde::{Deserialize, Serialize};

use crate::error::{HILppError::*, Result};

/// The standard library gives us the monotonic clock
/// This is corrected by NTP, which we do not want in measurements of micro seconds
/// Thus we want the raw monotonic clock (CLOCK_MONOTONIC_RAW)
#[derive(Debug, Clone, Copy)]
pub struct Timer {
    sec: u64,
    nsec: u32,
}

/// Use audio_thread_priority
impl Timer {
    #[allow(missing_docs)]
    pub fn new() -> Result<Timer> {
        let mut timer = Timer { sec: 0, nsec: 0 };
        timer.start()?;
        Ok(timer)
    }

    #[allow(missing_docs)]
    pub fn start(&mut self) -> Result<()> {
        let mut ts = timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };

        // Safety: we give a raw pointer to a libc function
        // As we rethrow the libc error if there is one, we can assume safety
        // as long as libc gives an error when it fails
        unsafe {
            if clock_gettime(CLOCK_MONOTONIC_RAW, &mut ts) != 0 {
                return Err(Libc(std::io::Error::last_os_error()));
            }
        }

        // Check if the conversions are valid (we do not lose any data)
        match ts {
            // This should never happen as POSIX throws an error if this is the case.
            // https://man7.org/linux/man-pages/man3/clock_gettime.3.html
            timespec { tv_sec, tv_nsec }
                if tv_sec < 0 || tv_nsec < 0 || tv_nsec > u32::MAX as i64 => Err(TimeError(
                    "This error should never happen as, POSIX ensures that seconds are positive and nanos are in the range [0..999,999,999].".into(),
                )),
            timespec { tv_sec, tv_nsec } => {
                self.sec = tv_sec as u64;
                self.nsec = tv_nsec as u32;
                Ok(())
            }
        }
    }

    #[allow(missing_docs)]
    pub fn elapsed(self) -> Result<Duration> {
        let Timer {
            sec: start_sec,
            nsec: start_nsec,
        } = self;

        match Self::new()? {
            Timer {
                sec: now_sec,
                nsec: _,
            } if self.sec > now_sec => Err(TimeError(format!(
                "Negative seconds: {} (start) > {} (now)",
                start_sec, now_sec
            ))),
            // Nanos overflow
            Timer {
                sec: now_sec,
                nsec: now_nsec,
            } if self.nsec > now_nsec => Ok(Duration::new(
                now_sec - start_sec - 1,
                1e9 as u32 - start_nsec + now_nsec,
            )),
            // No overflow
            Timer {
                sec: now_sec,
                nsec: now_nsec,
            } => Ok(Duration::new(now_sec - start_sec, now_nsec - start_nsec)),
        }
    }
}

/// Current Capacity in Coulomb (As)
/// Stored as micro Ampere Second
#[derive(Debug, Copy, PartialEq, Clone, Serialize, Deserialize)]
pub struct Coulomb(f32);

#[allow(missing_docs)]
impl Coulomb {
    pub fn from(current: Ampere, duration: Duration) -> Self {
        Self(current.to::<Micro>() * duration.as_secs_f32())
    }
    pub fn average_over(&self, duration: Duration) -> Ampere {
        Unit::from::<Micro>(self.to::<Micro>() / duration.as_secs_f32())
    }
}

impl Unit for Coulomb {
    type Storage = Micro;
    const SYMBOL: &'static str = "As";
    const ZERO: Self = Self(0.0);

    fn new(raw: f32) -> Self {
        Self(raw)
    }

    fn raw(&self) -> f32 {
        self.0
    }
}

/// Current in Ampere stored as μA
/// Storage in f32, [f32::MAX] is in the range of 10^{38}.
/// This is more than sufficient for our use case.
#[derive(Debug, Copy, PartialEq, Clone, Serialize, Deserialize)]
pub struct Ampere(f32);

impl Unit for Ampere {
    type Storage = Micro;
    const SYMBOL: &'static str = "A";
    const ZERO: Self = Self(0.0);

    fn new(raw: f32) -> Self {
        Self(raw)
    }

    fn raw(&self) -> f32 {
        self.0
    }
}

#[allow(missing_docs)]
pub trait Scale {
    const FACTOR: f32;
    const SYMBOL: &'static str;
}

#[allow(missing_docs)]
pub struct Pico;
#[allow(missing_docs)]
pub struct Nano;
#[allow(missing_docs)]
pub struct Micro;
#[allow(missing_docs)]
pub struct Milli;
#[allow(missing_docs)]
pub struct Base;
#[allow(missing_docs)]
pub struct Kilo;
#[allow(missing_docs)]
pub struct Mega;
#[allow(missing_docs)]
pub struct Giga;

impl Scale for Pico {
    const FACTOR: f32 = 1e-12;
    const SYMBOL: &'static str = "p";
}
impl Scale for Nano {
    const FACTOR: f32 = 1e-9;
    const SYMBOL: &'static str = "n";
}
impl Scale for Micro {
    const FACTOR: f32 = 1e-6;
    const SYMBOL: &'static str = "µ";
}
impl Scale for Milli {
    const FACTOR: f32 = 1e-3;
    const SYMBOL: &'static str = "m";
}
impl Scale for Base {
    const FACTOR: f32 = 1.0;
    const SYMBOL: &'static str = "";
}
impl Scale for Kilo {
    const FACTOR: f32 = 1e3;
    const SYMBOL: &'static str = "k";
}
impl Scale for Mega {
    const FACTOR: f32 = 1e6;
    const SYMBOL: &'static str = "M";
}
impl Scale for Giga {
    const FACTOR: f32 = 1e9;
    const SYMBOL: &'static str = "G";
}

#[allow(missing_docs)]
pub trait Unit: Sized {
    type Storage: Scale;

    const SYMBOL: &'static str;
    const ZERO: Self;

    fn new(raw: f32) -> Self;

    fn raw(&self) -> f32;

    fn zero() -> Self {
        Unit::new(0.0)
    }

    fn one() -> Self {
        Unit::new(1.0)
    }

    fn two() -> Self {
        Unit::new(2.0)
    }

    fn from<S: Scale>(value: f32) -> Self {
        Unit::new(value * S::FACTOR / Self::Storage::FACTOR)
    }

    fn to<S: Scale>(&self) -> f32 {
        self.raw() / S::FACTOR * Self::Storage::FACTOR
    }

    fn pretty<S: Scale>(self) -> String {
        format!("{:>8.0} {}{}", self.to::<S>(), S::SYMBOL, Self::SYMBOL)
    }

    // Units scale as well with mul and div so for now this is explicitly left
    // as a seperate implementation
    fn mul_by(&self, rhs: f32) -> f32 {
        self.raw() * rhs
    }

    // Units scale as well with mul and div so for now this is explicitly left
    // as a seperate implementation
    fn div_by(&self, rhs: f32) -> f32 {
        self.raw() / rhs
    }
}

impl PartialOrd for Coulomb {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Add for Coulomb {
    type Output = Coulomb;
    fn add(mut self, rhs: Self) -> Self::Output {
        self.0 = self.0 + rhs.0;
        self
    }
}

impl AddAssign for Coulomb {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Coulomb {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Coulomb {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl PartialOrd for Ampere {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Add for Ampere {
    type Output = Ampere;
    fn add(mut self, rhs: Self) -> Self::Output {
        self.0 = self.0 + rhs.0;
        self
    }
}

impl AddAssign for Ampere {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Ampere {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Ampere {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_pretty<S: Scale>(val: f32) -> String {
        let x: Ampere = Unit::from::<S>(val);
        x.pretty::<S>()
    }

    #[test]
    fn pretty_unit() {
        assert_eq!("1.234568 nA", gen_pretty::<Nano>(1.23456789));
        assert_eq!("1.2345679 µA", gen_pretty::<Micro>(1.23456789));
        assert_eq!("1.2345679 mA", gen_pretty::<Milli>(1.23456789));
        assert_eq!("1.2345679 A", gen_pretty::<Base>(1.23456789));
        assert_eq!("1.2345679 kA", gen_pretty::<Kilo>(1.23456789));
        assert_eq!("1.2345679 MA", gen_pretty::<Mega>(1.23456789));
        assert_eq!("1.2345679 GA", gen_pretty::<Giga>(1.23456789));

        assert_eq!("1234567.9 nA", gen_pretty::<Nano>(1234567.89));
        assert_eq!("1234567.9 µA", gen_pretty::<Micro>(1234567.89));
        assert_eq!("1234567.8 mA", gen_pretty::<Milli>(1234567.89));
        assert_eq!("1234567.9 A", gen_pretty::<Base>(1234567.89));
        assert_eq!("1234567.9 kA", gen_pretty::<Kilo>(1234567.89));
        assert_eq!("1234567.9 MA", gen_pretty::<Mega>(1234567.89));
        assert_eq!("1234567.9 GA", gen_pretty::<Giga>(1234567.89));
    }

    #[test]
    fn unit_converts() {
        let base: Ampere = Unit::from::<Base>(1.0);
        let milli: Ampere = Unit::from::<Milli>(1.0);
        let micro: Ampere = Unit::from::<Micro>(1.0);
        assert!(base > milli);
        assert!(milli > micro);
    }
}
