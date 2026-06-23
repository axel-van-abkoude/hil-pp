//! Logic that gives context to measurements or instructs what is measured

use std::ops::{BitAnd, BitOr, Not};

use crate::data::Sample;
use ppk2::types::{Level, LogicPortPins};
use std::iter::zip;
use uom::si::{f64::ElectricCurrent, f64::Time};

#[derive(Debug, Clone)]
/// Predicate for when a measurement should be started or ended
pub enum When {
    /// Always evaluates to true
    Now,
    /// Always evaluates to false
    Never,
    /// An amount of time has elapsed
    Duration(Time),
    /// The latency is greater than
    LatencyGt(Time),
    /// A mark has been identified via a pin configuration
    Logic(Pins),
    /// The Current is greater than a value
    CurrentGt(ElectricCurrent),
    /// The Current is less than a value
    CurrentLt(ElectricCurrent),
    /// Negates the predicate
    Not(Box<When>),
    /// Logical AND
    And(Box<When>, Box<When>),
    /// Logical OR
    Or(Box<When>, Box<When>),
}

impl When {
    /// Evaluates the predicate from the information given.
    pub fn eval(
        &self,
        sample @ Sample {
            timestamp,
            latency,
            current,
            pins,
        }: &Sample,
    ) -> bool {
        use When::*;
        match self {
            Now => true,
            Never => false,
            Duration(pred_timestamp) => timestamp > pred_timestamp,
            LatencyGt(pred_latency) => latency > pred_latency,
            Logic(pred_pins) => pins.matches(pred_pins),
            CurrentGt(pred_current) => current > pred_current,
            CurrentLt(pred_current) => current < pred_current,
            Not(pred) => !pred.eval(sample),
            And(left, right) => left.eval(sample) && right.eval(sample),
            Or(left, right) => left.eval(sample) || right.eval(sample),
        }
    }
}

impl BitAnd for When {
    type Output = When;
    fn bitand(self, y: When) -> When {
        When::And(Box::new(self), Box::new(y))
    }
}

impl BitOr for When {
    type Output = When;
    fn bitor(self, y: When) -> When {
        When::And(Box::new(self), Box::new(y))
    }
}

impl Not for When {
    type Output = When;
    fn not(self) -> When {
        When::Not(Box::new(self))
    }
}

/// The status of the measurement.
///
/// Used in [Setup::measure] to keep track if we are waiting for a predicate to
/// start measuring or if we are measuring until a predicate holds to stop the
/// measurement.
#[derive(Debug, Clone)]
pub enum MeasureStatus {
    /// Indicates that we are waiting for a condition to hold
    Waiting,
    /// Indicates that a measurement is taking place
    Measuring,
}

#[derive(Copy, Debug, Clone)]
/// Implementation of ppk2-rs LogicPortPins
pub struct Pins(LogicPortPins);
#[allow(missing_docs)]
impl Pins {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut arr = [Level::Either; 8];
        for i in 0..8 {
            arr[i] = match bytes[i] {
                b'1' => Level::High,
                b'0' => Level::Low,
                _ => todo!("Error"),
            };
        }
        Self(LogicPortPins::with_levels(arr))
    }
    pub fn is_d0(self) -> bool {
        self.0.pin_is_high(0)
    }
    pub fn is_d1(self) -> bool {
        self.0.pin_is_high(1)
    }
    pub fn is_d2(self) -> bool {
        self.0.pin_is_high(2)
    }
    pub fn is_d3(self) -> bool {
        self.0.pin_is_high(3)
    }
    pub fn is_d4(self) -> bool {
        self.0.pin_is_high(4)
    }
    pub fn is_d5(self) -> bool {
        self.0.pin_is_high(5)
    }
    pub fn is_d6(self) -> bool {
        self.0.pin_is_high(6)
    }
    pub fn is_d7(self) -> bool {
        self.0.pin_is_high(7)
    }

    pub fn pin_high(d: usize) -> Self {
        let mut arr = [Level::Either; 8];
        arr[d] = Level::High;
        Self(LogicPortPins::with_levels(arr))
    }

    pub fn pin_low(d: usize) -> Self {
        let mut arr = [Level::Either; 8];
        arr[d] = Level::Low;
        Self(LogicPortPins::with_levels(arr))
    }

    pub fn matches(&self, other: &Self) -> bool {
        for (inner_self, inner_other) in zip(self.0.inner(), other.0.inner()) {
            match (inner_self, inner_other) {
                (Level::Low, Level::High) => return false,
                (Level::High, Level::Low) => return false,
                _ => continue,
            }
        }
        return true;
    }
}

