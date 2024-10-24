use crate::paths::*;
use crate::utils::{color, convex_hull::*, image::*, median::*};
use arrayref::{array_mut_ref, array_ref};
use itertools::iproduct;
use num::interpolate::*;
use num::operation::length::*;
use num::point::{_2::*, _3::*, _4::*};
use num::ratio::f32::*;
use num::scale::*;
use pixels;
use std::io::Write;

#[derive(Clone)]
pub struct ThreadData {
    pub file: String,
    pub fcount: u32,
    pub start_color: _3<f32>,
    pub end_color: _3<f32>,
    pub frame_delta_threshold: f32,
    pub threshold: color::Threshold,
    pub size_overestimate: f32,
    pub make_new_frames: bool,
}

struct Result<T, const N: usize> {
    samples: [T; N],
    mean: T,
    center_x: T,
    center_y: T,
}

fn black_distance(color: _4<f32>) -> f32 {
    let [r, g, b, _] = *color;
    color::rgb_distance(_3([0., 0., 0.]), _3([r, g, b]))
}

fn frame_delta(
    image: &mut Image,
    image_before: &Image,
    threshold: f32,
) -> Option<()> {
    if image.dimensions != image_before.dimensions {
        return None;
    }
    for i in (0..image.pixels.len()).step_by(4) {
        let pixel: &mut [f32; 4] = array_mut_ref![image.pixels, i, 4];
        let color = _4(*pixel);
        let color_before = _4(*array_ref![image_before.pixels, i, 4]);
        let blend_color = color::delta_blend(color_before, color);
        *pixel = if black_distance(blend_color) < threshold {
            [0., 0., 0., 1.]
        } else {
            *color
        };
    }
    Some(())
}

fn new_frame<const N: usize>(
    image: &mut Image,
    image_before: &Image,
    color: _3<f32>,
    frame_delta_threshold: f32,
    threshold: color::Threshold,
    size_overestimate: f32,
) -> Option<Result<f32, N>> {
    frame_delta(image, image_before, frame_delta_threshold)?;
    let mut positions = color::filter(image, color, threshold);
    let median = median(&mut positions)?;
    positions.retain(|position| {
        (*position - median).length_squared()
            < size_overestimate * size_overestimate
    });
    if positions.len() == 0 {
        return None;
    }
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
        samples: (0..N)
            .map(|i| distances_squared[i * (n - 1) / (N - 1)].sqrt())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
        mean: mean.sqrt(),
        center_x: center[0],
        center_y: center[1],
    })
}

fn thread(range: (u32, u32), data: ThreadData) -> Vec<(u32, Result<f32, 5>)> {
    let index_digits = (data.fcount.ilog10() + 1) as usize;
    let old_frame_file =
        format!("{}/{}", old_frames_dir(&data.file), data.file);
    let new_frame_file =
        format!("{}/{}", new_frames_dir(&data.file), data.file);
    let mut local_results = Vec::new();
    let load = |frame| {
        let index = format!("{:0width$}", frame, width = index_digits);
        let (dimensions, pixels) =
            pixels::read::f32_array(&format!("{old_frame_file}_{index}.jpg"))
                .expect("read");
        (index, Image { pixels, dimensions })
    };
    for frame in range.0.max(2)..=range.1 {
        let before = (frame / 2).max(1);
        let color = f32::interpolate(
            f32_ratio(frame - 1, data.fcount - 1),
            &[data.start_color, data.end_color],
        );
        let (_, image_before) = load(frame - before);
        let (index, mut image) = load(frame);
        if let Some(result) = new_frame::<5>(
            &mut image,
            &image_before,
            color,
            data.frame_delta_threshold,
            data.threshold,
            data.size_overestimate,
        ) {
            local_results.push((frame, result));
        }
        if data.make_new_frames {
            pixels::write::f32_array(
                image.dimensions,
                image.pixels,
                &format!("{new_frame_file}_{index}.jpg"),
            )
            .expect("dimensions")
            .expect("save");
        }
    }
    local_results
}

pub fn make_directory(thread_data: ThreadData, threads: u32) {
    let old_frames_dir = old_frames_dir(&thread_data.file);
    let new_frames_dir = new_frames_dir(&thread_data.file);
    if !std::path::Path::new(&old_frames_dir).exists() {
        println!("{old_frames_dir} does not exist");
        return;
    }
    std::fs::create_dir_all(&new_frames_dir).expect("new frames directory");
    let chunk_size = (thread_data.fcount + threads - 1) / threads;
    let mut results = Vec::new();
    let and = thread_data.make_new_frames;
    println!("saving sizes{}", if and { " and new frames" } else { "" });
    let handles: Vec<_> = (0..threads)
        .map(|i| {
            let range = (
                i * chunk_size + 1,
                ((i + 1) * chunk_size).min(thread_data.fcount),
            );
            let data = thread_data.clone();
            std::thread::spawn(move || thread(range, data))
        })
        .collect();
    for handle in handles {
        results.extend(handle.join().unwrap());
    }
    let sizes_file = std::fs::File::create(sizes_file(&thread_data.file))
        .expect("sizes file");
    let mut writer = std::io::BufWriter::new(sizes_file);
    writeln!(writer, "frame,min,lower_quartile,median,higher_quartile,max,mean,center_x,center_y")
        .expect("header");
    for (frame, result) in results.iter() {
        let fields = [
            result.samples[0],
            result.samples[1],
            result.samples[2],
            result.samples[3],
            result.samples[4],
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
