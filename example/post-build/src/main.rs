use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Get output directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir = Path::new(&manifest_dir);
    let target_dir = manifest_dir.parent().unwrap().join("target");
    let profile = env::var("TRUNK_PROFILE").unwrap();
    let output_dir = Path::new(&target_dir)
        .join("wasm32-unknown-unknown")
        .join(profile);

    // Get staging directory
    let staging_dir = env::var("TRUNK_STAGING_DIR").unwrap();
    let staging_dir = Path::new(&staging_dir);

    // Copy *.css from the output directory to the staging directory
    for entry in fs::read_dir(output_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let ext = path.extension();
        if let Some(ext) = ext {
            if ext == "css" {
                let dist_path = staging_dir.join(path.file_name().unwrap());
                fs::copy(path, dist_path).unwrap();
            }
        }
    }
}
