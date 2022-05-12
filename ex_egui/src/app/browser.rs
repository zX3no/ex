use eframe::egui::*;
use egui_extras::*;
use ex_core::Ex;
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
    refocus: bool,
    popup: bool,
}

impl Browser {
    pub fn new() -> Self {
        Self {
            ex: Ex::new(),
            search: String::new(),
            event: None,
            buffer: None,
            refocus: true,
            popup: false,
        }
    }
    pub fn set_path(mut self, path: &Path) -> Self {
        self.ex.set_directory(path);
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
    pub fn paste(&mut self, ui: &mut Ui) {
        if self.buffer.is_some() {
            if ui.button("Paste").clicked() {
                ui.close_menu();
            };
            ui.separator();
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
                                self.ex.delete(path).unwrap();

                                //Update files.
                                self.ex.refresh();
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
            self.paste(ui);

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

        let response = CentralPanel::default()
            .show(ctx, |ui| {
                //Header
                //TODO: only show the first 5 paths
                let current_dir = self.ex.current_path_string();

                let files = self.ex.get_files().to_owned();
                let files: Vec<PathBuf> = files
                    .into_iter()
                    .filter(|file| {
                        if self.search.is_empty() {
                            true
                        } else {
                            let file_name = file
                                .file_name()
                                .unwrap_or(file.as_os_str())
                                .to_string_lossy()
                                .to_ascii_lowercase();
                            file_name.contains(&self.search)
                        }
                    })
                    .collect();

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

                            self.ex.set_directory(Path::new(&path));
                        }
                    }
                });

                ui.separator();

                //Button Styling
                let style = ui.style_mut();
                style.visuals.button_frame = false;
                style.spacing.button_padding = Vec2::new(0.0, 0.0);

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
                                        if self.text_query(ui, &file) {
                                            //is_file() can fail on fails that have bad permissions.
                                            let button = if file.is_file() {
                                                ui.add(
                                                    Button::new(&format!("🖹  {}", name))
                                                        .wrap(false),
                                                )
                                            } else {
                                                ui.add(
                                                    Button::new(&format!("🗀  {}", name))
                                                        .wrap(false),
                                                )
                                            };
                                            if button.clicked() && file.is_dir() {
                                                self.ex.set_directory(&file);
                                            }
                                            if button.double_clicked() && !file.is_dir() {
                                                if let Err(e) = self.ex.open(&file) {
                                                    //TODO: print to error bar like Onivim
                                                    dbg!(e);
                                                }
                                            }
                                            if button.middle_clicked() && file.is_dir() {
                                                tab = Some(file.clone());
                                            }
                                            button.context_menu(|ui| {
                                                if ui.button("Copy").clicked() {
                                                    self.buffer = Some(Buffer::Copy(file.clone()));
                                                    ui.close_menu();
                                                };

                                                if ui.button("Cut").clicked() {
                                                    self.buffer = Some(Buffer::Cut(file.clone()));
                                                    ui.close_menu();
                                                };

                                                ui.separator();

                                                self.paste(ui);

                                                if ui.button("Rename").clicked() {
                                                    self.event = Some(Event::Rename(
                                                        name.clone(),
                                                        file.clone(),
                                                    ));
                                                    ui.close_menu();
                                                };

                                                ui.separator();

                                                if ui.button("Delete").clicked() {
                                                    self.popup = true;
                                                    self.event = Some(Event::Delete(file.clone()));

                                                    ui.close_menu();
                                                };
                                            });
                                        }
                                    });
                                }

                                row.col(|ui| {
                                    if let Some(date) = Ex::last_modified(&file) {
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
                                    if let Some(size) = Ex::file_size(&file) {
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

        (response, tab)
    }
    fn text_query(&mut self, ui: &mut Ui, file: &PathBuf) -> bool {
        if let Some(event) = &mut self.event {
            let (text, path) = match event {
                Event::NewFile(text, path) => (text, path),
                Event::Rename(text, path) => (text, path),
                _ => return true,
            };

            if file == path {
                let text_edit = ui.text_edit_singleline(text);

                if self.refocus {
                    text_edit.request_focus();
                    self.refocus = false;
                }

                if text_edit.lost_focus()
                    || ui.input().key_pressed(Key::Enter)
                    || text_edit.clicked_elsewhere()
                {
                    eprintln!("Renaming {:?} to {}", path, text);
                    self.ex.rename(text, path).unwrap();

                    //reset
                    self.event = None;
                    self.refocus = true;
                    self.ex.refresh();

                    return false;
                }
            }
        }
        true
    }

    pub fn previous(&mut self) {
        self.ex.previous();
    }

    pub fn next(&mut self) {
        //TODO: keep history of paths visited
        // self.ex.next();
    }
}
