use color::hsl::linear::*;
use color::hsl::*;
use color::rgb::*;
use itertools::iproduct;
use num::operation::length::*;
use num::point::{_2::*, _3::*};

#[inline]
pub fn hue_distance(color1: _3<f32>, color2: _3<f32>) -> f32 {
    let c1: HSL<f32> = RGB(color1).into();
    let c2: HSL<f32> = RGB(color2).into();
    let linear1: LinearHSL<f32> = c1.into();
    let linear2: LinearHSL<f32> = c2.into();
    let hue1 = linear1.0 .0;
    let hue2 = linear2.0 .0;
    (hue1 - hue2).length_squared() / 2.
}

pub fn hue_filter(
    image: &mut super::image::Image,
    color: _3<f32>,
    threshold: f32,
) -> Vec<_2<f32>> {
    iproduct!(
        0..image.dimensions.height as i32,
        0..image.dimensions.width as i32
    )
    .flat_map(|(y, x)| {
        let point = _2([x, y]);
        if let Some(pixel_color) = image.get_pixel(point) {
            if hue_distance(color, pixel_color) > threshold {
                image.set_pixel(point, _3([0., 0., 0.]))
            } else {
                return Some(_2(point.0.map(|c| c as f32)));
            }
        }
        None
    })
    .collect()
}
