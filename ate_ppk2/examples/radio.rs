//!
//! An example that measures the power consumption of an example with different
//! measurement rates.
//!

use std::{path::Path, process::Command};

use ate_ppk2::{
    Rate, Setup,
    data::{Axis, Current, Latency, Metrics, Samples, Timestamp, load_dataframe, store_dataframe},
    logic::{Pins, When::*},
    plot::*,
};
use plotters::style::{BLUE, RED, full_palette::{BROWN, GREEN_700, ORANGE, PURPLE}};
use polars::{frame::DataFrame, prelude::{ChunkAgg, DataFrameJoinOps, IntoLazy, NamedFrom, col}, series::Series};

fn main() {
    let experiment = "radio";
    let mut setup = Setup::find();
    setup.rate = Rate::FINE;

    setup.flash(
        Path::new("../dut_nrf52840"),
        Command::new("cargo")
            .arg("flash")
            .arg("--chip")
            .arg("nRF52840_xxAA")
            .arg("--release")
            .arg("--bin")
            .arg(experiment),
    );

    //MEASUREMENT

    let mut df_each = setup.measure(
        Logic(Pins::pin_low(0)),
        !Logic(Pins::pin_low(0))
    );
    println!("{}", df_each);

    let mut df_once = setup.measure(
        Logic(Pins::pin_low(4)),
        !Logic(Pins::pin_low(4))
    );
    println!("{}", df_once);

    // STORAGE

    //store_dataframe(&mut df_each, Path::new("data/radio_each.parquet"));
    //store_dataframe(&mut df_once, Path::new("data/radio_once.parquet"));

    let df_each = load_dataframe(Path::new("data/radio_each.parquet"));
    let df_once = load_dataframe(Path::new("data/radio_once.parquet"));

    // TIMESTAMP vs CURRENT

    // PLOTTING TWO LINES
    let mut plot = Plot::<Timestamp, Current>::new(
        &df_each,
        "Current Comparison of \'Each\' and \'Once\'".to_string(),
        Path::new("plots/radio_cmp_tim.png"),
    );
    plot.draw_line(&df_once, BLUE, "once".to_string());
    plot.draw_line(&df_each, GREEN_700, "each".to_string());
    plot.present();

    // PLOTTING DIGITAL PINS

    let mut plot = Plot::<Timestamp, Current>::new(
        &df_each,
        "Turn on Radio for each message (Each)".to_string(),
        Path::new("plots/radio_each_tim.png"),
    );
    plot.draw_line(&df_each, GREEN_700, "Each".to_string());
    plot.draw_poi_bounds(&df_each,RED,"D0".to_string(), "POI".to_string());
    plot.draw_poi_bounds(&df_each,ORANGE,"D1".to_string(), "on()/off()".to_string());
    plot.present();

    let mut plot = Plot::<Timestamp, Current>::new(
        &df_each,
        "Turn on Radio per message (Once)".to_string(),
        Path::new("plots/radio_once_tim.png"),
    );
    plot.draw_line(&df_once, BLUE, "Once".to_string());
    plot.draw_poi_bounds(&df_once,PURPLE,"D4".to_string(), "POI".to_string());
    plot.draw_poi_bounds(&df_once,ORANGE,"D5".to_string(), "on()/off()".to_string());
    plot.present();

    // TIMESTAMP vs CURRENT

    // PLOTTING TWO LINES
    let mut plot = Plot::<Samples, Current>::new(
        &df_each,
        "Current Comparison of \'Each\' and \'Once\'".to_string(),
        Path::new("plots/radio_cmp_smp.png"),
    );
    plot.draw_line(&df_once, BLUE, "once".to_string());
    plot.draw_line(&df_each, GREEN_700, "each".to_string());
    plot.present();

    // PLOTTING DIGITAL PINS

    let mut plot = Plot::<Samples, Current>::new(
        &df_each,
        "Turn on Radio for each message (Each)".to_string(),
        Path::new("plots/radio_each_smp.png"),
    );
    plot.draw_line(&df_each, GREEN_700, "Each".to_string());
    plot.draw_poi_bounds(&df_each,PURPLE,"D0".to_string(), "POI".to_string());
    plot.draw_poi_bounds(&df_each,ORANGE,"D1".to_string(), "on()/off()".to_string());
    plot.present();

    let mut plot = Plot::<Samples, Current>::new(
        &df_each,
        "Turn on Radio per message (Once)".to_string(),
        Path::new("plots/radio_once_smp.png"),
    );
    plot.draw_line(&df_once, BLUE, "Once".to_string());
    plot.draw_poi_bounds(&df_once,PURPLE,"D4".to_string(), "POI".to_string());
    plot.draw_poi_bounds(&df_once,ORANGE,"D5".to_string(), "on()/off()".to_string());
    plot.present();


    // PLOTTING TWO LINES
    let mut plot = Plot::<Samples, Latency>::new(
        &df_each,
        "Latency Plot for Radio".to_string(),
        Path::new("plots/radio_cmp_lat.png"),
    );
    plot.draw_line(&df_each, GREEN_700, "each".to_string());
    plot.present();

    // // METRICS
    // println!("Each {}", Metrics::<Current>::new(&df_each));
    // println!("Once {}\n", Metrics::<Current>::new(&df_once));
    // println!("Each {}", Metrics::<Latency>::new(&df_each));
    // println!("Once {}", Metrics::<Latency>::new(&df_once));
}
