use eframe::{egui::*, epi};
use ex::Ex;
use std::path::PathBuf;

#[derive(Default)]
struct Rename {
    pub path: PathBuf,
    pub name: String,
}

pub struct App {
    files: Vec<PathBuf>,
    copied_file: PathBuf,
    renamed_file: Rename,
}

impl App {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            copied_file: PathBuf::new(),
            renamed_file: Rename::default(),
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "Explorer"
    }

    fn setup(&mut self, _ctx: &Context, _frame: &epi::Frame, _storage: Option<&dyn epi::Storage>) {
        self.files = Ex::get_files().unwrap();
    }

    fn update(&mut self, ctx: &Context, frame: &epi::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                menu::menu_button(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        #[allow(unused_must_use)]
        SidePanel::left("side_panel").show(ctx, |ui| {
            ui.button("Quick Access:");
            ui.button("Downloads");
            ui.button("Documents");
            ui.separator();
            if ui.button("Drives:").clicked() {
                self.files = Ex::get_drives();
            };
            if ui.button("C:/").clicked() {
                Ex::set_directory(&PathBuf::from("C://"));
                self.files = Ex::get_files().unwrap();
            };
            if ui.button("D:/").clicked() {
                Ex::set_directory(&PathBuf::from("D://"));
                self.files = Ex::get_files().unwrap();
            };
        });

        CentralPanel::default().show(ctx, |ui| {
            warn_if_debug_build(ui);

            let text_style = TextStyle::Body;
            let row_height = ui.text_style_height(&text_style);
            let files = self.files.clone();
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
                                        Ex::rename(
                                            &self.renamed_file.name,
                                            &self.renamed_file.path,
                                        )
                                        .unwrap();

                                        //reset and update
                                        self.renamed_file = Rename::default();
                                        self.files = Ex::get_files().unwrap();
                                    }
                                } else if row == 0 {
                                    Ex::previous_dir().unwrap();
                                    self.files = Ex::get_files().unwrap();
                                } else if file.is_dir() {
                                    Ex::set_directory(file).unwrap();
                                    self.files = Ex::get_files().unwrap();
                                } else {
                                    Ex::open(file).unwrap();
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

                            if let Some(size) = Ex::file_size(file) {
                                columns[1].label(size);
                            }
                        });
                    } else {
                        ui.columns(2, |columns| {
                            let name = file.to_string_lossy();
                            if name == "C:\\" || name == "D:\\" {
                                if columns[0].button("../").clicked() {
                                    self.files = Ex::get_drives();
                                }
                            } else if columns[0].button(format!("{}", name)).clicked() {
                                Ex::set_directory(file).unwrap();
                                self.files = Ex::get_files().unwrap();
                            }
                        });
                    }
                }
            });
        });
    }
}
