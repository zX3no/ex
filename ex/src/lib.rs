use jwalk::WalkDir;
use std::{
    env, fs,
    io::Result,
    os::windows::prelude::MetadataExt,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct Ex {}

impl Ex {
    pub fn set_directory(path: &Path) -> Result<()> {
        env::set_current_dir(path)
    }

    pub fn previous_dir() -> Result<()> {
        if let Some(dir) = env::current_dir()?.parent() {
            Ex::set_directory(dir)?;
        }
        Ok(())
    }

    pub fn get_files() -> Result<Vec<PathBuf>> {
        let dir = env::current_dir()?;
        let mut files: Vec<_> = WalkDir::new(&dir)
            .max_depth(1)
            .skip_hidden(false)
            .into_iter()
            .flat_map(|entry| {
                if let Ok(entry) = entry {
                    Some(entry.path())
                } else {
                    None
                }
            })
            .collect();

        files.sort_by_key(|a| !a.is_dir());

        Ok(files)
    }

    pub fn get_drives() -> Vec<PathBuf> {
        vec![PathBuf::from("C:/"), PathBuf::from("D:/")]
    }

    pub fn file_size(path: &Path) -> Option<String> {
        if let Ok(metadata) = path.metadata() {
            let size = metadata.file_size();
            #[allow(clippy::if_same_then_else)]
            let size_str = if size < 1_000 {
                if size == 0 {
                    String::from("0 KB")
                } else {
                    String::from("1 KB")
                }
                // String::from("< 1 KB")
                // format!("{} B", size)
            } else if size < 1_000_000 {
                format!("{} KB", size / 1_000)
            } else {
                format!("{} KB", size / 1_000)
                // format!("{:.2} megabytes", size as f64 / 1_000_000.0)
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

    pub fn open(path: &Path) -> Result<()> {
        open::that(path)
    }

    pub fn rename(new_name: &str, file: &Path) -> Result<()> {
        let mut new_path = file.to_path_buf();
        new_path.set_file_name(new_name);

        fs::rename(file, new_path)
    }

    pub fn delete(file: &Path) -> std::result::Result<(), trash::Error> {
        trash::delete(file)
    }

    pub fn create_file(path: &Path) -> Result<()> {
        fs::File::create(path)?;
        Ok(())
    }

    pub fn create_dir(path: &Path) -> Result<()> {
        fs::create_dir(path)
    }
}
