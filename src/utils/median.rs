use num::point::_2::*;

pub fn median(points: &mut Vec<_2<f32>>) -> Option<_2<f32>> {
    if points.len() == 0 {
        return None;
    }
    use std::cmp::Ordering::Equal;
    let median = points.len() / 2;
    points.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap_or(Equal));
    let median_axis0 = points[median][0];
    points.sort_by(|a, b| a[1].partial_cmp(&b[1]).unwrap_or(Equal));
    let median_axis1 = points[median][1];
    Some(_2([median_axis0, median_axis1]))
}
