#![allow(unreachable_code)]

use eframe::egui::*;
use egui_extras::*;
use ex_core::Ex;
use std::path::{Path, PathBuf};

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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
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
            if ui.button("Quick Access:").clicked() {
                //todo
            }
            if ui.button("Desktop").clicked() {
                ex.set_directory(Path::new("C:/Users/Bay/Desktop"));
                //todo
            }
            if ui.button("Downloads").clicked() {
                //todo
            }
            if ui.button("Documents").clicked() {
                //todo
            }
            ui.separator();
            if ui.button("Drives:").clicked() {
                ex.set_drives();
            };
            if ui.button("C:/").clicked() {
                ex.set_directory(&PathBuf::from("C:/"));
            };
            if ui.button("D:/").clicked() {
                ex.set_directory(&PathBuf::from("D:/"));
            };
        });

        CentralPanel::default().show(ctx, |ui| {
            warn_if_debug_build(ui);

            let files = ex.get_files().to_owned();

            TableBuilder::new(ui)
                .column(Size::remainder().at_least(280.0))
                .column(Size::remainder().at_least(20.0))
                .column(Size::remainder().at_least(20.0))
                .column(Size::remainder().at_least(20.0))
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
                    body.rows(20.0, files.len(), |i, mut row| {
                        let file = files[i].clone();

                        if let Some(name) = file.file_name() {
                            let name = name.to_string_lossy().to_string();

                            row.col(|ui| {
                                let pressed_enter = ui.input().key_pressed(Key::Enter);
                                let response = if i == 0 {
                                    //TODO: clip the name
                                    ui.button(format!("../{name}"))
                                } else if file == self.renamed_file.path {
                                    //TODO: pre select the file name

                                    //Rename textbox
                                    let r = ui.text_edit_singleline(&mut self.renamed_file.name);
                                    r.request_focus();
                                    r
                                } else {
                                    ui.button(&name)
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
                                    } else if i == 0 {
                                        ex.previous_dir().unwrap();
                                    } else if file.is_dir() {
                                        ex.set_directory(&file);
                                    }
                                }

                                if response.double_clicked() && i != 0 && !file.is_dir() {
                                    ex.open(&file).unwrap();
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
                        } else {
                            row.col(|ui| {
                                ui.columns(2, |columns| {
                                    let name = file.to_string_lossy();
                                    if ex.get_files().len() > 2 {
                                        if columns[0].button("../").clicked() {
                                            ex.set_drives();
                                        }
                                    } else {
                                        //
                                        if columns[0].button(format!("{}", name)).clicked() {
                                            ex.set_directory(&file);
                                        }
                                    }
                                });
                            });
                        }
                        row.col(|_| {});
                        row.col(|_| {});
                        row.col(|_| {});
                    });
                });
        });
    }
}
