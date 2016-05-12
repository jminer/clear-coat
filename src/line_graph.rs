/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use super::control_prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::ops::Range;
use super::{
    Canvas,
    CanvasActionArgs,
};

pub struct DataPoint {
    x: f64,
    y: f64,
}

pub struct MergedDataPoint {
    min_y: f64,
    max_y: f64,
}

/// Don't create pyramids smaller than this many points because they are less likely to be used.
/// Should be a little larger than the smallest a graph will typically be.
const MIN_PYRAMID_LEVEL_SIZE: usize = 1000;

/// A [pyramid] level is a representation of a `DataSeries` with a reduced number of points. In
/// images, this structure is called a [mipmap]. Each point is a constant x value apart.
///
/// [pyramid]: https://en.wikipedia.org/wiki/Pyramid_%28image_processing%29
/// [mipmap]: https://en.wikipedia.org/wiki/Mipmap
pub struct PyramidLevel {
    // The x value of the first point in `merged_points`.
    min_x: f64,
    // The difference in the value of x between each point in `merged_points`. This value should
    // be a power-of-two (like 0.25, 0.5, 1.0, 2.0, 4.0)
    x_step: f64,
    // Each point contains the min and max y values over the
    // range (min_x + x_step*n)..(min_x + x_step*(n+1)).
    merged_points: Vec<MergedDataPoint>,
}

pub struct DataSeries {
    // Sorted by each point's x value.
    points: Vec<DataPoint>,
    axis: u8,

    pyramid: Vec<PyramidLevel>,
}

pub struct LineGraphData {
    canvas: Canvas,

    // the order of DataSeries determines the z-order on screen
    series: Vec<DataSeries>,
    x_axis: Range<f64>,
    y_axes: Vec<Range<f64>>,
}

#[derive(Clone)]
pub struct LineGraph(Rc<RefCell<LineGraphData>>);

/*
To draw the screen, the mipmap >= to the number of pixels on screen is chosen and reduced to
a list of min & max values, one for each pixel.

The midmaps for a DataSeries could be calculated on a different thread. When it is done, the midmap
is swapped in, and painting becomes faster.
*/

impl LineGraph {
    pub fn new() -> Self {
        let canvas = Canvas::new();
        let data = Rc::new(RefCell::new(LineGraphData {
            canvas: canvas.clone(),
            series: vec![],
            x_axis: 0.0..1.0,
            y_axes: vec![0.0..1.0],
        }));

        let data2 = data.clone();
        canvas.action_event().add(move |args: &CanvasActionArgs|
            LineGraph(data2.clone()).when_painting(args)
        );

        LineGraph(data)
    }

    fn when_painting(&self, args: &CanvasActionArgs) {

    }

    pub fn autoscale_x(&self) {
        let mut data = self.0.borrow_mut();
        let (mut min_x, mut max_x) = (None, None);
        for series in data.series.iter() {
            if let Some(pt) = series.points.first() {
                min_x = Some(min_x.map_or(pt.x, |mx| pt.x.min(mx)));
            }
            if let Some(pt) = series.points.last() {
                max_x = Some(max_x.map_or(pt.x, |mx| pt.x.max(mx)));
            }
        }
        if let (Some(min_x), Some(max_x)) = (min_x, max_x) {
            data.x_axis = min_x..max_x;
        }
    }

    pub fn autoscale_all() {

    }
}
