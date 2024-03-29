use chrono::prelude::*;
use jwalk::WalkDir;
use std::{
    env, fs,
    io::{self},
    os::windows::prelude::MetadataExt,
    path::{Path, PathBuf},
};

pub struct Ex {
    pub files: Vec<PathBuf>,
    current: PathBuf,
}

impl Ex {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut s = Self {
            files: Vec::new(),
            current: PathBuf::default(),
        };
        s.set_directory(Path::new("C:\\"), "");
        s
    }

    pub fn previous(&mut self) {
        let path = if let Some(parent) = self.current.parent() {
            parent.to_path_buf()
        } else {
            return;
        };
        self.set_directory(&path, "");
    }

    pub fn current_path(&self) -> &Path {
        &self.current
    }

    pub fn current_path_string(&self) -> String {
        self.current.to_string_lossy().to_string()
    }

    pub fn current_file(&self) -> String {
        self.current
            .file_name()
            .unwrap_or(self.current.as_os_str())
            .to_string_lossy()
            .to_string()
    }

    pub fn set_directory(&mut self, path: &Path, search: &str) {
        if env::set_current_dir(path).is_ok() {
            let mut files: Vec<_> = WalkDir::new(path)
                .max_depth(1)
                .skip_hidden(false)
                .into_iter()
                .flatten()
                //Hide ntfs related files
                .filter(|dir| dir.depth == 1 && dir.metadata().is_ok())
                .map(|dir| dir.path())
                .filter(|file| {
                    if search.is_empty() {
                        true
                    } else {
                        let file_name = file
                            .file_name()
                            .unwrap_or(file.as_os_str())
                            .to_string_lossy()
                            .to_ascii_lowercase();
                        file_name.contains(search)
                    }
                })
                .collect();

            self.current = path.to_path_buf();

            files.sort_by_key(|a| {
                !a.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .starts_with('.')
            });

            files.sort_by_key(|a| !a.is_dir());

            self.files = files;
        };
    }

    pub fn reset(&mut self) {
        self.files = Vec::new();
    }
}

pub fn file_size(path: &Path) -> Option<String> {
    if let Ok(metadata) = path.metadata() {
        let size = metadata.file_size();
        let size_str = if size < 1_000 {
            if size == 0 {
                String::from("0 KB")
            } else {
                String::from("1 KB")
            }
        } else if size < 1_000_000 {
            format!("{} KB", size / 1_000)
        } else {
            format!("{} MB", size / 1_000_000)
        };

        if !path.is_dir() {
            return Some(size_str);
        }
    }
    None
}

pub fn last_modified(path: &Path) -> Option<String> {
    if let Ok(metadata) = path.metadata() {
        let last_write_time = metadata.last_write_time();
        if let Some(date) = windows_date(last_write_time as i64) {
            return Some(date.format("%d/%m/%Y %H:%M").to_string());
        }
    }
    None
}

pub fn open(path: &Path) -> Result<(), String> {
    match open::that(path) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{e:?}")),
    }
}

pub fn rename(new_name: &str, file: &Path) -> io::Result<()> {
    let mut new_path = file.to_path_buf();
    new_path.set_file_name(new_name);

    fs::rename(file, new_path)
}

pub fn delete(file: &Path) -> Result<(), trash::Error> {
    trash::delete(file)
}

pub fn copy(from: &Path, to: &Path) {
    if let Some(name) = from.file_name() {
        let mut to = to.to_path_buf();
        to.push(name);
        eprintln!("Copying file from: {from:?} to: {to:?}");
        fs::copy(from, to).unwrap();
    }
}

pub fn cut(from: &Path, to: &Path) {
    if let Some(name) = from.file_name() {
        let mut to = to.to_path_buf();
        to.push(name);
        eprintln!("Moving file from: {from:?} to: {to:?}");
        fs::rename(from, to).unwrap();
    }
}

pub fn create_file(path: &Path) -> io::Result<()> {
    fs::File::create(path)?;
    Ok(())
}

pub fn create_dir(path: &Path) -> io::Result<()> {
    fs::create_dir(path)
}

// 1601-01-01 is 11,644,473,600 seconds before Unix epoch.
//https://github.com/oylenshpeegul/Epochs-rust/blob/master/src/lib.rs
fn windows_date(x: i64) -> Option<NaiveDateTime> {
    let d = 10_000_000;
    let s = -11_644_473_600;
    let q = x / d;
    let n = ((x % d) * (1_000_000_000 / d)) as u32;
    let t = q.checked_add(s)?;
    NaiveDateTime::from_timestamp_opt(t, n)
}
