use color::hsl::linear::*;
use color::hsl::*;
use color::rgb::*;
use num::operation::length::*;
use num::point::{_2::*, _3::*, _4::*};

pub struct HslDistance {
    pub hue: f32,
    pub sl: f32,
}

#[derive(Clone, Copy)]
pub struct Threshold {
    pub hue: f32,
    pub sl: f32,
    pub rgb: f32,
}

#[inline]
pub fn rgb_distance(color1: _3<f32>, color2: _3<f32>) -> f32 {
    (color1 - color2).length_squared() / 3.
}

#[inline]
pub fn hsl_distance(color1: _3<f32>, color2: _3<f32>) -> HslDistance {
    let c1: HSL<f32> = RGB(color1).into();
    let c2: HSL<f32> = RGB(color2).into();
    let linear1: LinearHSL<f32> = c1.into();
    let linear2: LinearHSL<f32> = c2.into();
    let hue = (linear1.0 .0 - linear2.0 .0).length_squared() / 2.;
    let sl = (linear1.0 .1 - linear2.0 .1).length_squared() / 2.;
    HslDistance { hue, sl }
}

pub fn filter(
    image: &mut super::image::Image,
    filter_color: _3<f32>,
    threshold: Threshold,
) -> Vec<_2<f32>> {
    let mut x = 0;
    let mut y = 0;
    let mut positions = Vec::new();
    for pixel in image.pixels.chunks_exact_mut(4) {
        if let [r, g, b, _a] = pixel {
            let color = _3([*r, *g, *b]);
            let rgb = rgb_distance(filter_color, color);
            let hsl = hsl_distance(filter_color, color);
            if hsl.hue > threshold.hue
                || hsl.sl > threshold.sl
                || rgb > threshold.rgb
            {
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

pub fn delta_blend(up: _4<f32>, down: _4<f32>) -> _4<f32> {
    _4([
        (up[0] - down[0]).abs(),
        (up[1] - down[1]).abs(),
        (up[2] - down[2]).abs(),
        down[3],
    ])
}
