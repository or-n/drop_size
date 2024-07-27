use crate::convex_hull::*;
use crate::paths::*;
use color::hsl::linear::*;
use color::hsl::*;
use color::rgb::*;
use itertools::iproduct;
use num::interpolate::*;
use num::operation::length::*;
use num::point::{_2::*, _3::*};
use num::ratio::f32::*;
use num::scale::*;
use pixels;
use pixels::dimensions::Dimensions;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;

#[inline]
fn distance(color1: _3<f32>, color2: _3<f32>) -> f32 {
    let c1: HSL<f32> = RGB(color1).into();
    let c2: HSL<f32> = RGB(color2).into();
    let linear1: LinearHSL<f32> = c1.into();
    let linear2: LinearHSL<f32> = c2.into();
    let hue1 = linear1.0 .0;
    let hue2 = linear2.0 .0;
    (hue1 - hue2).length_squared() / 2.
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
    use std::cmp::Ordering::Equal;
    let median = points.len() / 2;
    points.sort_by(|a, b| a.0[0].partial_cmp(&b.0[0]).unwrap_or(Equal));
    let median_axis0 = points[median].0[0];
    points.sort_by(|a, b| a.0[1].partial_cmp(&b.0[1]).unwrap_or(Equal));
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
        let point = _2([x as f32, y as f32]);
        if let [r, g, b, _a] = pixel {
            if distance(color, _3([*r, *g, *b])) > threshold {
                *r = 0.0;
                *g = 0.0;
                *b = 0.0;
            } else {
                positions.push(point);
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
    let median = median(&mut positions);
    positions.retain(|position| {
        (*position - median).length_squared()
            < size_overestimate * size_overestimate
    });
    let hull = convex_hull(&positions);
    let dense_hull = insert_intermediate_points(&hull, 0.1);
    let dense_hull_len_inverse = 1.0 / (dense_hull.len() as f32);
    let center = dense_hull
        .iter()
        .fold(_2([0., 0.]), |a, b| a + (*b).scale(dense_hull_len_inverse));
    let mean_size_square = dense_hull.iter().fold(0., |a, b| {
        a + (*b - center).length_squared() * dense_hull_len_inverse
    });
    let mut modify_pixel = |point, [r, g, b]: [f32; 3]| {
        if let Some(index) = index(dimensions, point) {
            pixels[index + 0] = r;
            pixels[index + 1] = g;
            pixels[index + 2] = b;
        }
    };
    for point in &dense_hull {
        let point_i32 = _2(point.0.map(|c| c as i32));
        modify_pixel(point_i32, [0., 1., 0.]);
    }
    let center_i32 = _2(center.0.map(|c| c as i32));
    let size = 2;
    for (dy, dx) in iproduct!(-size..=size, -size..=size) {
        modify_pixel(center_i32 + _2([dx, dy]), [1., 1., 1.]);
    }
    Some(mean_size_square.sqrt())
}

pub fn make_directory(
    file: &str,
    fcount: u32,
    start_color: _3<f32>,
    end_color: _3<f32>,
    threshold: f32,
    size_overestimate: f32,
    threads: u32,
    make_new_frames: bool,
) {
    let old_frames_dir = old_frames_dir(file);
    let new_frames_dir = new_frames_dir(file);
    let old_frame_file = Arc::new(format!("{old_frames_dir}/{file}"));
    let new_frame_file = Arc::new(format!("{new_frames_dir}/{file}"));
    if !std::path::Path::new(&old_frames_dir).exists() {
        println!("{old_frames_dir} does not exist");
        return;
    }
    std::fs::create_dir_all(&new_frames_dir).expect("new frames directory");
    let index_digits = (fcount.ilog10() + 1) as usize;
    let chunk_size = (fcount + threads - 1) / threads;
    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];
    print!("saving sizes");
    if make_new_frames {
        print!(" and new frames");
    }
    println!();
    for i in 0..threads {
        let results = Arc::clone(&results);
        let old_frame_file = Arc::clone(&old_frame_file);
        let new_frame_file = Arc::clone(&new_frame_file);
        let start = i * chunk_size + 1;
        let end = ((i + 1) * chunk_size).min(fcount);
        let handle = thread::spawn(move || {
            let mut local_results = Vec::new();
            for frame in start..=end {
                let index = format!("{:0width$}", frame, width = index_digits);
                let (dimensions, mut pixels) = pixels::read::f32_array(
                    &format!("{old_frame_file}_{index}.jpg"),
                )
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
                    local_results.push((frame, size));
                }
                if make_new_frames {
                    pixels::write::f32_array(
                        dimensions,
                        pixels,
                        &format!("{new_frame_file}_{index}.jpg"),
                    )
                    .expect("dimensions")
                    .expect("save");
                }
            }
            let mut results = results.lock().unwrap();
            results.extend(local_results);
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    let mut results = results.lock().unwrap();
    results.sort_by(|a, b| a.0.cmp(&b.0));
    let sizes_file =
        std::fs::File::create(sizes_file(file)).expect("sizes file");
    let mut writer = std::io::BufWriter::new(sizes_file);
    writeln!(writer, "frame,size").expect("header");
    for (i, size) in results.iter() {
        writeln!(writer, "{},{}", i, size).expect("size");
    }
}
