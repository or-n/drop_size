use crate::paths::*;

pub fn make_directory(
    full_path: &str,
    file: &str,
    fcount: u32,
    ranges: [range::MinSize<u32>; 2],
) {
    let old_frames_dir = old_frames_dir(file);
    if std::path::Path::new(&old_frames_dir).exists() {
        println!("{old_frames_dir} exists, skipping saving old frames");
        return;
    }
    std::fs::create_dir_all(&old_frames_dir).expect("old frames directory");
    println!("saving old frames");
    frames::from_video::save(
        full_path,
        &format!("{old_frames_dir}/{file}"),
        fcount,
        ranges,
    )
    .expect("old frames");
}
