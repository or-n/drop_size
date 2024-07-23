use crate::paths::*;
use num::point::{dot::*, _2::*, _3::*};
use pixels;
use std::io::Write;

fn distance(color1: _3<f32>, color2: _3<f32>) -> f32 {
    let delta = color1 - color2;
    delta.dot(delta) / 3.
}

fn modify_pixels(
    dimensions: &pixels::dimensions::Dimensions,
    pixels: &mut Vec<f32>,
    color: [f32; 3],
) {
    let mut x = 0;
    let mut y = 0;
    let mut positions = Vec::new();
    for pixel in pixels.chunks_exact_mut(4) {
        if let [r, g, b, _a] = pixel {
            if distance(_3(color), _3([*r, *g, *b])) > 0.0125 {
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
    let center = sum.0.map(|c| (c / n) as i32);
    for dy in -4..=4 {
        for dx in -4..=4 {
            let x = center[0] + dx;
            if x < 0 || x >= dimensions.width as i32 {
                continue;
            }
            let y = center[1] + dy;
            if y < 0 || x >= dimensions.height as i32 {
                continue;
            }
            let index = ((y * dimensions.width as i32 + x) * 4) as usize;
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

pub fn make_directory(file: &str, fcount: u32, color: [f32; 3]) {
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
