// The proc macro is executed for each crate.
// If the dependent crate and your own crate use yew-style-in-rs,
// the following process is executed twice,
// once for the dependent crate and once for your own crate.
//
// After outputting CSS fragments from each of the `style!` crates into files,
// in order to combine the contents of the `css!` declarations in the `style!`
// into a single `style.css` file, the files are finally combined into a single file.
// To make sure that the eight random characters for scope in the `css!` declaration
// in `style!` do not overlap, we use file locking to control exclusivity
// while using the names of the files to check the names that are already there.
// These files survive beyond the proc macro process that is run on a per-crate basis.
//
// The process of writing out the CSS fragments to a file and then combining them
// into a single `style.css` must be performed after the CSS fragments have been written out.
// By using the process that is performed when the `STATE` singleton is destroyed in proc macro,
// we can ensure that it is also executed at the end.
// In Rust, Drop is not called on destruction of static objects,
// so we explicitly specify atexit in libc.
//
// In order to avoid unnecessary styles from being mixed in when dependent crates are removed,
// when style.css is generated at the end, it is compared with the list of dependent crates
// and unneeded CSS fragments are removed.

use anyhow::Result;
use fslock::LockFile;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::iter::repeat_with;
use std::path::PathBuf;
use std::sync::Mutex;

// It is a singleton inherent in the proc macro process.
// The timing of when this STATE is destroyed is monitored and
// the final generation process is executed at the end of the proc macro.
pub static STATE: Lazy<Mutex<State>> = Lazy::new(|| {
    Mutex::new(State::new().expect("Failed to create state of yew-style-in-rs-macro"))
});

pub struct State {
    pub build_path: PathBuf,
    pub lockfile_path: PathBuf,
    pub package_path: PathBuf,
    pub write_flag: bool,
}
impl State {
    // Create `target/release/build-yew-style-in-rs/` directory,
    // and create `target/release/build-yew-style-in-rs/<CRATE NAME>/`,
    // and clean `target/release/build-yew-style-in-rs/<CRATE NAME>/`,
    // and create `target/release/build-yew-style-in-rs/lockfile`.
    fn new() -> Result<Self> {
        // Rust does not execute drop traits for static elements,
        // so we explicitly register a libc atexit.
        extern "C" fn dropper() {
            let mut state = STATE.lock().unwrap();
            state.generate_css();
        }
        unsafe { ::libc::atexit(dropper) };

        let out_dir = crate::util::get_out_dir();

        let build_path = out_dir.join("build-yew-style-in-rs");

        let package_path = build_path.join(env::var("CARGO_PKG_NAME")?);

        if package_path.exists() {
            for entry in fs::read_dir(&package_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    fs::remove_file(path)?;
                }
            }
        }

        let lockfile_path = build_path.join("lockfile");

        Ok(Self {
            build_path,
            lockfile_path,
            package_path,
            write_flag: false,
        })
    }

    // Check `target/release/build-yew-style-in-rs/<CRATE NAME>/<RANDOM 8 CHARACTER>`
    // is exists or not for every exist <CRATE NAME> directories,
    fn exists_id(&self, id: &str) -> bool {
        if !self.build_path.exists() {
            fs::create_dir_all(&self.build_path).unwrap();
        }

        fs::read_dir(&self.build_path)
            .expect("build yew-style-in-rs dir is not exists")
            .into_iter()
            .map(|entry| {
                let entry = entry.unwrap();
                entry.path()
            })
            .filter(|p| p.is_dir())
            .flat_map(|p| {
                fs::read_dir(p)
                    .expect(&format!("some internal dir is not exists"))
                    .into_iter()
                    .map(|entry| {
                        let entry = entry.unwrap();
                        let p = entry.path();
                        p.to_str().unwrap().to_owned()
                    })
            })
            .any(|path| path == id)
    }

    // Create `target/release/build-yew-style-in-rs/<CRATE NAME>/<RANDOM 8 CHARACTER>`
    // for new <RANDOM 8 CHARACTER>.
    pub fn create_random_id_file(&mut self) -> Result<(String, fs::File)> {
        if !self.build_path.exists() {
            fs::create_dir_all(&self.build_path)?;
        }

        let mut lockfile = LockFile::open(&self.lockfile_path)?;
        lockfile.lock()?;

        if !self.package_path.exists() {
            fs::create_dir_all(&self.package_path)?;
        }

        let (id, file) = loop {
            let id = repeat_with(fastrand::alphabetic)
                .take(8)
                .collect::<String>();
            let id_path = self.package_path.join(&id);
            if !self.exists_id(&id) {
                let file = fs::File::create(id_path)?;
                break (id, file);
            }
        };

        lockfile.unlock()?;

        Ok((id, file))
    }

    // Remove CSS fragments related to deleted crate,
    // and remove output style.css and other css,
    // and write CSS fragments into files.
    // CSS fragments first line is filename for output css file.
    fn generate_css(&mut self) {
        // if not write_flag, do nothing.
        if !self.write_flag {
            return;
        }

        // Generate build path is not exists
        if !self.build_path.exists() {
            fs::create_dir_all(&self.build_path).unwrap();
        }

        // Removing CSS files from a deleted package
        let packages = crate::util::get_cargo_packages();
        for entry in fs::read_dir(&self.build_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                if let Some(package_name) = path.file_name() {
                    if let Some(package_name) = package_name.to_str() {
                        if !packages.contains(&package_name.to_owned()) {
                            fs::remove_dir_all(path).unwrap();
                        }
                    }
                }
            }
        }

        // Remove css file in build directory
        let out_dir = crate::util::get_out_dir();
        for entry in fs::read_dir(&out_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "css" {
                        fs::remove_file(path).unwrap();
                    }
                }
            }
        }

        // Write css files
        let mut hashmap = HashMap::new();
        for p in fs::read_dir(&self.build_path)
            .expect("build yew-style-in-rs dir is not exists")
            .into_iter()
            .map(|entry| {
                let entry = entry.unwrap();
                entry.path()
            })
            .filter(|p| p.is_dir())
            .flat_map(|p| {
                fs::read_dir(p)
                    .expect(&format!("some internal dir is not exists"))
                    .into_iter()
                    .map(|entry| {
                        let entry = entry.unwrap();
                        entry.path()
                    })
            })
        {
            let content = fs::read_to_string(p).unwrap();
            let filename = content.lines().next().unwrap().to_string();
            let content = content
                .lines()
                .skip(1)
                .fold("".to_string(), |mut content, item| {
                    content.push_str(item);
                    content
                });
            let entry = hashmap.entry(filename).or_insert(vec![]);
            entry.push(content)
        }
        for (filename, contents) in hashmap {
            let css = contents
                .into_iter()
                .reduce(|content, item| content + &item)
                .unwrap();
            let mut file = fs::File::create(out_dir.join(format!("{filename}.css"))).unwrap();
            file.write(css.as_bytes()).unwrap();
        }
    }
}
