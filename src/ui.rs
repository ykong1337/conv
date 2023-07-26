use std::sync::atomic::Ordering;

use clap_builder::ValueEnum;
use eframe::Frame;
use egui::{ComboBox, Context, ProgressBar};

use crate::config::{DOWNLOADED, FILE_SIZE, Language, Model};
use crate::conv::Conv;
use crate::utils::{DOWNLOADING, MERGE, WHISPER};

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
            ui.horizontal(|ui| {
                ComboBox::from_label("模型")
                    .selected_text(format!("{}", self.config.model))
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        for i in Model::value_variants() {
                            ui.selectable_value(&mut self.config.model, *i, format!("{}", *i));
                        }
                    });
                if ui.button("下载模型").clicked() {
                    DOWNLOADING.store(false, Ordering::Relaxed);
                    let model = self.config.model;
                    if std::fs::remove_file(model.get_path()).is_err() {}
                    self.rt.spawn(async move {
                        if model.download().await.is_err() {
                            DOWNLOADING.store(false, Ordering::Relaxed);
                        }
                    });
                }
            });

            if ui.button("音频 -> 字幕").clicked() {
                if !WHISPER.load(Ordering::Relaxed) && !DOWNLOADING.load(Ordering::Relaxed) {
                    self.whisper();
                }
            }
            if DOWNLOADING.load(Ordering::Relaxed) {
                ui.label("下载模型中");
                ui.add(ProgressBar::new(DOWNLOADED.load(Ordering::Relaxed) as f32 / FILE_SIZE.load(Ordering::Relaxed) as f32).desired_width(200.0));
            }
            ui.label(if WHISPER.load(Ordering::Relaxed) { "转换中" } else { "转换结束" });

            ui.separator();

            if ui.button("合并音频/图片/字幕").clicked() {
                if !MERGE.load(Ordering::Relaxed) {
                    self.ffmpeg_merge();
                }
            }
            ui.label(if MERGE.load(Ordering::Relaxed) { "合并中" } else { "合并结束" });
        });
    }
}