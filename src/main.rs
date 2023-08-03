// #![windows_subsystem = "windows"]

use eframe::NativeOptions;
use egui::Vec2;

use crate::conv::Conv;

mod ui;
mod font;
mod utils;
mod whisper;
mod config;
mod conv;

fn main() {
    run();
}

fn run() {
    let option = NativeOptions {
        icon_data: None,
        initial_window_size: Some(Vec2::new(400.0, 400.0)),
        follow_system_theme: true,
        centered: true,
        resizable: false,
        ..NativeOptions::default()
    };
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _rt = rt.enter();
    eframe::run_native("Conv", option, Box::new(|cc| Conv::new(cc)))
        .unwrap();
}
