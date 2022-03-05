use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::iter::repeat_with;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::Result;
use fslock::LockFile;
use once_cell::sync::Lazy;
use parcel_css::stylesheet::PrinterOptions;
use parcel_css::stylesheet::{ParserOptions, StyleSheet};
use parcel_css::targets::Browsers;

pub static STATE: Lazy<Mutex<State>> = Lazy::new(|| {
    Mutex::new(State::new().expect("Failed to create state of yew-style-in-rs-macro"))
});

pub struct State {
    pub build_path: PathBuf,
    pub lockfile_path: PathBuf,
    pub package_path: PathBuf,
    lockfile: LockFile,
}
impl State {
    fn new() -> Result<Self> {
        extern "C" fn dropper() {
            let mut state = STATE.lock().unwrap();
            state.generate_css();
        }
        unsafe { ::libc::atexit(dropper) };

        let out_dir = crate::util::get_out_dir();

        let build_path = out_dir.join("build-yew-style-in-rs");
        if !build_path.exists() {
            fs::create_dir_all(&build_path)?;
        }

        let package_path = build_path.join(env::var("CARGO_PKG_NAME")?);
        if !package_path.exists() {
            fs::create_dir_all(&package_path)?;
        }

        for entry in fs::read_dir(&package_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                fs::remove_file(path)?;
            }
        }

        let lockfile_path = build_path.join("lockfile");
        let lockfile = LockFile::open(&lockfile_path)?;

        Ok(Self {
            build_path,
            lockfile_path,
            lockfile,
            package_path,
        })
    }

    fn exists_id(&self, id: &str) -> bool {
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

    pub fn create_random_id_file(&mut self) -> Result<(String, fs::File)> {
        self.lockfile.lock()?;

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

        self.lockfile.unlock()?;

        Ok((id, file))
    }

    fn generate_css(&mut self) {
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
            let parser_options = ParserOptions {
                nesting: true,
                custom_media: false,
                css_modules: false,
                source_index: 0,
            };
            let printer_options = PrinterOptions {
                minify: crate::util::is_release(),
                source_map: None,
                targets: Some(Browsers::default()),
                analyze_dependencies: false,
                pseudo_classes: None,
            };
            let code = contents
                .into_iter()
                .reduce(|content, item| content + "\n" + &item)
                .unwrap();
            let stylesheet = StyleSheet::parse(filename.to_owned(), &code, parser_options).unwrap();
            let css = stylesheet.to_css(printer_options).unwrap().code;
            let mut file = fs::File::create(out_dir.join(format!("{filename}.css"))).unwrap();
            file.write(css.as_bytes()).unwrap();
        }
    }
}
