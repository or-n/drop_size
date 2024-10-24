mod new_frames;
mod old_frames;
mod paths;
mod utils;
mod video;

use clap::Parser;
use num::point::_3::*;

/// Simple program to determine drop size
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_file: String,

    #[arg(long)]
    make_old_frames: bool,

    #[arg(long)]
    make_sizes: bool,

    #[arg(long)]
    make_new_frames: bool,

    #[arg(long)]
    make_video: bool,

    #[arg(
        long,
        value_name = "COLOR",
        required_if_eq("make_sizes", "true"),
        help = "Hex color code (e.g., #00457b), in quotes"
    )]
    start_color: Option<String>,

    #[arg(
        long,
        value_name = "COLOR",
        required_if_eq("make_sizes", "true"),
        help = "Hex color code (e.g., #00457b), in quotes"
    )]
    end_color: Option<String>,

    #[arg(
        long,
        value_name = "number",
        required_if_eq("make_sizes", "true"),
        help = "threshold for frame delta rgb distance to black (e.g. 0.04)"
    )]
    frame_delta_threshold: Option<f32>,

    #[arg(
        long,
        value_name = "number",
        required_if_eq("make_sizes", "true"),
        help = "threshold for hue (e.g. 0.04)"
    )]
    hue_threshold: Option<f32>,

    #[arg(
        long,
        value_name = "number",
        required_if_eq("make_sizes", "true"),
        help = "threshold for saturation and lightness (e.g. 0.04)"
    )]
    sl_threshold: Option<f32>,

    #[arg(
        long,
        value_name = "number",
        required_if_eq("make_sizes", "true"),
        help = "threshold for RGB (e.g. 0.04)"
    )]
    rgb_threshold: Option<f32>,

    #[arg(
        long,
        value_name = "number",
        required_if_eq("make_sizes", "true"),
        help = "size overestimate to detect outliers (e.g. 400)"
    )]
    size_overestimate: Option<f32>,

    #[arg(
        long,
        value_name = "number",
        required_if_eq("make_sizes", "true"),
        help = "threads (e.g. 4)"
    )]
    threads: Option<u32>,

    #[arg(long, value_name = "number", required_if_eq("make_sizes", "true"))]
    min_x: u32,

    #[arg(long, value_name = "number", required_if_eq("make_sizes", "true"))]
    size_x: u32,

    #[arg(long, value_name = "number", required_if_eq("make_sizes", "true"))]
    min_y: u32,

    #[arg(long, value_name = "number", required_if_eq("make_sizes", "true"))]
    size_y: u32,
}

fn main() {
    let start = std::time::Instant::now();
    let args = Args::parse();
    let input_path = std::path::Path::new(&args.input_file);
    if !input_path.exists() {
        println!("{input_path:?} does not exist");
        return;
    }
    let file = input_path
        .file_name()
        .expect("file")
        .to_str()
        .expect("file str");
    let fps = frames::per_second::extract(&args.input_file).expect("fps");
    let fcount = frames::count::extract(&args.input_file).expect("fcount");
    println!("{file}: {fps:?}, {fcount} frames");
    if args.make_old_frames {
        let ranges = [
            range::MinSize {
                min: args.min_x,
                size: args.size_x,
            },
            range::MinSize {
                min: args.min_y,
                size: args.size_y,
            },
        ];
        old_frames::make_directory(&args.input_file, &file, fcount, ranges);
    }
    if args.make_sizes {
        let color = |arg: Option<String>| {
            let text = arg.expect("color arg");
            let u8 = utils::hex_color::decode(&text).expect("color hex");
            _3(u8.map(|c| c as f32 / 255.))
        };
        let thread_data = new_frames::ThreadData {
            file: file.to_string(),
            fcount,
            start_color: color(args.start_color),
            end_color: color(args.end_color),
            frame_delta_threshold: args
                .frame_delta_threshold
                .expect("frame delta threshold"),
            threshold: utils::color::Threshold {
                hue: args.hue_threshold.expect("hue threshold"),
                sl: args.sl_threshold.expect("sl threshold"),
                rgb: args.rgb_threshold.expect("rgb threshold"),
            },
            size_overestimate: args
                .size_overestimate
                .expect("size-overestimate"),
            make_new_frames: args.make_new_frames,
        };
        new_frames::make_directory(thread_data, args.threads.expect("threads"));
    }
    if args.make_video {
        video::make_from_new_frames(&file, fps, fcount);
    }
    println!("elapsed time: {:.2?}", start.elapsed());
}
