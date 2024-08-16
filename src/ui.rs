use std::sync::atomic::Ordering;

use eframe::Frame;
use egui::Context;

use crate::conv::Conv;
use crate::utils::MERGE;

impl eframe::App for Conv {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("选择音频").clicked() {
                self.open_audio(self.files.clone());
            }
            ui.label(format!(
                "音频: {}",
                if let Some(ref p) = self.files.lock().unwrap().audio {
                    p.file_name().unwrap().to_str().unwrap()
                } else {
                    "None"
                }
            ));

            if ui.button("选择背景图片").clicked() {
                self.open_image(self.files.clone());
            }
            ui.label(format!(
                "背景图片: {}",
                if let Some(ref p) = self.files.lock().unwrap().image {
                    p.file_name().unwrap().to_str().unwrap()
                } else {
                    "None"
                }
            ));

            if ui.button("选择字幕").clicked() {
                self.open_subtitle(self.files.clone());
            }
            ui.label(format!(
                "字幕: {}",
                if let Some(ref p) = self.files.lock().unwrap().subtitle {
                    p.file_name().unwrap().to_str().unwrap()
                } else {
                    "None"
                }
            ));

            ui.separator();

            if ui.button("合并音频/图片/字幕").clicked() {
                if !MERGE.load(Ordering::Relaxed) {
                    self.ffmpeg_merge();
                }
            }
            ui.label(if MERGE.load(Ordering::Relaxed) {
                "合并中"
            } else {
                "合并结束"
            });
        });
    }
}
