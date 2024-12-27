use eframe::{egui, Frame};
use egui::Context;

use crate::conv::{Conv, ALL, COUNT};
use crate::utils::MERGE;

impl eframe::App for Conv {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            let files = self.files.read();

            if ui.button("选择背景图片").clicked() {
                self.open_image(self.files.clone());
            }
            ui.label(format!(
                "背景图片: {}",
                if let Some(ref image) = files.image {
                    image
                        .file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default()
                } else {
                    "None"
                }
            ));
            if ui.button("选择音频").clicked() {
                self.open_audio(self.files.clone());
            }
            ui.collapsing("音频", |ui| {
                files.audio.iter().for_each(|path| {
                    ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                });
            });

            if ui.button("选择字幕").clicked() {
                self.open_subtitle(self.files.clone());
            }
            ui.collapsing("字幕", |ui| {
                files.subtitle.iter().for_each(|path| {
                    ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                });
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("合并音频/图片/字幕").clicked() && !MERGE.load() {
                    self.ffmpeg_merge();
                }
                if ui.button("停止").clicked() {
                    MERGE.store(false);
                }
            });
            ui.label(if MERGE.load() {
                format!("合并中 {}/{}", COUNT.load(), ALL.load())
            } else {
                "合并结束".to_string()
            });
        });
    }
}
