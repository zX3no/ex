use super::browser::{Browser, BrowserEvent};
use eframe::egui::*;
use std::path::Path;

pub struct Tabs {
    browsers: Vec<Browser>,
    index: usize,
    search: String,
}

impl Tabs {
    pub fn new() -> Self {
        Self {
            browsers: vec![Browser::new()],
            index: 0,
            search: String::new(),
        }
    }
    pub fn add(&mut self, path: &Path) {
        self.index = self.browsers.len();
        self.browsers.push(Browser::new().set_path(path));
    }
    pub fn add_new(&mut self) {
        self.index = self.browsers.len();
        self.browsers.push(Browser::new());
    }
    pub fn remove(&mut self, i: usize) {
        if self.browsers.len() != 1 {
            self.browsers.remove(i);
            self.index = self.index.saturating_sub(1);
        }
    }
    pub fn ui(&mut self, ctx: &Context) {
        match self.browsers[self.index].ui(ctx, &self.search) {
            BrowserEvent::Add(path) => self.add(&path),
            BrowserEvent::None => (),
        };
    }
    pub fn set_directory(&mut self, path: &Path) {
        self.browsers[self.index].ex.set_directory(path);
    }
    pub fn header(&mut self, ctx: &Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mut remove = None;
                for (i, b) in self.browsers.iter().enumerate() {
                    let selected = i == self.index;

                    let label = ui.selectable_label(selected, b.title());
                    if label.clicked() {
                        self.index = i;
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

                //Delete tab
                if let Some(i) = remove {
                    self.remove(i);
                }

                if ui.button("+").clicked() {
                    self.add_new();
                };

                ui.with_layout(Layout::right_to_left(), |ui| {
                    ui.add(TextEdit::singleline(&mut self.search));
                });
            });
        });
    }
    pub fn quick_access(&mut self, ctx: &Context) {
        SidePanel::left("side_panel").show(ctx, |ui| {
            ui.style_mut().visuals.button_frame = false;

            let mut item = |ui: &mut Ui, label: &str, path: &str| {
                let item = ui.button(label);
                let path = Path::new(path);
                if item.clicked() {
                    self.set_directory(path);
                }
                if item.middle_clicked() {
                    self.add(path);
                }
            };

            CollapsingHeader::new("Quick Access")
                .default_open(true)
                .show(ui, |ui| {
                    item(ui, "Desktop", "C:\\Users\\Bay\\Desktop");
                    item(ui, "Downloads", "C:\\Users\\Bay\\Downloads");
                    item(ui, "Music", "D:\\Music");
                });

            CollapsingHeader::new("Drives")
                .default_open(true)
                .show(ui, |ui| {
                    item(ui, "C:\\", "C:\\");
                    item(ui, "D:\\", "D:\\");
                });
        });
    }
}
