use eframe::egui::*;
use std::path::{Path, PathBuf};

use crate::browser::Browser;

pub struct App {
    browser: Vec<Browser>,
    selected_browser: usize,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        //dark mode
        cc.egui_ctx.set_visuals(Visuals::dark());

        Self {
            browser: vec![Browser::new()],
            selected_browser: 0,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let Self {
            browser,
            selected_browser,
        } = self;

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mut remove = None;
                for (i, b) in browser.iter().enumerate() {
                    let selected = i == *selected_browser;

                    let label = ui.selectable_label(selected, b.title());
                    if label.clicked() {
                        *selected_browser = i;
                    };

                    if label.middle_clicked() {
                        remove = Some(i);
                    }

                    label.context_menu(|ui| {
                        if ui.button("Close").clicked() {
                            remove = Some(i);
                            ui.close_menu();
                        };
                    });
                }

                if let Some(i) = remove {
                    if browser.len() != 1 {
                        browser.remove(i);
                        *selected_browser = selected_browser.saturating_sub(1);
                    }
                }

                if ui.button("+").clicked() {
                    browser.push(Browser::new());
                    *selected_browser += 1;
                };
            });
        });

        let b = &mut browser[*selected_browser];

        SidePanel::left("side_panel").show(ctx, |ui| {
            ui.style_mut().visuals.button_frame = false;

            CollapsingHeader::new("Quick Access")
                .default_open(true)
                .show(ui, |ui| {
                    let desktop = ui.button("Desktop");
                    if desktop.clicked() {
                        b.ex.set_directory(Path::new("C:\\Users\\Bay\\Desktop"));
                    }
                    if desktop.middle_clicked() {
                        // browser.push(Browser::new().set_path(Path::new("C:\\Users\\Bay\\Desktop")));
                    }
                    if ui.button("Downloads").clicked() {
                        b.ex.set_directory(Path::new("C:\\Users\\Bay\\Downloads"));
                    }
                    if ui.button("Music").clicked() {
                        b.ex.set_directory(Path::new("D:\\Music"));
                    }
                });

            CollapsingHeader::new("Drives")
                .default_open(true)
                .show(ui, |ui| {
                    if ui.button("C:\\").clicked() {
                        b.ex.set_directory(&PathBuf::from("C:\\"));
                    };
                    if ui.button("D:\\").clicked() {
                        b.ex.set_directory(&PathBuf::from("D:\\"));
                    };
                });
        });

        b.ui(ctx);

        //TODO: footer
        // TopBottomPanel::bottom("footer").show(ctx, |ui| {
        //     ui.label("Hello World!");
        // });
    }
}