impl Into<Pins> for LogicPortPins {
    fn into(self) -> Pins {
        Pins(self)
    }
}

impl PartialEq for Pins {
    fn eq(&self, other: &Self) -> bool {
        for (inner_self, inner_other) in zip(self.0.inner(), other.0.inner()) {
            if PinLevel(*inner_self) != PinLevel(*inner_other) {
                return false;
            }
        }
        return true;
    }
}

impl Into<u8> for Pins {
    fn into(self) -> u8 {
        let mut ret: u8 = 0;
        for (i, pin) in self.0.inner().iter().enumerate() {
            match pin {
                Level::Low => {
                    continue;
                }
                Level::High => {
                    ret |= 1 << i;
                }
                Level::Either => {
                    todo!("either in u8::from");
                }
            }
        }
        ret
    }
}

impl Into<Pins> for u8 {
    fn into(self) -> Pins {
        Pins(LogicPortPins::from(self))
    }
}

impl ToString for Pins {
    fn to_string(&self) -> String {
        self.0
            .inner()
            .iter()
            .map(|p| char::from(PinLevel(*p)))
            .collect()
    }
}

#[derive(Debug, Clone)]
/// Implementation of ppk2-rs Level
pub struct PinLevel(Level);

impl PinLevel {
    ///
    pub const LOW: PinLevel = PinLevel(Level::Low);
    ///
    pub const HIGH: PinLevel = PinLevel(Level::High);
}

impl PartialEq for PinLevel {
    fn eq(&self, other: &Self) -> bool {
        match (self.0, other.0) {
            (Level::Low, Level::Low) => true,
            (Level::High, Level::High) => true,

            // Can be used for queries on sections
            (Level::Either, _) => true,
            (_, Level::Either) => true,

            _ => false,
        }
    }
}

impl From<Level> for PinLevel {
    fn from(value: Level) -> Self {
        Self(value)
    }
}

/// Implementation to parse a PinLevel from a char used in deserialisation
impl From<u8> for PinLevel {
    fn from(c: u8) -> Self {
        match c {
            b'0' => PinLevel(Level::Low),
            b'1' => PinLevel(Level::High),
            _ => PinLevel(Level::Either),
        }
    }
}

impl From<char> for PinLevel {
    fn from(c: char) -> Self {
        match c {
            '0' => PinLevel(Level::Low),
            '1' => PinLevel(Level::High),
            _ => PinLevel(Level::Either),
        }
    }
}

impl From<PinLevel> for char {
    fn from(level: PinLevel) -> Self {
        match level.0 {
            Level::Low => '0',
            Level::High => '1',
            Level::Either => 'x',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inverses_level() {
        assert_eq!(
            PinLevel(Level::Low),
            PinLevel::from(char::from(PinLevel(Level::Low)))
        );
        assert_eq!(
            PinLevel(Level::High),
            PinLevel::from(char::from(PinLevel(Level::High)))
        );
        assert_eq!(
            PinLevel(Level::Either),
            PinLevel::from(char::from(PinLevel(Level::Either)))
        );
    }

    #[test]
    fn test_inverses_char() {
        assert_eq!('0', char::from(PinLevel::from(b'0')));
        assert_eq!('1', char::from(PinLevel::from(b'1')));
        assert_eq!('x', char::from(PinLevel::from(b'x')));
        assert_eq!('x', char::from(PinLevel::from(b'\x00')));
    }
}
