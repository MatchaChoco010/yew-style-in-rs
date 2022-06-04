use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::env;
use std::path::PathBuf;

// cache workspace directory
static WORKSPACE: OnceCell<PathBuf> = OnceCell::new();

// Get flag that this build is release build or not.
//
// I don't want the profile of the proc macro itself,
// so I check the args starting with `debuginfo` instead of `#[cfg(debug_assertions)]`.
pub fn is_release() -> bool {
    let mut is_release = true;
    let mut args = env::args();
    while let Some(arg) = args.next() {
        if arg.starts_with("debuginfo") {
            is_release = false;
        }
    }
    is_release
}

// Get output directory.
// eg)
// - target/debug/
// - target/release/
// - workspace/target/release/
//
// `OUT_DIR` env can't be used in proc macro.
// https://github.com/rust-lang/cargo/issues/9084
//
// Get the directory from which the last directory was removed from the `--out-dir` argument
// as a workaround.
// if there is no `--out-dir` argument, then use default directory for `workspace/target/{profile}`.
//
// This method create Cargo.lock file if not exists, so this method should not call when dry-run.
pub fn get_out_dir() -> PathBuf {
    let profile = if is_release() { "release" } else { "debug" };

    let mut out_dir = get_cargo_workspace().join("target").join(&profile);

    let mut args = env::args();
    while let Some(arg) = args.next() {
        if arg == "--out-dir" {
            if let Some(dir) = args.next() {
                out_dir = PathBuf::from(dir);
            }
        }
    }

    while !out_dir.ends_with(&profile) {
        if !out_dir.pop() {
            panic!("Failed to find out_dir");
        }
    }

    out_dir
}

// Get workspace directory path.
//
// Currently rust workspace directory information is only in `cargo metadata`.
// So execute `cargo metadata` and parse json to get workspace_root directory.
//
// This method create Cargo.lock file if not exists, so this method should not call when dry-run.
pub fn get_cargo_workspace() -> PathBuf {
    WORKSPACE
        .get_or_init(|| {
            let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
            let output = std::process::Command::new(env::var("CARGO").unwrap())
                .arg("metadata")
                .arg("--format-version=1")
                .current_dir(manifest_dir)
                .output()
                .unwrap();

            #[derive(Deserialize)]
            struct Metadata {
                workspace_root: String,
            }

            let metadata: Metadata = serde_json::from_slice(&output.stdout).unwrap();

            let workspace = PathBuf::from(metadata.workspace_root);
            workspace
        })
        .into()
}

// Get dependencies and itself package names.
//
// `cargo metadata` return dependencies information/
// So execute `cargo metadata` and parse json to get dependencies name.
//
// This method create Cargo.lock file if not exists, so this method should not call when dry-run.
pub fn get_cargo_packages() -> Vec<String> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let output = std::process::Command::new(env::var("CARGO").unwrap())
        .arg("metadata")
        .arg("--format-version=1")
        .current_dir(manifest_dir)
        .output()
        .unwrap();

    #[derive(Deserialize)]
    struct Package {
        name: String,
    }
    #[derive(Deserialize)]
    struct Metadata {
        packages: Vec<Package>,
    }

    let metadata: Metadata = serde_json::from_slice(&output.stdout).unwrap();
    metadata
        .packages
        .into_iter()
        .map(|p| p.name)
        .collect::<Vec<_>>()
}
