use crate::paths::*;
use itertools::iproduct;
use num::interpolate::*;
use num::point::{dot::*, _2::*, _3::*};
use num::ratio::f32::*;
use pixels;
use pixels::dimensions::Dimensions;
use std::collections::HashSet;
use std::io::Write;

const SIZE: i32 = 2;
const THRESHOLD: f32 = 0.04;

#[inline]
fn distance(color1: _3<f32>, color2: _3<f32>) -> f32 {
    let delta = color1 - color2;
    delta.dot(delta) / 3.
}

#[inline]
fn index(dimensions: &Dimensions, point: _2<i32>) -> Option<usize> {
    let [x, y] = point.0;
    if x >= 0
        && x < dimensions.width as i32
        && y >= 0
        && y < dimensions.height as i32
    {
        Some(((y * dimensions.width as i32 + x) * 4) as usize)
    } else {
        None
    }
}

fn dilate(dimensions: &Dimensions, pixels: &mut Vec<f32>, color: _3<f32>) {
    let mut indices = HashSet::new();
    for (y, x) in
        iproduct!(0..dimensions.height as i32, 0..dimensions.width as i32)
    {
        let point = _2([x, y]);
        if let Some(i) = index(dimensions, point) {
            let r = pixels[i + 0];
            let g = pixels[i + 1];
            let b = pixels[i + 2];
            if distance(color, _3([r, g, b])) <= THRESHOLD {
                for (dy, dx) in iproduct!(-SIZE..=SIZE, -SIZE..=SIZE) {
                    let delta = _2([dx, dy]);
                    if let Some(index) = index(dimensions, point + delta) {
                        indices.insert(index);
                    }
                }
            }
        }
    }
    let [r, g, b] = color.0;
    for index in indices {
        pixels[index + 0] = r;
        pixels[index + 1] = g;
        pixels[index + 2] = b;
    }
}

fn modify_pixels(
    dimensions: &Dimensions,
    pixels: &mut Vec<f32>,
    color: _3<f32>,
) {
    dilate(dimensions, pixels, color);
    let mut x = 0;
    let mut y = 0;
    let mut positions = Vec::new();
    for pixel in pixels.chunks_exact_mut(4) {
        if let [r, g, b, _a] = pixel {
            if distance(color, _3([*r, *g, *b])) > THRESHOLD {
                *r = 0.0;
                *g = 0.0;
                *b = 0.0;
            } else {
                positions.push(_2([x, y]));
            }
            x += 1;
            if x == dimensions.width {
                x = 0;
                y += 1;
            }
        }
    }
    let n = positions.len() as u32;
    let sum = positions.into_iter().fold(_2([0, 0]), std::ops::Add::add);
    let center = _2(sum.0.map(|c| (c / n) as i32));
    for (dy, dx) in iproduct!(-SIZE..=SIZE, -SIZE..=SIZE) {
        if let Some(index) = index(dimensions, center + _2([dx, dy])) {
            pixels[index + 0] = 1.;
            pixels[index + 1] = 1.;
            pixels[index + 2] = 1.;
        }
    }
}

fn clear_line() {
    print!("\x1B[2K\r");
    std::io::stdout().flush().expect("flush");
}

pub fn make_directory(
    file: &str,
    fcount: u32,
    start_color: _3<f32>,
    end_color: _3<f32>,
) {
    let old_frames_dir = old_frames_dir(file);
    let new_frames_dir = new_frames_dir(file);
    let old_frame_file = format!("{old_frames_dir}/{file}");
    let new_frame_file = format!("{new_frames_dir}/{file}");
    if !std::path::Path::new(&old_frames_dir).exists() {
        println!("{old_frames_dir} does not exist");
        return;
    }
    std::fs::create_dir_all(&new_frames_dir).expect("new frames directory");
    let index_digits = (fcount.ilog10() + 1) as usize;
    for frame in 1..=fcount {
        print!("\rsaving new frames: {frame} ");
        std::io::stdout().flush().expect("flush");
        let index = format!("{:0width$}", frame, width = index_digits);
        let (dimensions, mut pixels) =
            pixels::read::f32_array(&format!("{old_frame_file}_{index}.png"))
                .expect("read");
        let color = f32::interpolate(
            f32_ratio(frame - 1, fcount - 1),
            &[start_color, end_color],
        );
        modify_pixels(&dimensions, &mut pixels, color);
        pixels::write::f32_array(
            dimensions,
            pixels,
            &format!("{new_frame_file}_{index}.png"),
        )
        .expect("dimensions")
        .expect("save");
    }
    clear_line();
}
