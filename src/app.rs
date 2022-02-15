use eframe::{egui::*, epi};
use jwalk::WalkDir;
use std::os::windows::prelude::MetadataExt;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

#[derive(Default)]
struct Rename {
    pub path: PathBuf,
    pub name: String,
}

pub struct App {
    files: Vec<PathBuf>,
    copied_file: PathBuf,
    renamed_file: Rename,
    dropped_files: Vec<DroppedFile>,
}

impl App {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            copied_file: PathBuf::new(),
            renamed_file: Rename::default(),
            dropped_files: Vec::new(),
        }
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
            .skip_hidden(false)
            .into_iter()
            .flat_map(|entry| {
                if let Ok(entry) = entry {
                    Some(entry.path())
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
        // self.files.sort_by_key(|a| a.starts_with("."));
        // self.files
        //     .sort_by_key(|a| a.to_string_lossy().to_lowercase());
        self.files.sort_by_key(|a| !a.is_dir());
    }

    fn detect_files_being_dropped(&mut self, ctx: &CtxRef) {
        // Preview hovering files:
        if !ctx.input().raw.hovered_files.is_empty() {
            let mut text = "Dropping files:\n".to_owned();
            for file in &ctx.input().raw.hovered_files {
                if let Some(path) = &file.path {
                    text += &format!("\n{}", path.display());
                } else if !file.mime.is_empty() {
                    text += &format!("\n{}", file.mime);
                } else {
                    text += "\n???";
                }
            }

            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

            let screen_rect = ctx.input().screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading,
                Color32::WHITE,
            );
        }

        // Collect dropped files:
        if !ctx.input().raw.dropped_files.is_empty() {
            self.dropped_files = ctx.input().raw.dropped_files.clone();
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "Explorer"
    }

    fn setup(&mut self, _ctx: &CtxRef, _frame: &epi::Frame, _storage: Option<&dyn epi::Storage>) {
        self.updates_files().unwrap();
    }

    fn update(&mut self, ctx: &CtxRef, frame: &epi::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                menu::menu_button(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            warn_if_debug_build(ui);

            // if !self.dropped_files.is_empty() {
            //     ui.group(|ui| {
            //         ui.label("Dropped files:");

            //         for file in &self.dropped_files {
            //             let mut info = if let Some(path) = &file.path {
            //                 path.display().to_string()
            //             } else if !file.name.is_empty() {
            //                 file.name.clone()
            //             } else {
            //                 "???".to_owned()
            //             };
            //             if let Some(bytes) = &file.bytes {
            //                 info += &format!(" ({} bytes)", bytes.len());
            //             }
            //             ui.label(info);
            //         }
            //     });
            // }

            // self.detect_files_being_dropped(ctx);

            let row_height = ui.fonts()[TextStyle::Body].row_height();
            let files = self.files.clone();
            let num_rows = files.len();

            ScrollArea::vertical().show_rows(ui, row_height, num_rows, |ui, row_range| {
                for row in row_range {
                    let file = files.get(row).unwrap();
                    if let Some(name) = file.file_name() {
                        let name = name.to_string_lossy().to_string();
                        let pressed_enter = ui.input().key_pressed(Key::Enter);

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
                                    if pressed_enter {
                                        //rename
                                        let old_path = self.renamed_file.path.clone();
                                        let mut new_path = old_path.clone();
                                        let name = &self.renamed_file.name;
                                        new_path.set_file_name(name);
                                        fs::rename(old_path, new_path).unwrap();

                                        //reset and update
                                        self.renamed_file = Rename::default();
                                        self.updates_files().unwrap();
                                    }
                                } else if row == 0 {
                                    self.previous_dir().unwrap();
                                } else if file.is_dir() {
                                    self.set_directory(file.as_path()).unwrap();
                                } else {
                                    open::that(file.as_path()).unwrap();
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

                            if let Ok(metadata) = file.metadata() {
                                let size = metadata.file_size();
                                let size_str = if size < 1000 {
                                    format!("{} bytes", size)
                                } else if size < 1_000_000 {
                                    format!("{} kilobytes", size / 1000)
                                } else {
                                    format!("{} megabytes", size / 1_000_000)
                                };

                                if file.is_dir() {
                                    columns[1].label("");
                                } else {
                                    columns[1].label(&size_str);
                                }
                            }
                        });
                    }
                }
            });
        });
    }
}
