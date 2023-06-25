use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use eframe::{CreationContext, Frame};
use egui::{Context, FontId};
use egui::FontFamily::Proportional;
use egui::TextStyle::*;
use tokio::runtime::Runtime;
use crate::font::load_fonts;

#[derive(Debug, Clone)]
pub struct Conv {
    rt: Arc<Runtime>,
    files: Arc<Mutex<Files>>,
}

#[derive(Debug, Clone, Default)]
struct Files {
    audio: Option<PathBuf>,
    image: Option<PathBuf>,
    subtitle: Option<PathBuf>,
}

impl Conv {
    pub fn new(cc: &CreationContext) -> Self {
        load_fonts(&cc.egui_ctx);
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles = [
            (Heading, FontId::new(30.0, Proportional)),
            (Name("Heading2".into()), FontId::new(25.0, Proportional)),
            (Name("Context".into()), FontId::new(23.0, Proportional)),
            (Body, FontId::new(18.0, Proportional)),
            (Monospace, FontId::new(14.0, Proportional)),
            (Button, FontId::new(14.0, Proportional)),
            (Small, FontId::new(10.0, Proportional)),
        ]
            .into();
        cc.egui_ctx.set_style(style);

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        Self {
            rt: Arc::new(rt),
            files: Default::default(),
        }
    }

    fn open_audio(files: Arc<Mutex<Files>>) {
        thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Audio File", &["mp3", "wav"])
                .pick_file() {
                files.lock().unwrap().audio = Some(path);
            }
        });
    }

    fn open_image(files: Arc<Mutex<Files>>) {
        thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Image File", &["jpg", "png"])
                .pick_file() {
                files.lock().unwrap().image = Some(path);
            }
        });
    }

    fn open_subtitle(files: Arc<Mutex<Files>>) {
        thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Subtitle File", &["srt", "lrc", "vtt"])
                .pick_file() {
                files.lock().unwrap().subtitle = Some(path);
            }
        });
    }
}

impl eframe::App for Conv {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| ui.heading("Conv"));

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("选择音频");
                if ui.button("打开").clicked() {
                    Conv::open_audio(self.files.clone());
                }
            });
            ui.label(format!("音频: {}", if let Some(ref p) = self.files.lock().unwrap().audio {
                p.to_str().unwrap()
            } else {
                "None"
            }));

            ui.horizontal(|ui| {
                ui.label("选择背景图片");
                if ui.button("打开").clicked() {
                    Conv::open_image(self.files.clone());
                }
            });
            ui.label(format!("背景图片: {}", if let Some(ref p) = self.files.lock().unwrap().image {
                p.to_str().unwrap()
            } else {
                "None"
            }));

            ui.horizontal(|ui| {
                ui.label("选择字幕");
                if ui.button("打开").clicked() {
                    Conv::open_subtitle(self.files.clone());
                }
            });
            ui.label(format!("字幕: {}", if let Some(ref p) = self.files.lock().unwrap().subtitle {
                p.to_str().unwrap()
            } else {
                "None"
            }));

            ui.separator();


        });
    }
}