use eframe::egui::*;
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

            let text_style = TextStyle::Body;
            let row_height = ui.text_style_height(&text_style);
            let files = ex.get_files().clone();
            let num_rows = files.len();

            ScrollArea::vertical().show_rows(ui, row_height, num_rows, |ui, row_range| {
                for row in row_range {
                    let file = files.get(row).unwrap();

                    if let Some(name) = file.file_name() {
                        let name = name.to_string_lossy().to_string();
                        let pressed_enter = ui.input().key_pressed(Key::Enter);

                        if ui.input().key_pressed(Key::Escape) {
                            //TODO: Close rename box
                        }

                        ui.columns(2, |columns| {
                            let response = if row == 0 {
                                columns[0].button(format!("../{name}"))
                            } else if file == &self.renamed_file.path {
                                //TODO: pre select the file name
                                let r =
                                    columns[0].text_edit_singleline(&mut self.renamed_file.name);
                                r.request_focus();
                                r
                            } else {
                                columns[0].button(&name)
                            };

                            if response.clicked() {
                                if file == &self.renamed_file.path {
                                    if pressed_enter || response.lost_focus() {
                                        ex.rename(&self.renamed_file.name, &self.renamed_file.path)
                                            .unwrap();

                                        //reset and update
                                        self.renamed_file = Rename::default();
                                    }
                                } else if row == 0 {
                                    ex.previous_dir().unwrap();
                                } else if file.is_dir() {
                                    ex.set_directory(file);
                                }
                            }

                            if response.double_clicked() && row != 0 && !file.is_dir() {
                                ex.open(file).unwrap();
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

                            // if let Some(size) = ex.file_size(file) {
                            //     columns[1].label(size);
                            // }
                        });
                    } else {
                        ui.columns(2, |columns| {
                            let name = file.to_string_lossy();
                            if ex.get_files().len() > 2 {
                                if columns[0].button("../").clicked() {
                                    ex.set_drives();
                                }
                            } else {
                                //
                                if columns[0].button(format!("{}", name)).clicked() {
                                    ex.set_directory(file);
                                }
                            }
                        });
                    }
                }
            });
        });
    }
}
