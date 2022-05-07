use jwalk::WalkDir;
use std::{
    env, fs,
    io::Result,
    os::windows::prelude::MetadataExt,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct Ex {
    files: Vec<PathBuf>,
}

impl Ex {
    pub fn set_directory(&mut self, path: &Path) {
        if env::set_current_dir(path).is_ok() {
            let mut files: Vec<PathBuf> = WalkDir::new(&path)
                .max_depth(1)
                .skip_hidden(false)
                .into_iter()
                .flatten()
                .map(|dir| dir.path())
                .collect();

            files.sort_by_key(|a| !a.is_dir());
            self.files = files;
        };
    }

    pub fn previous_dir(&mut self) {
        if let Some(path) = self.files.first().cloned() {
            if let Some(parent) = path.parent() {
                self.set_directory(parent);
            }
        }
    }

    pub fn get_files(&self) -> &[PathBuf] {
        &self.files
    }

    //TODO: too slow on windows
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

            if path.is_dir() {
                Some(String::from(""))
            } else {
                Some(size_str)
            }
        } else {
            None
        }
    }

    pub fn last_modified(path: &Path) -> Option<String> {
        if let Ok(metadata) = path.metadata() {
            let last_write_time = metadata.last_write_time();
        }
        None
    }

    pub fn open(&self, path: &Path) -> Result<()> {
        open::that(path)
    }

    pub fn rename(&self, new_name: &str, file: &Path) -> Result<()> {
        let mut new_path = file.to_path_buf();
        new_path.set_file_name(new_name);

        fs::rename(file, new_path)
    }

    pub fn delete(&self, file: &Path) -> std::result::Result<(), trash::Error> {
        trash::delete(file)
    }

    pub fn create_file(&self, path: &Path) -> Result<()> {
        fs::File::create(path)?;
        Ok(())
    }

    pub fn create_dir(&self, path: &Path) -> Result<()> {
        fs::create_dir(path)
    }

    pub fn reset(&mut self) {
        self.files = Vec::new();
    }
}
