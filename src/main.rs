#![windows_subsystem = "windows"]

use eframe::NativeOptions;
use egui::{Vec2, ViewportBuilder};

use crate::conv::Conv;

mod conv;
mod font;
mod ui;
mod utils;

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    let viewport = ViewportBuilder {
        resizable: Some(false),
        inner_size: Some(Vec2::new(400.0, 500.0)),
        maximize_button: Some(false),
        ..Default::default()
    };

    let option = NativeOptions {
        viewport,
        ..NativeOptions::default()
    };
    eframe::run_native("Conv", option, Box::new(|cc| Conv::new(cc))).unwrap();
}
