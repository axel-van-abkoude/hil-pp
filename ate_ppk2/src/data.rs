//! Datastructures that store the experiment data

use std::{
    fmt::{Debug, Display},
    fs::File,
    ops::Range,
    path::Path,
};

use polars::{
    df,
    frame::DataFrame,
    io::SerReader,
    prelude::{
        ChunkAgg, ChunkedArray, Float64Type, NamedFrom, ParquetReader, ParquetWriter
    },
    series::Series,
};
use ppk2::types::Level;
use serde::{Deserialize, Deserializer, Serialize, ser::Serializer};
use uom::si::{
    Unit, electric_charge::coulomb, electric_current::microampere, f64::{ElectricCharge, ElectricCurrent, Ratio, Time}, ratio::basis_point, time::microsecond
};

use crate::logic::Pins;

#[allow(missing_docs)]
pub fn axis<A: Axis>(df: &DataFrame) -> &ChunkedArray<Float64Type> {
    df.column(A::header().as_str()).unwrap().f64().unwrap()
}

#[allow(missing_docs)]
pub fn axis_iter<A: Axis>(df: &DataFrame) -> impl Iterator<Item = f64> {
    axis::<A>(df).into_no_null_iter()
}

#[allow(missing_docs)]
pub fn axis_zip<X: Axis, Y: Axis>(df: &DataFrame) -> impl Iterator<Item = (f64, f64)> {
    axis_iter::<X>(df).zip(axis_iter::<Y>(df))
}

pub fn slice_on(df: &DataFrame, on: String, level: Level) -> Vec<DataFrame> {
    let mask = df.column(on.as_str()).unwrap().bool().unwrap();
    let mut slices = Vec::new();
    let mut start = None;

    for (i, v) in mask.no_null_iter().enumerate() {
        match (level, v, start) {
            (Level::High, true, None) => start = Some(i),
            (Level::High, false, Some(s)) => {
                slices.push(df.slice(s as i64, i - s));
                start = None;
            }
            (Level::Low, false, None) => start = Some(i),
            (Level::Low, true, Some(s)) => {
                slices.push(df.slice(s as i64, i - s));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        slices.push(df.slice(s as i64, mask.len() - s));
    }

    slices
}

pub fn load_dataframe(file: &Path) -> DataFrame {
    let mut file = File::open(file).unwrap();
    ParquetReader::new(&mut file).finish().unwrap()
}

pub fn store_dataframe(df: &mut DataFrame, file: &Path) {
    let mut file = File::create(file).unwrap();
    ParquetWriter::new(&mut file).finish(df).unwrap();
}

#[derive(Debug)]
#[allow(missing_docs)]
pub struct Metrics<A: Axis> {
    /// Max value
    pub max: A::Unit,
    /// Min value
    pub min: A::Unit,
    ///  Average value
    pub avg: A::Unit,
    /// Range of values
    pub range: Range<f64>,
}

impl<A: Axis> Display for Metrics<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n---- {} \nMAX {:>10} MIN {:>10} AVG {:>10}\n====",
            A::header(),
            A::pretty(self.max),
            A::pretty(self.min),
            A::pretty(self.avg),
        )
    }
}

#[allow(missing_docs)]
impl Metrics<Current> {
    pub fn new(df: &DataFrame) -> Self {
        let max = axis::<Current>(df).max().unwrap();
        let min = axis::<Current>(df).min().unwrap();

        let ch_sum = Charge::load(axis::<Charge>(df).sum().unwrap());
        let tim_max = Timestamp::load(axis::<Timestamp>(df).max().unwrap());

        Self {
            max: Current::load(max),
            min: Current::load(min),
            avg: ch_sum/tim_max,
            range: min..max,
        }
    }
}

impl Metrics<Latency> {
    pub fn new(df: &DataFrame) -> Self {
        let max = axis::<Latency>(df).max().unwrap();
        let min = axis::<Latency>(df).min().unwrap();
        let avg = axis::<Latency>(df).mean().unwrap();

        Self {
            max: Latency::load(max),
            min: Latency::load(min),
            avg: Latency::load(avg),
            range: min..max,
        }
    }
}

#[allow(missing_docs)]
pub trait Axis: Sized {
    const NAME: &'static str;
    type Unit: Debug + Copy + Clone;
    type StoredAs: Unit;

    fn load(val: f64) -> Self::Unit;
    fn store(unit: Self::Unit) -> f64;
    fn pretty(unit: Self::Unit) -> String;

    fn header() -> String {
        format!("{} ({})", Self::NAME, Self::StoredAs::abbreviation())
    }

