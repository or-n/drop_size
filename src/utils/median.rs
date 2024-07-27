use num::point::_2::*;

pub fn median(points: &mut Vec<_2<f32>>) -> _2<f32> {
    use std::cmp::Ordering::Equal;
    let median = points.len() / 2;
    points.sort_by(|a, b| a.0[0].partial_cmp(&b.0[0]).unwrap_or(Equal));
    let median_axis0 = points[median].0[0];
    points.sort_by(|a, b| a.0[1].partial_cmp(&b.0[1]).unwrap_or(Equal));
    let median_axis1 = points[median].0[1];
    _2([median_axis0, median_axis1])
}
