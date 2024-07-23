mod hex_color;
mod new_frames;
mod old_frames;
mod paths;
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
    make_new_frames: bool,

    #[arg(long)]
    make_video: bool,

    #[arg(
        long,
        value_name = "COLOR",
        required_if_eq("make_new_frames", "true"),
        help = "Hex color code (e.g., #FF5733), in quotes"
    )]
    start_color: Option<String>,

    #[arg(
        long,
        value_name = "COLOR",
        required_if_eq("make_new_frames", "true"),
        help = "Hex color code (e.g., #FF5733), in quotes"
    )]
    end_color: Option<String>,
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
        old_frames::make_directory(&args.input_file, &file, fcount);
    }
    if args.make_new_frames {
        if let (Some(text1), Some(text2)) = (args.start_color, args.end_color) {
            let start_color =
                hex_color::decode(&text1).expect("start hex color");
            let end_color = hex_color::decode(&text2).expect("end hex color");
            new_frames::make_directory(
                &file,
                fcount,
                _3(start_color.map(|c| c as f32 / 255.)),
                _3(end_color.map(|c| c as f32 / 255.)),
            );
        }
    }
    if args.make_video {
        video::make_from_new_frames(&file, fps, fcount);
    }
    println!("elapsed time: {:.2?}", start.elapsed());
}
