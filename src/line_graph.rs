/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

//use super::control_prelude::*;
use super::Canvas;

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
    data: Vec<DataPoint>,
    axis: u8,

    pyramid: Vec<PyramidLevel>,
}

#[derive(Clone)]
pub struct LineGraph {
    canvas: Canvas,
    // the order of DataSeries determines the z-order on screen
}

/*
To draw the screen, the mipmap >= to the number of pixels on screen is chosen and reduced to
a list of min & max values, one for each pixel.

The midmaps for a DataSeries could be calculated on a different thread. When it is done, the midmap
is swapped in, and painting becomes faster.
*/
