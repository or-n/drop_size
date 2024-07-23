mod hex_color;
mod new_frames;
mod old_frames;
mod paths;

use clap::Parser;
use paths::*;
use std::path::Path;

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
    color: Option<String>,
}

fn make_video_from_new_frames(
    file: &str,
    fps: frames::per_second::FPS,
    fcount: u32,
) {
    let new_frames_dir = new_frames_dir(file);
    if !std::path::Path::new(&new_frames_dir).exists() {
        println!("{new_frames_dir} does not exist");
        return;
    }
    let output_file = new_video_file(file);
    println!("saving video");
    frames::to_video::save(
        &format!("{}/{}", new_frames_dir, file),
        &output_file,
        &format!("{}/{}", fps.numerator, fps.denumerator),
        fcount,
        None,
    )
    .expect("save video");
    println!("{output_file} saved");
}

fn main() {
    let start = std::time::Instant::now();
    let args = Args::parse();
    let input_path = Path::new(&args.input_file);
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
        old_frames::make_directory(&file, fcount);
    }
    if args.make_new_frames {
        if let Some(text) = args.color {
            let color = hex_color::decode(&text).expect("hex color");
            new_frames::make_directory(
                &file,
                fcount,
                color.map(|c| c as f32 / 255.),
            );
        }
    }
    if args.make_video {
        make_video_from_new_frames(&file, fps, fcount);
    }
    println!("elapsed time: {:.2?}", start.elapsed());
}
