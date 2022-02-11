use eframe::{egui::*, epi};
use jwalk::WalkDir;
use std::path::{Path, PathBuf};
use std::{env, io, os::windows::prelude::MetadataExt};

pub struct App {
    files: Vec<PathBuf>,
    dropped_files: Vec<DroppedFile>,
}

impl App {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
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

        //TODO: right click menu

        CentralPanel::default().show(ctx, |ui| {
            warn_if_debug_build(ui);

            if !self.dropped_files.is_empty() {
                ui.group(|ui| {
                    ui.label("Dropped files:");

                    for file in &self.dropped_files {
                        let mut info = if let Some(path) = &file.path {
                            path.display().to_string()
                        } else if !file.name.is_empty() {
                            file.name.clone()
                        } else {
                            "???".to_owned()
                        };
                        if let Some(bytes) = &file.bytes {
                            info += &format!(" ({} bytes)", bytes.len());
                        }
                        ui.label(info);
                    }
                });
            }

            self.detect_files_being_dropped(ctx);

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
                            } else if columns[0].button(&name).clicked() {
                                if file.is_dir() {
                                    self.set_directory(file.as_path()).unwrap();
                                } else {
                                    open::that(file.as_path()).unwrap();
                                }
                            }

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
