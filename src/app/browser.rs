use eframe::egui::*;
use egui_extras::*;
use ex::Ex;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub enum Event {
    NewFile(String, PathBuf),
    Rename(String, PathBuf),
    Delete(PathBuf),
}

pub enum Buffer {
    Copy(PathBuf),
    Cut(PathBuf),
}

pub struct Browser {
    pub ex: Ex,
    pub search: String,
    event: Option<Event>,
    buffer: Option<Buffer>,
    popup: bool,
    new_tab: Option<PathBuf>,
    new_dir: Option<PathBuf>,
}

impl Browser {
    pub fn new() -> Self {
        Self {
            search: String::new(),
            event: None,
            buffer: None,
            popup: false,
            ex: Ex::new(),
            new_tab: None,
            new_dir: None,
        }
    }
    pub fn set_path(mut self, path: &Path) -> Self {
        self.ex.set_directory(path, &self.search);
        self
    }
    pub fn previous(&mut self) {
        self.ex.previous();
    }
    pub fn next(&mut self) {
        //TODO: keep history of paths visited
        // self.ex.next();
    }
    pub fn title(&self) -> String {
        let file = self.ex.current_file();
        if file.contains(':') {
            let file = file.as_str().replace(":\\", ":");
            format!("Drive ({file})")
        } else {
            file
        }
    }
    pub fn ui(&mut self, ctx: &Context) -> Option<PathBuf> {
        if self.popup {
            Window::new("Delete?")
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            if let Some(Event::Delete(path)) = &self.event {
                                eprintln!("Deleting path: {path:?}");
                                ex::delete(path).unwrap();
                                let path = self.ex.current_path().to_path_buf();
                                self.ex.set_directory(&path, &self.search);
                            };
                            self.popup = false;
                        };

                        if ui.button("No").clicked() {
                            self.popup = false;
                        };
                    });
                });
        }

        let cd = self.ex.current_path_string();

        let response = CentralPanel::default()
            .show(ctx, |ui| {
                self.header(ui, &cd);
                self.center(ui);
            })
            .response;

        response.context_menu(|ui| {
            if ui.button("New File").clicked() {
                //TODO: new file
                self.event = Some(Event::NewFile(String::new(), PathBuf::default()));
                ui.close_menu();
            };

            if ui.button("Open in Terminal").clicked() {
                Command::new("wt.exe").args(["-d", &cd]).output().unwrap();
                ui.close_menu();
            };

            if ui.button("Open in VSCode").clicked() {
                Command::new("cmd")
                    .args(["/c", "code", &cd])
                    .output()
                    .unwrap();
                ui.close_menu();
            };
        });

        if let Some(dir) = self.new_dir.take() {
            self.ex.set_directory(&dir, &self.search);
        }

        self.new_tab.take()
    }
    //TODO: only show the first 5 paths
    fn header(&mut self, ui: &mut Ui, cd: &str) {
        ui.horizontal(|ui| {
            //Add the frame back just for these buttons
            ui.style_mut().visuals.button_frame = true;

            let splits: Vec<&str> = cd.split('\\').filter(|str| !str.is_empty()).collect();

            for (i, s) in splits.iter().enumerate() {
                let label = if s.contains(':') {
                    //TODO: drive name
                    format!("Drive ({s})")
                } else {
                    s.to_string()
                };

                let button = ui.button(label);

                let selection = &splits[..i + 1];
                //join doesn't work if there is only one item
                let path = if selection.len() == 1 {
                    format!("{}\\", selection.join(" "))
                } else {
                    selection.join("\\")
                };

                let path = Path::new(&path);

                if button.clicked() {
                    self.ex.set_directory(path, &self.search);
                }

                if button.middle_clicked() {
                    self.new_tab = Some(path.to_path_buf());
                }
            }
        });
    }
    fn center(&mut self, ui: &mut Ui) {
        let files = &self.ex.files;

        if files.is_empty() {
            if !self.search.is_empty() {
                ui.centered_and_justified(|ui| ui.label("No results found."));
            } else {
                ui.centered_and_justified(|ui| ui.label("Folder is empty."));
            }
            return;
        }

        ui.style_mut().spacing.button_padding = Vec2::new(0.0, 0.5);

        TableBuilder::new(ui)
            .striped(true)
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("Name");
                });
                header.col(|ui| {
                    ui.heading("Date modified");
                });
                header.col(|ui| {
                    ui.heading("Type");
                });
                header.col(|ui| {
                    ui.heading("Size");
                });
            })
            .body(|body| {
                #[allow(unused)]
                body.rows(20.0, files.len(), |i, mut row| {
                    let file = files[i].clone();
                    if let Some(name) = file.file_name() {
                        let name = name.to_string_lossy().to_string();

                        row.col(|ui| {
                            //is_file() can fail on fails that have bad permissions.
                            let button = if file.is_file() {
                                ui.add(Button::new(format!("🖹  {name}")).wrap(false))
                            } else {
                                ui.add(Button::new(format!("🗀  {name}")).wrap(false))
                            };

                            if button.clicked() && file.is_dir() {
                                self.new_dir = Some(file.clone());
                            }

                            if button.double_clicked() && !file.is_dir() {
                                if let Err(e) = ex::open(&file) {
                                    //TODO: print to error bar like Onivim
                                    dbg!(e);
                                }
                            }

                            if button.middle_clicked() && file.is_dir() {
                                //TODO: don't focus this new tab
                                self.new_tab = Some(file.to_path_buf());
                            }

                            button.context_menu(|ui| {
                                if ui.button("Copy").clicked() {
                                    self.buffer = Some(Buffer::Copy(file.to_path_buf()));
                                    ui.close_menu();
                                };

                                if ui.button("Cut").clicked() {
                                    self.buffer = Some(Buffer::Cut(file.to_path_buf()));
                                    ui.close_menu();
                                };

                                ui.separator();

                                if self.buffer.is_some() {
                                    if ui.button("Paste").clicked() {
                                        match &self.buffer {
                                            Some(Buffer::Copy(from)) => {
                                                ex::copy(from, self.ex.current_path())
                                            }
                                            Some(Buffer::Cut(from)) => {
                                                ex::cut(from, self.ex.current_path())
                                            }
                                            None => (),
                                        }
                                        self.new_dir = Some(self.ex.current_path().to_path_buf());
                                        ui.close_menu();
                                    };
                                    ui.separator();
                                }

                                if ui.button("Rename").clicked() {
                                    self.event =
                                        Some(Event::Rename(name.clone(), file.to_path_buf()));
                                    ui.close_menu();
                                };

                                ui.separator();

                                if ui.button("Delete").clicked() {
                                    self.popup = true;
                                    self.event = Some(Event::Delete(file.to_path_buf()));

                                    ui.close_menu();
                                };
                            });
                        });
                    }

                    row.col(|ui| {
                        if let Some(date) = ex::last_modified(&file) {
                            ui.add(Button::new(date).wrap(false));
                        }
                    });

                    row.col(|ui| {
                        if file.is_dir() {
                            ui.add(Button::new("File folder").wrap(false));
                        } else if let Some(ex) = file.extension() {
                            let ex = ex.to_string_lossy().to_string();
                            let unknown = format!(".{ex} file");
                            let file_type = match ex.as_str() {
                                "lnk" => "Shortcut",
                                "zip" => "zip Archive",
                                "exe" => "Application",
                                _ => &unknown,
                            };
                            ui.add(Button::new(file_type).wrap(false));
                        } else if let Some(file_name) = file.file_name() {
                            let file_name = file_name.to_string_lossy().to_string();
                            if file_name.starts_with('.') {
                                let file_type = match file_name.as_str() {
                                    ".gitignore" => "Git Ignore",
                                    ".gitconfig" => "Git Config",
                                    _ => "Unknown dot file",
                                };
                                ui.add(Button::new(file_type).wrap(false));
                            }
                        }
                    });

                    row.col(|ui| {
                        if let Some(size) = ex::file_size(&file) {
                            ui.add(Button::new(size).wrap(false));
                        }
                    });
                });
            });
    }
}
