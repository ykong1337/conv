use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use clap_builder::ValueEnum;
use eframe::{CreationContext, Frame};
use egui::{ComboBox, Context, FontId};
use egui::FontFamily::Proportional;
use egui::TextStyle::*;
use tokio::runtime::Runtime;
use whisper_cli::{Language, Size};
use crate::font::load_fonts;
use crate::utils::{ffmpeg_merge, MERGE, WHISPER, whisper};

#[derive(Clone)]
pub struct Conv {
    rt: Arc<Runtime>,
    files: Arc<Mutex<Files>>,
    config: Config,
}

#[derive(Clone)]
struct Config {
    lang: Language,
    size: Size,
}

#[derive(Debug, Clone, Default)]
pub struct Files {
    pub audio: Option<PathBuf>,
    pub image: Option<PathBuf>,
    pub subtitle: Option<PathBuf>,
}

impl Conv {
    pub fn new(cc: &CreationContext) -> Box<Self> {
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

        Box::new(Self {
            rt: Arc::new(rt),
            files: Default::default(),
            config: Config { lang: Language::Auto, size: Size::Medium },
        })
    }

    fn open_audio(&self, files: Arc<Mutex<Files>>) {
        self.rt.spawn(async move {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Audio File", &["mp3", "wav"])
                .pick_file() {
                files.lock().unwrap().audio = Some(path);
            }
        });
    }

    fn open_image(&self, files: Arc<Mutex<Files>>) {
        self.rt.spawn(async move {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Image File", &["jpg", "png"])
                .pick_file() {
                files.lock().unwrap().image = Some(path);
            }
        });
    }

    fn open_subtitle(&self, files: Arc<Mutex<Files>>) {
        self.rt.spawn(async move {
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
            if ui.button("选择音频").clicked() {
                self.open_audio(self.files.clone());
            }
            ui.label(format!("音频: {}", if let Some(ref p) = self.files.lock().unwrap().audio {
                p.to_str().unwrap()
            } else {
                "None"
            }));

            if ui.button("选择背景图片").clicked() {
                self.open_image(self.files.clone());
            }
            ui.label(format!("背景图片: {}", if let Some(ref p) = self.files.lock().unwrap().image {
                p.to_str().unwrap()
            } else {
                "None"
            }));

            if ui.button("选择字幕").clicked() {
                self.open_subtitle(self.files.clone());
            }
            ui.label(format!("字幕: {}", if let Some(ref p) = self.files.lock().unwrap().subtitle {
                p.to_str().unwrap()
            } else {
                "None"
            }));

            ui.separator();

            ui.label("Whisper");
            ComboBox::from_label("语言")
                .selected_text(<&str>::from(self.config.lang))
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap = Some(false);
                    for i in Language::value_variants() {
                        ui.selectable_value(&mut self.config.lang, *i, <&str>::from(*i));
                    }
                });
            ComboBox::from_label("模型")
                .selected_text(format!("{}", self.config.size))
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap = Some(false);
                    for i in Size::value_variants() {
                        ui.selectable_value(&mut self.config.size, *i, format!("{}", *i));
                    }
                });

            if ui.button("音频 -> 字幕").clicked() {
                if !WHISPER.load(Ordering::Relaxed) {
                    if let Some(ref path) = self.files.lock().unwrap().audio {
                        whisper(self.rt.clone(), path.clone(), self.config.lang, self.config.size);
                    }
                }
            }
            ui.label(if WHISPER.load(Ordering::Relaxed) { "转换中" } else { "转换结束" });

            ui.separator();

            if ui.button("合并音频/图片/字幕").clicked() {
                if !MERGE.load(Ordering::Relaxed) {
                    let file = self.files.lock().unwrap();
                    ffmpeg_merge(self.rt.clone(), file.audio.clone(), file.image.clone(), file.subtitle.clone());
                }
            }
            ui.label(if MERGE.load(Ordering::Relaxed) { "合并中" } else { "合并结束" });
        });
    }
}