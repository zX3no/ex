#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app::App;

mod app;

fn main() {
    let _ = eframe::run_native(
        "ex",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(App::new(cc))),
    );
}
