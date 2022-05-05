use jwalk::WalkDir;
use std::{
    env, fs,
    io::Result,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct Ex {
    current_dir: PathBuf,
    files: Vec<PathBuf>,
}

impl Ex {
    pub fn set_directory(&mut self, path: &Path) {
        self.current_dir = path.to_path_buf();

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

    #[allow(mutable_borrow_reservation_conflict, clippy::unnecessary_to_owned)]
    pub fn previous_dir(&mut self) -> Result<()> {
        if let Some(parent) = self.current_dir.parent() {
            self.set_directory(&parent.to_path_buf());
        }
        Ok(())
    }

    pub fn current_dir(&self) -> &Path {
        &self.current_dir
    }

    pub fn get_files(&self) -> &[PathBuf] {
        &self.files
    }

    //TODO: too slow on windows
    // pub fn file_size(&self, path: &Path) -> Option<String> {
    //     if let Ok(metadata) = path.metadata() {
    //         let size = metadata.file_size();
    //         #[allow(clippy::if_same_then_else)]
    //         let size_str = if size < 1_000 {
    //             if size == 0 {
    //                 String::from("0 KB")
    //             } else {
    //                 String::from("1 KB")
    //             }
    //             // String::from("< 1 KB")
    //             // format!("{} B", size)
    //         } else if size < 1_000_000 {
    //             format!("{} KB", size / 1_000)
    //         } else {
    //             format!("{} KB", size / 1_000)
    //             // format!("{:.2} megabytes", size as f64 / 1_000_000.0)
    //         };

    //         if path.is_dir() {
    //             Some(String::from(""))
    //         } else {
    //             Some(size_str)
    //         }
    //     } else {
    //         None
    //     }
    // }

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
        self.current_dir = PathBuf::default()
    }
}
