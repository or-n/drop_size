pub fn old_frames_dir(file: &str) -> String {
    format!("results/{file}/frames/old")
}

pub fn new_frames_dir(file: &str) -> String {
    format!("results/{file}/frames/new")
}

pub fn new_video_file(file: &str) -> String {
    format!("results/{file}/{file}")
}
