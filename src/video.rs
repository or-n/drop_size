use crate::paths::*;

pub fn make_from_new_frames(
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
