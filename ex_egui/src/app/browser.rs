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
}

impl Browser {
    pub fn new() -> Self {
        Self {
            search: String::new(),
            event: None,
            buffer: None,
            popup: false,
            ex: Ex::new(),
        }
    }
    pub fn set_path(mut self, path: &Path) -> Self {
        self.ex.set_directory(path, &self.search);
        self
    }
    pub fn title(&self) -> String {
        let file = self.ex.current_file();
        if file.contains(':') {
            let file = file.as_str().replace(":\\", ":");
            format!("Drive ({})", file)
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
                                eprintln!("Deleting path: {:?}", path);
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

        let (response, event) = self.center(ctx);
        let current_dir_string = self.ex.current_path_string();

        response.context_menu(|ui| {
            if ui.button("New File").clicked() {
                //TODO: new file
                self.event = Some(Event::NewFile(String::new(), PathBuf::default()));
                ui.close_menu();
            };

            if ui.button("Open in Terminal").clicked() {
                Command::new("wt.exe")
                    .args(&["-d", &current_dir_string])
                    .output()
                    .unwrap();
                ui.close_menu();
            };

            if ui.button("Open in VSCode").clicked() {
                Command::new("cmd")
                    .args(&["/c", "code", &current_dir_string])
                    .output()
                    .unwrap();
                ui.close_menu();
            };
        });

        event
    }
    fn center(&mut self, ctx: &Context) -> (Response, Option<PathBuf>) {
        //Check if the user wants a new tab.
        let mut tab = None;
        let mut new_dir = None;

        let response = CentralPanel::default()
            .show(ctx, |ui| {
                //Header
                //TODO: only show the first 5 paths
                let current_dir = self.ex.current_path_string();

                let splits: Vec<&str> = current_dir
                    .split('\\')
                    .filter(|str| !str.is_empty())
                    .collect();

                ui.horizontal(|ui| {
                    //Add the frame back just for these buttons
                    ui.style_mut().visuals.button_frame = true;

                    for (i, s) in splits.iter().enumerate() {
                        let label = if s.contains(':') {
                            //TODO: drive name
                            format!("Drive ({})", s)
                        } else {
                            s.to_string()
                        };
                        if ui.button(label).clicked() {
                            let selection = &splits[..i + 1];

                            //join doesn't work if there is only one item
                            let path = if selection.len() == 1 {
                                format!("{}\\", selection.join(" "))
                            } else {
                                selection.join("\\")
                            };

                            self.ex.set_directory(Path::new(&path), &self.search);
                        }
                    }
                });

                ui.separator();

                //Button Styling
                let style = ui.style_mut();
                style.visuals.button_frame = false;
                style.spacing.button_padding = Vec2::new(0.0, 0.0);

                let files = &self.ex.files;

                if !files.is_empty() {
                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .column(Size::relative(0.45))
                        .column(Size::relative(0.25))
                        .column(Size::relative(0.15))
                        .column(Size::relative(0.15))
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
                                            ui.add(Button::new(&format!("🖹  {}", name)).wrap(false))
                                        } else {
                                            ui.add(Button::new(&format!("🗀  {}", name)).wrap(false))
                                        };

                                        if button.clicked() && file.is_dir() {
                                            new_dir = Some(file.clone());
                                        }

                                        if button.double_clicked() && !file.is_dir() {
                                            if let Err(e) = ex::open(&file) {
                                                //TODO: print to error bar like Onivim
                                                dbg!(e);
                                            }
                                        }

                                        if button.middle_clicked() && file.is_dir() {
                                            //TODO: don't focus this new tab
                                            tab = Some(file.to_path_buf());
                                        }

                                        button.context_menu(|ui| {
                                            if ui.button("Copy").clicked() {
                                                self.buffer =
                                                    Some(Buffer::Copy(file.to_path_buf()));
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
                                                    new_dir =
                                                        Some(self.ex.current_path().to_path_buf());
                                                    ui.close_menu();
                                                };
                                                ui.separator();
                                            }

                                            if ui.button("Rename").clicked() {
                                                self.event = Some(Event::Rename(
                                                    name.clone(),
                                                    file.to_path_buf(),
                                                ));
                                                ui.close_menu();
                                            };

                                            ui.separator();

                                            if ui.button("Delete").clicked() {
                                                self.popup = true;
                                                self.event =
                                                    Some(Event::Delete(file.to_path_buf()));

                                                ui.close_menu();
                                            };
                                        });
                                    });
                                }

                                row.col(|ui| {
                                    if let Some(date) = ex::last_modified(&file) {
                                        ui.button(date);
                                    }
                                });

                                row.col(|ui| {
                                    if file.is_dir() {
                                        ui.button("File folder");
                                    } else if let Some(ex) = file.extension() {
                                        let ex = ex.to_string_lossy().to_string();
                                        let unknown = format!(".{ex} file");
                                        let file_type = match ex.as_str() {
                                            "lnk" => "Shortcut",
                                            "zip" => "zip Archive",
                                            "exe" => "Application",
                                            _ => &unknown,
                                        };
                                        ui.button(file_type);
                                    } else if let Some(file_name) = file.file_name() {
                                        let file_name = file_name.to_string_lossy().to_string();
                                        if file_name.starts_with('.') {
                                            let file_type = match file_name.as_str() {
                                                ".gitignore" => "Git Ignore",
                                                ".gitconfig" => "Git Config",
                                                _ => "Unknown dot file",
                                            };
                                            ui.button(file_type);
                                        }
                                    }
                                });

                                row.col(|ui| {
                                    if let Some(size) = ex::file_size(&file) {
                                        ui.button(size);
                                    }
                                });
                            });
                        });
                } else {
                    ui.centered_and_justified(|ui| ui.label("Folder is empty."));
                };
            })
            .response;

        if let Some(dir) = new_dir {
            self.ex.set_directory(&dir, &self.search);
        }

        (response, tab)
    }
    pub fn previous(&mut self) {
        self.ex.previous();
    }
    pub fn next(&mut self) {
        //TODO: keep history of paths visited
        // self.ex.next();
    }
}
