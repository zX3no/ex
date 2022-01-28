use app::App;
use eframe::NativeOptions;

mod app;

fn main() {
    let app = App::new();
    eframe::run_native(Box::new(app), NativeOptions::default());
}
