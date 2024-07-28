use crate::paths::*;
use crate::utils::{color, convex_hull::*, image::*, median::*};
use itertools::iproduct;
use num::interpolate::*;
use num::operation::length::*;
use num::point::{_2::*, _3::*};
use num::ratio::f32::*;
use num::scale::*;
use pixels;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;

struct Result<T> {
    min: T,
    lower_quartile: T,
    median: T,
    higher_quartile: T,
    max: T,
    mean: T,
    center_x: T,
    center_y: T,
}

fn new_frame(
    image: &mut Image,
    color: _3<f32>,
    threshold: color::Threshold,
    size_overestimate: f32,
) -> Option<Result<f32>> {
    let mut positions = color::filter(image, color, threshold);
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
    let n = dense_hull.len();
    let dense_hull_len_inverse = 1.0 / (n as f32);
    let center = dense_hull
        .iter()
        .fold(_2([0., 0.]), |a, b| a + (*b).scale(dense_hull_len_inverse));
    let mut distances_squared: Vec<f32> = dense_hull
        .iter()
        .map(|point| (*point - center).length_squared())
        .collect();
    distances_squared
        .sort_by(|a, b| a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal));
    let mean = distances_squared.iter().sum::<f32>() * dense_hull_len_inverse;
    for point in &dense_hull {
        let point_i32 = _2(point.0.map(|c| c as i32));
        image.set_pixel(point_i32, _3([0., 1., 0.]));
    }
    let center_i32 = _2(center.0.map(|c| c as i32));
    let size = 2;
    for (dy, dx) in iproduct!(-size..=size, -size..=size) {
        image.set_pixel(center_i32 + _2([dx, dy]), _3([1., 1., 1.]));
    }
    Some(Result {
        min: distances_squared[0].sqrt(),
        lower_quartile: distances_squared[n / 4].sqrt(),
        median: distances_squared[n / 2].sqrt(),
        higher_quartile: distances_squared[(n * 3) / 4].sqrt(),
        max: distances_squared[n - 1].sqrt(),
        mean: mean.sqrt(),
        center_x: center.0[0],
        center_y: center.0[1],
    })
}

pub fn make_directory(
    file: &str,
    fcount: u32,
    start_color: _3<f32>,
    end_color: _3<f32>,
    threshold: color::Threshold,
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
                let (dimensions, pixels) = pixels::read::f32_array(&format!(
                    "{old_frame_file}_{index}.jpg"
                ))
                .expect("read");
                let color = f32::interpolate(
                    f32_ratio(frame - 1, fcount - 1),
                    &[start_color, end_color],
                );
                let mut image = Image { pixels, dimensions };
                if let Some(result) =
                    new_frame(&mut image, color, threshold, size_overestimate)
                {
                    local_results.push((frame, result));
                }
                if make_new_frames {
                    pixels::write::f32_array(
                        image.dimensions,
                        image.pixels,
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
    writeln!(writer, "frame,min,lower_quartile,median,higher_quartile,max,mean,center_x,center_y")
        .expect("header");
    for (frame, result) in results.iter() {
        let fields = [
            result.min,
            result.lower_quartile,
            result.median,
            result.higher_quartile,
            result.max,
            result.mean,
            result.center_x,
            result.center_y,
        ]
        .into_iter()
        .map(|field| field.to_string())
        .collect::<Vec<_>>()
        .join(",");
        writeln!(writer, "{frame},{fields}").expect("size");
    }
}
