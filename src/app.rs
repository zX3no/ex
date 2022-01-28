use eframe::egui::*;
use eframe::epi;
use ntfs::indexes::NtfsFileNameIndex;
use ntfs::Ntfs;
use ntfs::NtfsFile;
use sector_reader::SectorReader;
use std::fs::File;
use std::io::BufReader;

mod sector_reader;
mod table;

// fn get_dir(path: &str) -> Vec<FileEntry> {
//     let new = String::from(r"\\.\\C:");
//     let path = if path.is_empty() { &new } else { path };
//     let f = File::open(r"\\.\C:").unwrap();
//     let sr = SectorReader::new(f, 512).unwrap();
//     let mut fs = BufReader::new(sr);
//     let mut ntfs = Ntfs::new(&mut fs).unwrap();
//     ntfs.read_upcase_table(&mut fs).unwrap();
//     let file = ntfs.root_directory(&mut fs).unwrap();

//     let index = file.directory_index(&mut fs).unwrap();
//     let mut iter = index.entries();

//     let mut files = Vec::new();

//     while let Some(entry) = iter.next(&mut fs) {
//         let entry = entry.unwrap();
//         let file_name = entry
//             .key()
//             .expect("key must exist for a found Index Entry")
//             .unwrap();

//         let name = file_name.name().to_string();
//         let dir = file_name.is_directory();
//         files.push(FileEntry::new(name, dir));
//     }
//     files
// }
pub fn cd(name: &str) -> String {
    let f = File::open(r"\\.\C:").unwrap();
    let sr = SectorReader::new(f, 512).unwrap();
    let mut fs = BufReader::new(sr);
    let mut ntfs = Ntfs::new(&mut fs).unwrap();
    ntfs.read_upcase_table(&mut fs).unwrap();

    let root = ntfs.root_directory(&mut fs).unwrap();

    let mut files = vec![root];

    let mut dir_str = String::new();

    for item in name.split('\\') {
        let index = files.last().unwrap().directory_index(&mut fs).unwrap();
        let mut finder = index.finder();
        let maybe_entry = NtfsFileNameIndex::find(&mut finder, &ntfs, &mut fs, item);

        if maybe_entry.is_none() {
            println!("Cannot find subdirectory \"{}\".", name);
            return dir_str;
        }

        let entry = maybe_entry.unwrap().unwrap();
        let file_name = entry
            .key()
            .expect("key must exist for a found Index Entry")
            .unwrap();

        if !file_name.is_directory() {
            println!("\"{}\" is not a directory.", name);
            return dir_str;
        }

        if !dir_str.is_empty() {
            dir_str += "\\";
        }

        dir_str += &file_name.name().to_string_lossy();

        let file = entry.to_file(&ntfs, &mut fs).unwrap();

        files.push(file);
    }

    dir_str
}

fn get_files(name: &str) -> Vec<FileEntry> {
    let f = File::open(r"\\.\C:").unwrap();
    let sr = SectorReader::new(f, 512).unwrap();
    let mut fs = BufReader::new(sr);
    let mut ntfs = Ntfs::new(&mut fs).unwrap();
    ntfs.read_upcase_table(&mut fs).unwrap();

    let root = ntfs.root_directory(&mut fs).unwrap();

    let mut files = vec![root];

    if !name.is_empty() {
        for item in name.split('\\') {
            let index = files.last().unwrap().directory_index(&mut fs).unwrap();
            let mut finder = index.finder();
            let maybe_entry = NtfsFileNameIndex::find(&mut finder, &ntfs, &mut fs, item);

            if maybe_entry.is_none() {
                panic!();
            }

            let entry = maybe_entry.unwrap().unwrap();
            let file_name = entry
                .key()
                .expect("key must exist for a found Index Entry")
                .unwrap();

            if !file_name.is_directory() {
                println!("\"{}\" is not a directory.", name);
                panic!();
            }

            let file = entry.to_file(&ntfs, &mut fs).unwrap();

            files.push(file);
        }
    }

    let dir = files.last().unwrap();
    let index = dir.directory_index(&mut fs).unwrap();
    let mut iter = index.entries();
    let mut files = Vec::new();

    while let Some(entry) = iter.next(&mut fs) {
        let entry = entry.unwrap();
        let file_name = entry
            .key()
            .expect("key must exist for a found Index Entry")
            .unwrap();

        let name = file_name.name().to_string();
        let dir = file_name.is_directory();
        files.push(FileEntry::new(name, dir));
    }
    files
}

struct FileEntry {
    name: String,
    is_directory: bool,
}

impl FileEntry {
    pub fn new(name: String, is_directory: bool) -> Self {
        Self { name, is_directory }
    }
    pub fn is_dir(&self) -> bool {
        self.is_directory
    }
}

pub struct App {
    selected_row: Option<usize>,
    data: Vec<FileEntry>,
    current_directory: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            selected_row: None,
            data: Vec::new(),
            current_directory: String::new(),
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "Search!"
    }

    fn setup(
        &mut self,
        _ctx: &CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
    }

    fn update(&mut self, ctx: &CtxRef, frame: &mut epi::Frame<'_>) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            warn_if_debug_build(ui);
            let files = get_files(&self.current_directory);
            let row_height = ui.fonts()[TextStyle::Body].row_height();
            let num_rows = files.len();
            ScrollArea::auto_sized().show_rows(ui, row_height, num_rows, |ui, row_range| {
                for row in row_range {
                    let file = files.get(row).unwrap();
                    ui.columns(1, |columns| {
                        if columns[0].button(file.name.clone()).clicked() {
                            self.current_directory = cd(&file.name);
                        };
                    });
                }
            });
        });
    }
}
