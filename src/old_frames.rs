use crate::paths::*;

pub fn make_directory(file: &str, fcount: u32) {
    let old_frames_dir = old_frames_dir(file);
    if std::path::Path::new(&old_frames_dir).exists() {
        println!("{old_frames_dir} exists, skipping saving old frames");
        return;
    }
    std::fs::create_dir_all(&old_frames_dir).expect("old frames directory");
    println!("saving old frames");
    frames::from_video::save(
        &file,
        &format!("{old_frames_dir}/{file}"),
        fcount,
    )
    .expect("old frames");
}
