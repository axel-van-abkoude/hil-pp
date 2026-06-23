use ate_ppk2::{data::*, plot::Plot};
use plotters::style::{BLUE};
use std::path::Path;

fn main() {
    let df = load_dataframe(Path::new("data/trace.parquet"));

    let mut plot = Plot::<Timestamp, Current>::new(
        &df,
        "Trace".to_string(),
        Path::new("plots/trace.png"),
    );
    plot.draw_line(&df, BLUE, "trace".to_string());
    plot.draw_all_bounds(&df);
    plot.present();
}
