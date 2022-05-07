use app::App;

mod app;
mod browser;

fn main() {
    eframe::run_native(
        "My egui App",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(App::new(cc))),
    );
}
