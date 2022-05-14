use super::browser::Browser;
use eframe::egui::*;
use std::path::Path;

pub struct Tabs {
    browsers: Vec<Browser>,
    index: usize,
}

impl Tabs {
    pub fn new() -> Self {
        Self {
            browsers: vec![Browser::new()],
            index: 0,
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
    pub fn body(&mut self, ctx: &Context) {
        if let Some(path) = self.browsers[self.index].ui(ctx) {
            self.add(&path);
        };
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
                    let search = &mut self.browsers[self.index].search;

                    //change box color
                    let visuals = &mut ui.style_mut().visuals;
                    visuals.extreme_bg_color = visuals.window_fill();

                    ui.add(TextEdit::singleline(search).desired_width(150.0));
                });
            });
        });
    }
    pub fn quick_access(&mut self, ctx: &Context) {
        SidePanel::left("side_panel").show(ctx, |ui| {
            let mut item = |ui: &mut Ui, label: &str, path: &str| {
                let item = ui.button(label);
                let path = Path::new(path);
                if item.clicked() {
                    let browser = &mut self.browsers[self.index];
                    browser.ex.set_directory(path, &browser.search);
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

    pub fn side_buttons(&mut self, ctx: &Context) {
        if ctx.input().pointer.button_clicked(PointerButton::Back) {
            self.browsers[self.index].previous();
        }
        if ctx.input().pointer.button_clicked(PointerButton::Forward) {
            self.browsers[self.index].next();
        }
    }
}
