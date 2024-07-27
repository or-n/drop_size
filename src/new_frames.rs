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

fn new_frame(
    image: &mut Image,
    color: _3<f32>,
    threshold: f32,
    size_overestimate: f32,
) -> Option<f32> {
    let mut positions = color::hue_filter(image, color, threshold);
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
    for point in &dense_hull {
        let point_i32 = _2(point.0.map(|c| c as i32));
        image.set_pixel(point_i32, _3([0., 1., 0.]));
    }
    let center_i32 = _2(center.0.map(|c| c as i32));
    let size = 2;
    for (dy, dx) in iproduct!(-size..=size, -size..=size) {
        image.set_pixel(center_i32 + _2([dx, dy]), _3([1., 1., 1.]));
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
                let (dimensions, pixels) = pixels::read::f32_array(&format!(
                    "{old_frame_file}_{index}.jpg"
                ))
                .expect("read");
                let color = f32::interpolate(
                    f32_ratio(frame - 1, fcount - 1),
                    &[start_color, end_color],
                );
                let mut image = Image { pixels, dimensions };
                if let Some(size) =
                    new_frame(&mut image, color, threshold, size_overestimate)
                {
                    local_results.push((frame, size));
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
    writeln!(writer, "frame,size").expect("header");
    for (i, size) in results.iter() {
        writeln!(writer, "{},{}", i, size).expect("size");
    }
}
