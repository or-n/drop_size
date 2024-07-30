use num::operation::length::*;
use num::point::_2::*;
use num::scale::*;
use std::cmp::Ordering::Equal;

type T = f32;

#[inline]
fn polar_angle(a: &_2<T>, b: &_2<T>) -> T {
    (a[1] - b[1]).atan2(a[0] - b[0])
}

#[inline]
fn axis2_product(a: &_2<T>, b: &_2<T>, c: &_2<T>) -> T {
    (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])
}

pub fn convex_hull(points: &Vec<_2<T>>) -> Vec<_2<T>> {
    if points.len() < 3 {
        return points.clone();
    }
    let min_y_point = points
        .iter()
        .min_by(|a, b| match (-a[1]).partial_cmp(&-b[1]).unwrap_or(Equal) {
            Equal => a[0].partial_cmp(&b[0]).unwrap_or(Equal),
            ordering => ordering,
        })
        .unwrap();
    let mut sorted_points = points.clone();
    sorted_points.sort_by(|a, b| {
        polar_angle(a, min_y_point)
            .partial_cmp(&polar_angle(b, min_y_point))
            .unwrap_or(Equal)
    });
    let mut hull = vec![];
    for &point in sorted_points.iter() {
        while hull.len() >= 2
            && axis2_product(
                &hull[hull.len() - 2],
                &hull[hull.len() - 1],
                &point,
            ) < 0.
        {
            hull.pop();
        }
        hull.push(point);
    }
    hull
}

pub fn insert_intermediate_points(
    hull: &Vec<_2<T>>,
    max_distance: T,
) -> Vec<_2<T>> {
    let mut new_points = Vec::new();
    for i in 0..hull.len() {
        let p1 = hull[i];
        let p2 = hull[(i + 1) % hull.len()];
        new_points.push(p1);
        let distance = (p1 - p2).length();
        if distance > max_distance {
            let num_points = (distance / max_distance).ceil() as usize;
            for j in 1..num_points {
                let t = j as f64 / num_points as f64;
                new_points.push(p1 + (p2 - p1).scale(t));
            }
        }
    }
    new_points
}
