use color::hsl::linear::*;
use color::hsl::*;
use color::rgb::*;
use itertools::iproduct;
use num::operation::length::*;
use num::point::{_2::*, _3::*};

pub struct Distance {
    hue: f32,
    sl: f32,
    rgb: f32,
}

#[derive(Clone, Copy)]
pub struct Threshold {
    pub hue: f32,
    pub sl: f32,
    pub rgb: f32,
}

#[inline]
pub fn distance(color1: _3<f32>, color2: _3<f32>) -> Distance {
    let c1: HSL<f32> = RGB(color1).into();
    let c2: HSL<f32> = RGB(color2).into();
    let linear1: LinearHSL<f32> = c1.into();
    let linear2: LinearHSL<f32> = c2.into();
    let hue = (linear1.0 .0 - linear2.0 .0).length_squared() / 2.;
    let sl = (linear1.0 .1 - linear2.0 .1).length_squared() / 2.;
    let rgb = (color1 - color2).length_squared() / 3.;
    Distance { hue, sl, rgb }
}

pub fn hue_filter(
    image: &mut super::image::Image,
    color: _3<f32>,
    threshold: Threshold,
) -> Vec<_2<f32>> {
    iproduct!(
        0..image.dimensions.height as i32,
        0..image.dimensions.width as i32
    )
    .flat_map(|(y, x)| {
        let point = _2([x, y]);
        if let Some(pixel_color) = image.get_pixel(point) {
            let distance = distance(color, pixel_color);
            if distance.hue > threshold.hue
                || distance.sl > threshold.sl
                || distance.rgb > threshold.rgb
            {
                image.set_pixel(point, _3([0., 0., 0.]))
            } else {
                return Some(_2(point.0.map(|c| c as f32)));
            }
        }
        None
    })
    .collect()
}
