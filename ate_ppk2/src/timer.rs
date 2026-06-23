//! Types that store SI units which ensures we keep data correct through conversions.

#[cfg(target_os = "linux")]
use libc::{CLOCK_MONOTONIC_RAW, clock_gettime, timespec};

use uom::fmt::DisplayStyle::*;
use uom::si::{
    f64::Time,
    time::{microsecond, nanosecond, second},
};

use crate::error::{HILppError::*, Result};

/// The standard library gives us the monotonic clock
/// This is corrected by NTP, which we do not want in measurements of micro seconds
/// Thus we want the raw monotonic clock (CLOCK_MONOTONIC_RAW)
#[derive(Debug, Clone, Copy)]
pub struct Timer(Time);

/// Use audio_thread_priority
impl Timer {
    #[allow(missing_docs)]
    pub fn start() -> Result<Timer> {
        Ok(Timer(Self::get_raw_time()?))
    }

    #[allow(missing_docs)]
    pub fn elapsed(self) -> Result<Time> {
        let ret = Self::get_raw_time()? - self.0;
        match ret.is_sign_negative() {
            true => Err(TimeError(format!(
                "Negative elapsed time: {}",
                Time::format_args(microsecond, Abbreviation).with(ret)
            ))),
            false => Ok(ret),
        }
    }

    #[allow(missing_docs)]
    pub fn reset(&mut self) -> Result<()> {
        let now = Self::get_raw_time()?;
        self.0 = now;
        Ok(())
    }

    fn get_raw_time() -> Result<Time> {
        let mut ts = timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };

        // Safety:
        // We ensure that ts is the correct size by declaring the object before.
        // As we rethrow the libc error if there is one
        // We can assume safety as long as libc gives an error when it fails
        unsafe {
            if clock_gettime(CLOCK_MONOTONIC_RAW, &mut ts) != 0 {
                return Err(Libc(std::io::Error::last_os_error()));
            }
        }

        Ok(Time::new::<second>(ts.tv_sec as f64) + Time::new::<nanosecond>(ts.tv_nsec as f64))
    }
}
