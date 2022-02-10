use eframe::egui::*;
use eframe::epi;
use jwalk::WalkDir;
use std::env;
use std::io;
use std::os::windows::prelude::MetadataExt;
use std::path::Path;
use std::path::PathBuf;

pub struct App {
    files: Vec<PathBuf>,
}

impl App {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn set_directory(&mut self, path: &Path) -> io::Result<()> {
        env::set_current_dir(path)?;
        self.updates_files()?;
        Ok(())
    }

    pub fn updates_files(&mut self) -> io::Result<()> {
        let dir = env::current_dir()?;
        let files: Vec<_> = WalkDir::new(&dir)
            .max_depth(1)
            .into_iter()
            .flat_map(|entry| {
                if let Ok(entry) = entry {
                    Some(entry.path())
                    // let path = entry.path();
                    // if path != dir {
                    //     Some(path)
                    // } else {
                    //     None
                    // }
                } else {
                    None
                }
            })
            .collect();

        self.files = files;
        self.sort();

        Ok(())
    }
    pub fn previous_dir(&mut self) -> io::Result<()> {
        if let Some(dir) = env::current_dir()?.parent() {
            self.set_directory(dir)?
        }
        Ok(())
    }
    pub fn sort(&mut self) {
        //hidden files are sorted normally

        //Sort files into:
        //dot files
        //directorys
        //files
        //sort each category alphabetically
        self.files.sort_by_key(|a| !a.is_dir())
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "Explorer"
    }

    fn setup(
        &mut self,
        _ctx: &CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        self.updates_files().unwrap();
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
            let row_height = ui.fonts()[TextStyle::Body].row_height();
            let files = self.files.clone();
            let num_rows = files.len();

            ScrollArea::auto_sized().show_rows(ui, row_height, num_rows, |ui, row_range| {
                for row in row_range {
                    let file = files.get(row).unwrap();
                    if let Some(name) = file.file_name() {
                        let name = name.to_string_lossy();

                        ui.columns(2, |columns| {
                            if row == 0 {
                                if columns[0].button(format!("../{name}")).clicked() {
                                    self.previous_dir().unwrap();
                                }
                            } else if columns[0].button(name).clicked() && file.is_dir() {
                                self.set_directory(file.as_path()).unwrap();
                            }

                            if let Ok(metadata) = file.metadata() {
                                let size = metadata.file_size();
                                let size_str = if size / 1000 < 1 {
                                    format!("{} bytes", size)
                                } else if size / 10000 < 1 {
                                    format!("{} kilobytes", size / 1000)
                                } else {
                                    format!("{} megabytes", size / 10000)
                                };
                                columns[1].label(&size_str);
                            }
                        });
                    }
                }
            });
        });
    }
}
