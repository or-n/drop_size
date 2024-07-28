use color::hsl::linear::*;
use color::hsl::*;
use color::rgb::*;
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
    let mut x = 0;
    let mut y = 0;
    let mut positions = Vec::new();
    for pixel in image.pixels.chunks_exact_mut(4) {
        if let [r, g, b, _a] = pixel {
            let distance = distance(color, _3([*r, *g, *b]));
            if distance.hue > threshold.hue
                || distance.sl > threshold.sl
                || distance.rgb > threshold.rgb
            {
                *r = 0.0;
                *g = 0.0;
                *b = 0.0;
            } else {
                positions.push(_2([x as f32, y as f32]));
            }
            x += 1;
            if x == image.dimensions.width {
                x = 0;
                y += 1;
            }
        }
    }
    positions
}
