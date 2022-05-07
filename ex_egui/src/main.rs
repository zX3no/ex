use app::App;

mod app;
mod browser;

fn main() {
    eframe::run_native(
        "ex",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(App::new(cc))),
    );
}
