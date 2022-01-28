use crate::sector_reader::SectorReader;
use ntfs::{indexes::*, structured_values::*, *};
use std::{fs::File, io::BufReader};

pub struct Helper<'n> {
    fs: BufReader<SectorReader<File>>,
    ntfs: &'n Ntfs,
    current_dir: Vec<NtfsFile<'n>>,
    dir_string: String,
}
impl<'n> Helper<'n> {
    pub fn new(ntfs: &'n mut Ntfs, fs: BufReader<SectorReader<File>>) -> Self {
        Self {
            fs,
            ntfs,
            current_dir: Vec::new(),
            dir_string: String::new(),
        }
    }
    pub fn ls(&mut self) {
        let index = self
            .current_dir
            .last()
            .unwrap()
            .directory_index(&mut self.fs)
            .unwrap();
        let mut iter = index.entries();

        while let Some(entry) = iter.next(&mut self.fs) {
            let entry = entry.unwrap();
            let file_name = entry
                .key()
                .expect("key must exist for a found Index Entry")
                .unwrap();

            let prefix = if file_name.is_directory() {
                "<DIR>"
            } else {
                ""
            };

            println!("{:5}  {}", prefix, file_name.name());
        }
    }
    pub fn cd(&mut self, name: &str) {
        let fs = &mut self.fs;

        if self.current_dir.is_empty() {
            // let file = self.ntfs.root_directory(fs).unwrap();
            // self.current_dir.push(file);
        }

        if name == ".." {
            if self.dir_string.is_empty() {
                return;
            }

            self.current_dir.pop();

            let new_len = self.dir_string.rfind('\\').unwrap_or(0);
            self.dir_string.truncate(new_len);
        } else {
            let index = self
                .current_dir
                .last()
                .unwrap()
                .directory_index(fs)
                .unwrap();

            let mut finder = index.finder();
            let maybe_entry = NtfsFileNameIndex::find(&mut finder, &self.ntfs, fs, name);

            if maybe_entry.is_none() {
                println!("Cannot find subdirectory \"{}\".", name);
                return;
            }

            let entry = maybe_entry.unwrap().unwrap();
            let file_name = entry
                .key()
                .expect("key must exist for a found Index Entry")
                .unwrap();

            if !file_name.is_directory() {
                println!("\"{}\" is not a directory.", name);
                return;
            }

            let file = entry.to_file(&self.ntfs, fs).unwrap();
            let file_name = self
                .best_file_name(&file, self.current_dir.last().unwrap().file_record_number())
                .unwrap();

            if !self.dir_string.is_empty() {
                self.dir_string += "\\";
            }
            self.dir_string += &file_name.name().to_string_lossy();

            self.current_dir.push(file);
        }
    }

    #[allow(dead_code)]
    fn best_file_name(
        &mut self,
        file: &NtfsFile,
        parent_record_number: u64,
    ) -> Result<NtfsFileName> {
        // Try to find a long filename (Win32) first.
        // If we don't find one, the file may only have a single short name (Win32AndDos).
        // If we don't find one either, go with any namespace. It may still be a Dos or Posix name then.
        let priority = [
            Some(NtfsFileNamespace::Win32),
            Some(NtfsFileNamespace::Win32AndDos),
            None,
        ];

        for match_namespace in priority {
            if let Some(file_name) =
                file.name(&mut self.fs, match_namespace, Some(parent_record_number))
            {
                let file_name = file_name?;
                return Ok(file_name);
            }
        }

        panic!(
            "Found no FileName attribute for File Record {:#x}",
            file.file_record_number()
        )
    }
}
