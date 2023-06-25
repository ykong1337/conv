use eframe::NativeOptions;
use egui::Vec2;
use crate::ui::Conv;

mod ui;
mod font;
mod utils;

fn main() {
    run();
}

fn run() {
    let option = NativeOptions {
        icon_data: None,
        initial_window_size: Some(Vec2::new(400.0, 300.0)),
        follow_system_theme: true,
        centered: true,
        resizable: false,
        ..NativeOptions::default()
    };
    eframe::run_native("Conv", option, Box::new(|cc| Box::new(Conv::new(cc))))
        .unwrap();
}