    fn range(data: &DataFrame) -> Range<f64> {
        let max = axis::<Self>(data).max().unwrap();
        let min = axis::<Self>(data).min().unwrap();
        min..max
    }

    fn plot_bounds(data: &DataFrame) -> Range<f64> {
        let max = axis::<Self>(data).max().unwrap();
        let min = axis::<Self>(data).min().unwrap();
        min..max*1.1
    }
}

#[allow(missing_docs)]
pub struct Latency {}
impl Axis for Latency {
    const NAME: &'static str = "Latency";
    type Unit = Time;
    type StoredAs = microsecond;

    fn load(val: f64) -> Self::Unit {
        Self::Unit::new::<Self::StoredAs>(val)
    }

    fn store(unit: Self::Unit) -> f64 {
        unit.get::<Self::StoredAs>()
    }

    fn pretty(unit: Self::Unit) -> String {
        format!("{:.3}",
            Self::Unit::format_args(microsecond, uom::fmt::DisplayStyle::Abbreviation).with(unit)
        )
    }
}

#[allow(missing_docs)]
pub struct Samples {}
impl Axis for Samples {
    const NAME: &'static str = "Samples";
    type Unit = Ratio;
    type StoredAs = basis_point;

    fn load(val: f64) -> Self::Unit {
        Self::Unit::new::<Self::StoredAs>(val)
    }

    fn store(unit: Self::Unit) -> f64 {
        unit.get::<Self::StoredAs>()
    }
    fn pretty(unit: Self::Unit) -> String {
        format!("{:.3}",
            Self::Unit::format_args(basis_point, uom::fmt::DisplayStyle::Abbreviation).with(unit)
        )
    }
}

#[allow(missing_docs)]
pub struct Timestamp {}
impl Axis for Timestamp {
    const NAME: &'static str = "Timestamp";
    type Unit = Time;
    type StoredAs = microsecond;

    fn load(val: f64) -> Self::Unit {
        Self::Unit::new::<Self::StoredAs>(val)
    }

    fn store(unit: Self::Unit) -> f64 {
        unit.get::<Self::StoredAs>()
    }
    fn pretty(unit: Self::Unit) -> String {
        format!("{:.3}",
            Self::Unit::format_args(microsecond, uom::fmt::DisplayStyle::Abbreviation).with(unit)
        )
    }
}

#[allow(missing_docs)]
pub struct Current {}
impl Axis for Current {
    const NAME: &'static str = "Current";
    type Unit = ElectricCurrent;
    type StoredAs = microampere;

    fn load(val: f64) -> Self::Unit {
        Self::Unit::new::<Self::StoredAs>(val)
    }

    fn store(unit: Self::Unit) -> f64 {
        unit.get::<Self::StoredAs>()
    }
    fn pretty(unit: Self::Unit) -> String {
        format!("{:.3}",
            Self::Unit::format_args(microampere, uom::fmt::DisplayStyle::Abbreviation).with(unit)
        )
    }
}

pub struct Charge {}
impl Axis for Charge {
    const NAME: &'static str = "Charge";
    type Unit = ElectricCharge;
    type StoredAs = coulomb;

    fn load(val: f64) -> Self::Unit {
        Self::Unit::new::<Self::StoredAs>(val)
    }

    fn store(unit: Self::Unit) -> f64 {
        unit.get::<Self::StoredAs>()
    }
    fn pretty(unit: Self::Unit) -> String {
        format!("{:.3}",
            Self::Unit::format_args(coulomb, uom::fmt::DisplayStyle::Abbreviation).with(unit)
        )
    }
}

#[allow(non_snake_case)]
pub struct Buffers {
    acc: DataFrame,
    D0_buffer: Vec<bool>,
    D1_buffer: Vec<bool>,
    D2_buffer: Vec<bool>,
    D3_buffer: Vec<bool>,
    D4_buffer: Vec<bool>,
    D5_buffer: Vec<bool>,
    D6_buffer: Vec<bool>,
    D7_buffer: Vec<bool>,
    tim_buffer: Vec<f64>,
    lat_buffer: Vec<f64>,
    cur_buffer: Vec<f64>,
    chr_buffer: Vec<f64>,
}

