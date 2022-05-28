use std::env;
use std::path::Path;
use std::fs;

fn main() {
    // copy *.css from the output directory to the dist directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir = Path::new(&manifest_dir);
    let profile = env::var("PROFILE").unwrap();
    let target_dir = manifest_dir.parent().unwrap().join("target");
    let output_dir = Path::new(&target_dir).join("wasm32-unknown-unknown").join(profile);
    let staging_dir = Path::new(&manifest_dir).join("dist").join(".stage");
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

    // rerun the build script if dist/.stage directory is created
    println!("cargo:rerun-if-changed=dist/.stage/");
}
