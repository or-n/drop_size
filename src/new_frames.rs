use crate::convex_hull::*;
use crate::paths::*;
use itertools::iproduct;
use num::interpolate::*;
use num::operation::length::*;
use num::point::{dot::*, _2::*, _3::*};
use num::ratio::f32::*;
use num::scale::*;
use pixels;
use pixels::dimensions::Dimensions;
use std::io::Write;

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

fn median(points: &mut Vec<_2<f32>>) -> _2<f32> {
    let median = points.len() / 2;
    points.sort_by(|a, b| {
        let [a0, _a1] = a.0;
        let [b0, _b1] = b.0;
        a0.partial_cmp(&b0).unwrap_or(std::cmp::Ordering::Equal)
    });
    let median_axis0 = points[median].0[0];
    points.sort_by(|a, b| {
        let [_a0, a1] = a.0;
        let [_b0, b1] = b.0;
        a1.partial_cmp(&b1).unwrap_or(std::cmp::Ordering::Equal)
    });
    let median_axis1 = points[median].0[1];
    _2([median_axis0, median_axis1])
}

fn modify_pixels(
    dimensions: &Dimensions,
    pixels: &mut Vec<f32>,
    color: _3<f32>,
    threshold: f32,
    size_overestimate: f32,
) -> Option<f32> {
    let mut x = 0;
    let mut y = 0;
    let mut positions = Vec::new();
    for pixel in pixels.chunks_exact_mut(4) {
        if let [r, g, b, _a] = pixel {
            if distance(color, _3([*r, *g, *b])) > threshold {
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
    if positions.len() == 0 {
        return None;
    }
    let mut f32_positions: Vec<_2<f32>> = positions
        .iter()
        .map(|point| _2(point.0.map(|c| c as f32)))
        .collect();
    let median = median(&mut f32_positions);
    f32_positions.retain(|position| {
        (*position - median).length_squared()
            < size_overestimate * size_overestimate
    });
    let hull = convex_hull(&f32_positions);
    let full = insert_intermediate_points(&hull, 0.1);
    let full_len_inverse = 1.0 / (full.len() as f32);
    let center_f32 = full
        .iter()
        .fold(_2([0., 0.]), |a, b| a + (*b).scale(full_len_inverse));
    let center_i32 = _2(center_f32.0.map(|c| c as i32));
    let mean_size_square = full.iter().fold(0., |a, b| {
        a + (*b - center_f32).length_squared() * full_len_inverse
    });
    let hull: Vec<_2<i32>> = full
        .iter()
        .map(|point| _2(point.0.map(|c| c as i32)))
        .collect();
    for point in &hull {
        if let Some(index) = index(dimensions, *point) {
            pixels[index + 0] = 0.;
            pixels[index + 1] = 1.;
            pixels[index + 2] = 0.;
        }
    }
    const SIZE: i32 = 2;
    for (dy, dx) in iproduct!(-SIZE..=SIZE, -SIZE..=SIZE) {
        if let Some(index) = index(dimensions, center_i32 + _2([dx, dy])) {
            pixels[index + 0] = 1.;
            pixels[index + 1] = 1.;
            pixels[index + 2] = 1.;
        }
    }
    Some(mean_size_square.sqrt())
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
    threshold: f32,
    size_overestimate: f32,
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
    let mut sizes = Vec::new();
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
        if let Some(size) = modify_pixels(
            &dimensions,
            &mut pixels,
            color,
            threshold,
            size_overestimate,
        ) {
            sizes.push(size);
        }
        pixels::write::f32_array(
            dimensions,
            pixels,
            &format!("{new_frame_file}_{index}.png"),
        )
        .expect("dimensions")
        .expect("save");
    }
    clear_line();
    let sizes_file =
        std::fs::File::create(sizes_file(file)).expect("sizes file");
    let mut writer = std::io::BufWriter::new(sizes_file);
    writeln!(writer, "frame,size").expect("header");
    for (i, size) in sizes.iter().enumerate() {
        writeln!(writer, "{},{}", i + 1, size).expect("size");
    }
}