impl Buffers {
    pub fn new(batch_size: usize) -> Buffers {
        let acc = DataFrame::default();
        #[allow(non_snake_case)]
        let D0_buffer = Vec::with_capacity(batch_size);
        #[allow(non_snake_case)]
        let D1_buffer = Vec::with_capacity(batch_size);
        #[allow(non_snake_case)]
        let D2_buffer = Vec::with_capacity(batch_size);
        #[allow(non_snake_case)]
        let D3_buffer = Vec::with_capacity(batch_size);
        #[allow(non_snake_case)]
        let D4_buffer = Vec::with_capacity(batch_size);
        #[allow(non_snake_case)]
        let D5_buffer = Vec::with_capacity(batch_size);
        #[allow(non_snake_case)]
        let D6_buffer = Vec::with_capacity(batch_size);
        #[allow(non_snake_case)]
        let D7_buffer = Vec::with_capacity(batch_size);
        let tim_buffer = Vec::with_capacity(batch_size);
        let lat_buffer = Vec::with_capacity(batch_size);
        let cur_buffer = Vec::with_capacity(batch_size);
        let chr_buffer = Vec::with_capacity(batch_size);
        Self {
            acc,
            D0_buffer,
            D1_buffer,
            D2_buffer,
            D3_buffer,
            D4_buffer,
            D5_buffer,
            D6_buffer,
            D7_buffer,
            tim_buffer,
            lat_buffer,
            cur_buffer,
            chr_buffer,
        }
    }

    pub fn push(
        &mut self,
        &Sample {
            timestamp,
            latency,
            current,
            pins,
        }: &Sample,
    ) {
        self.tim_buffer.push(timestamp.get::<<Timestamp as Axis>::StoredAs>());
        self.lat_buffer.push(latency.get::<<Latency as Axis>::StoredAs>());
        self.cur_buffer.push(current.get::<<Current as Axis>::StoredAs>());
        self.chr_buffer.push((current*latency).get::<<Charge as Axis>::StoredAs>());
        self.D0_buffer.push(pins.is_d0());
        self.D1_buffer.push(pins.is_d1());
        self.D2_buffer.push(pins.is_d2());
        self.D3_buffer.push(pins.is_d3());
        self.D4_buffer.push(pins.is_d4());
        self.D5_buffer.push(pins.is_d5());
        self.D6_buffer.push(pins.is_d6());
        self.D7_buffer.push(pins.is_d7());

        if self.tim_buffer.capacity() <= self.tim_buffer.len() {
            self.flush();
        }
    }

    fn flush(&mut self) {
        let df = &df! {
            "D0" => &mut *self.D0_buffer,
            "D1" => &mut *self.D1_buffer,
            "D2" => &mut *self.D2_buffer,
            "D3" => &mut *self.D3_buffer,
            "D4" => &mut *self.D4_buffer,
            "D5" => &mut *self.D5_buffer,
            "D6" => &mut *self.D6_buffer,
            "D7" => &mut *self.D7_buffer,
            Timestamp::header() => &mut *self.tim_buffer,
            Latency::header() => &mut *self.lat_buffer,
            Current::header() => &mut *self.cur_buffer,
            Charge::header() => &mut *self.chr_buffer,
        }
        .unwrap();
        self.acc.vstack_mut(&df).unwrap();

        self.tim_buffer.clear();
        self.lat_buffer.clear();
        self.cur_buffer.clear();
        self.chr_buffer.clear();
        self.D0_buffer.clear();
        self.D1_buffer.clear();
        self.D2_buffer.clear();
        self.D3_buffer.clear();
        self.D4_buffer.clear();
        self.D5_buffer.clear();
        self.D6_buffer.clear();
        self.D7_buffer.clear();
    }

    pub fn finish(mut self) -> DataFrame {
        self.flush();
        // Including samples
        let smp = Series::new(
            Samples::header().into(),
            (0..self.acc.height())
                .map(|x| x as f64)
                .collect::<Vec<f64>>(),
        );
        self.acc.with_column(smp.into()).unwrap();
        self.acc
    }
}

/// A single sample of a measurement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    /// The time from the beginning of the measurement.
    pub timestamp: <Timestamp as Axis>::Unit,
    /// The time it took to measure this sample.
    pub latency: <Latency as Axis>::Unit,
    /// The current measured by the ppk2.
    pub current: <Current as Axis>::Unit,
    /// The state of the logic port pins
    #[serde(rename = "Logic Pins Sample (D0-D7)")]
    #[serde(serialize_with = "ser_pins_str")]
    #[serde(deserialize_with = "de_pins_str")]
    pub pins: Pins,
}

fn ser_pins_str<S>(pins: &Pins, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&pins.to_string())
}

fn de_pins_str<'de, D>(deserializer: D) -> Result<Pins, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_bytes() {
        bytes if bytes.len() != 8 => Err(serde::de::Error::custom("Expecting 8 bytes")),
        bytes => Ok(Pins::from_bytes(bytes)),
    }
}
