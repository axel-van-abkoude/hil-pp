//! Plotting code
use plotters::{
    chart::{ChartBuilder, ChartContext},
    coord::{Shift, types::RangedCoordf64},
    prelude::{
        BitMapBackend, Cartesian2d, DrawingArea, IntoDrawingArea, PathElement, Rectangle
    },
    series::LineSeries,
    style::{MAGENTA, RGBColor, ShapeStyle, full_palette::*},
};
use polars::frame::DataFrame;
use std::{marker::PhantomData, ops::Range, path::Path};

use crate::data::{Axis, axis_zip, slice_on};

/// A Plot with 2 specified Axis
/// These axis ensure that we can draw multiple lines to a plot but all lines
/// have the same underlying datatype.
pub struct Plot<'a, X: Axis, Y: Axis> {
    area: DrawingArea<BitMapBackend<'a>, Shift>,
    chart: ChartContext<'a, BitMapBackend<'a>, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    _x: PhantomData<X>,
    _y: PhantomData<Y>,
}

impl<'a, X: Axis, Y: Axis> Plot<'a, X, Y> {
    const FONT: (&'static str, f32) = ("sans-serif", 20.0);
    #[allow(missing_docs)]
    pub fn new(df: &'a DataFrame, caption: String, out_path: &'a Path) -> Self {
        let area = BitMapBackend::new(out_path, (1024, 768)).into_drawing_area();
        area.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&area)
            .caption(caption, Self::FONT)
            .margin(40)
            .x_label_area_size(60u32)
            .y_label_area_size(60u32)
            .build_cartesian_2d(
                X::plot_bounds(df),
                Y::plot_bounds(df),
            )
            .unwrap();

        chart
            .configure_mesh()
            .x_desc(X::header())
            .y_desc(Y::header())
            .draw()
            .unwrap();

        Plot {
            area,
            _x: PhantomData,
            _y: PhantomData,
            chart,
        }
    }

    #[allow(missing_docs)]
    pub fn draw_line(&mut self, data: &DataFrame, color: RGBColor, label: String) {
        self.chart
            .draw_series(LineSeries::new(
                axis_zip::<X, Y>(data),
                ShapeStyle::from(color).stroke_width(1),
            ))
            .unwrap()
            .label(label)
            .legend(move |(x, y)| PathElement::new([(x, y), (x + 10, y)], color));
    }

    pub fn draw_all_bounds(&mut self, data: &DataFrame) {
        let labels: [String; 8] = [
            "D0".to_string(),
            "D1".to_string(),
            "D2".to_string(),
            "D3".to_string(),
            "D4".to_string(),
            "D5".to_string(),
            "D6".to_string(),
            "D7".to_string(),
        ];
        let colors: [&RGBColor; 8] = [
            &BLUE, &RED, &GREEN, &CYAN, &MAGENTA, &YELLOW, &BLACK, &PURPLE,
        ];
        for (l, c) in labels.iter().zip(colors) {
            println!("{:?}{:?}", l, c);
            self.draw_poi_bounds(&data, *c, l.clone(), l.clone());
        }
    }
    pub fn draw_poi_bounds(&mut self, data: &DataFrame, color: RGBColor, id: String, label: String) {
        self.chart
            .draw_series(LineSeries::new(
                [(0.0, 0.0)],
                ShapeStyle::from(color).stroke_width(0),
            ))
            .unwrap()
            .label(label.clone())
            .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], color));

        for poi in slice_on(data, id.clone(), ppk2::types::Level::Low) {
            //println!("{}",poi);
            self.draw_bound(&poi, color);
        }
    }

    pub fn draw_bound(&mut self, df: &DataFrame, color: RGBColor) {
        let Range {
                start: xmin,
                end: xmax,
            } = X::range(df);

        self.chart.draw_series(std::iter::once(Rectangle::new(
            [(xmin, 0.0), (xmax, f64::MAX)],
            color,
        ))).unwrap();

        // self.chart
        //     .draw_series(LineSeries::new(vec![(xmin, 0.0), (xmin, f64::MAX)], color))
        //     .unwrap();

        // self.chart
        //     .draw_series(LineSeries::new(vec![(xmax, 0.0), (xmax, f64::MAX)], color))
        //     .unwrap();
    }

    #[allow(missing_docs)]
    pub fn present(&mut self) {
        self.chart
            .configure_series_labels()
            .border_style(&BLACK)
            .background_style(&YELLOW_50)
            .draw()
            .unwrap();

        self.area.present().unwrap()
    }
}
