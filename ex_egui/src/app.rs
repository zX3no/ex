#![allow(unreachable_code)]
use eframe::egui::*;
use egui_extras::*;
use ex_core::Ex;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Default)]
pub struct Rename {
    pub path: PathBuf,
    pub name: String,
}

pub struct App {
    ex: Ex,
    copied_file: PathBuf,
    renamed_file: Rename,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        //dark mode
        cc.egui_ctx.set_visuals(Visuals::dark());

        Self {
            ex: Ex::new(),
            copied_file: PathBuf::new(),
            renamed_file: Rename::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let Self {
            ex,
            copied_file: _,
            renamed_file: _,
        } = self;

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                menu::menu_button(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        SidePanel::left("side_panel").show(ctx, |ui| {
            ui.style_mut().visuals.button_frame = false;

            CollapsingHeader::new("Quick Access")
                .default_open(true)
                .show(ui, |ui| {
                    if ui.button("Desktop").clicked() {
                        ex.set_directory(Path::new("C:\\Users\\Bay\\Desktop"));
                    }
                    if ui.button("Downloads").clicked() {
                        ex.set_directory(Path::new("C:\\Users\\Bay\\Downloads"));
                    }
                    if ui.button("Music").clicked() {
                        ex.set_directory(Path::new("D:\\Music"));
                    }
                });

            CollapsingHeader::new("Drives")
                .default_open(true)
                .show(ui, |ui| {
                    if ui.button("C:\\").clicked() {
                        ex.set_directory(&PathBuf::from("C:\\"));
                    };
                    if ui.button("D:\\").clicked() {
                        ex.set_directory(&PathBuf::from("D:\\"));
                    };
                });
        });

        //get the files
        let mut files = ex.get_files().to_owned();
        let current_dir = if !files.is_empty() {
            files.remove(0)
        } else {
            PathBuf::default()
        };

        let current_dir_string = current_dir.to_string_lossy().to_string();

        CentralPanel::default()
            .show(ctx, |ui| {
                warn_if_debug_build(ui);

                //Header
                //TODO: only show the first 5 paths
                if !files.is_empty() {
                    let splits: Vec<&str> = current_dir_string
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

                                ex.set_directory(Path::new(&path));
                            }
                        }
                    });

                    ui.separator();
                }

                //Button Styling
                let style = ui.style_mut();
                style.visuals.button_frame = false;
                style.spacing.button_padding = Vec2::new(0.0, 0.0);

                if !files.is_empty() {
                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .column(Size::relative(0.45))
                        .column(Size::relative(0.20))
                        .column(Size::relative(0.20))
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
                                        let pressed_enter = ui.input().key_pressed(Key::Enter);
                                        let response = if file == self.renamed_file.path {
                                            //TODO: pre select the file name
                                            let r = ui
                                                .text_edit_singleline(&mut self.renamed_file.name);
                                            r.request_focus();
                                            r
                                        } else if file.is_file() {
                                            ui.add(Button::new(&format!("🖹 {}", name)).wrap(false))
                                        } else {
                                            ui.add(Button::new(&format!("🗀 {}", name)).wrap(false))
                                        };

                                        if response.clicked() {
                                            if file == self.renamed_file.path {
                                                if pressed_enter || response.lost_focus() {
                                                    ex.rename(
                                                        &self.renamed_file.name,
                                                        &self.renamed_file.path,
                                                    )
                                                    .unwrap();

                                                    //reset and update
                                                    self.renamed_file = Rename::default();
                                                }
                                            } else if file.is_dir() {
                                                ex.set_directory(&file);
                                            }
                                        }

                                        if response.double_clicked() && !file.is_dir() {
                                            if let Err(e) = ex.open(&file) {
                                                //TODO: print to error bar like Onivim
                                                dbg!(e);
                                            }
                                        }

                                        response.context_menu(|ui| {
                                            if ui.button("Cut").clicked() {
                                                ui.close_menu();
                                            };
                                            if ui.button("Copy").clicked() {
                                                self.copied_file = file.clone();
                                                ui.close_menu();
                                            };
                                            ui.separator();
                                            if ui.button("Rename").clicked() {
                                                self.renamed_file.path = file.clone();
                                                self.renamed_file.name = name.clone();
                                                ui.close_menu();
                                            };
                                            ui.separator();
                                            if ui.button("Delete").clicked() {
                                                ui.close_menu();
                                            };
                                        });
                                    });
                                }

                                row.col(|ui| {
                                    if let Some(date) = Ex::last_modified(&file) {
                                        ui.button(date);
                                    }
                                });

                                row.col(|ui| {
                                    if let Some(ex) = file.extension() {
                                        let ex = ex.to_string_lossy().to_string();
                                        let unknown = format!(".{ex} file");
                                        let file_type = match ex.as_str() {
                                            "lnk" => "Shortcut",
                                            "zip" => "zip Archive",
                                            "exe" => "Application",
                                            _ => unknown.as_str(),
                                        };
                                        ui.button(file_type);
                                    } else if let Some(file_name) = file.file_name() {
                                        let file_name = file_name.to_string_lossy().to_string();
                                        if file.is_dir() {
                                            ui.button("File folder");
                                        } else if file_name.starts_with('.') {
                                            let file_type = match file_name.as_str() {
                                                ".gitignore" => "Git Ignore",
                                                ".gitconfig" => "Git Config",
                                                _ => "Unknown dot file",
                                            };
                                            ui.button(file_type);
                                        } else {
                                            ui.button("File folder");
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
                };
            })
            .response
            .context_menu(|ui| {
                if ui.button("New File").clicked() {
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
    }
}
