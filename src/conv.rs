use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use eframe::CreationContext;
use egui::FontFamily::Proportional;
use egui::FontId;
use egui::TextStyle::{Body, Button, Heading, Monospace, Name, Small};
use parking_lot::RwLock;

use crate::font::load_fonts;
use crate::utils::{merge, MERGE};

pub static COUNT: AtomicUsize = AtomicUsize::new(0);
pub static NUM: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
pub struct Conv {
    pub files: Arc<RwLock<Files>>,
}

#[derive(Debug, Clone, Default)]
pub struct Files {
    pub audio: Vec<PathBuf>,
    pub image: Option<PathBuf>,
    pub subtitle: Vec<PathBuf>,
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

        Box::new(Self {
            files: Default::default(),
        })
    }

    pub fn open_audio(&self, files: Arc<RwLock<Files>>) {
        tokio::spawn(async move {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Audio File", &["mp3", "wav"])
                .pick_files()
            {
                files.write().audio = path;
            }
        });
    }

    pub fn open_image(&self, files: Arc<RwLock<Files>>) {
        tokio::spawn(async move {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Image File", &["jpg", "png"])
                .pick_file()
            {
                files.write().image = Some(path);
            }
        });
    }

    pub fn open_subtitle(&self, files: Arc<RwLock<Files>>) {
        tokio::spawn(async move {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Subtitle File", &["srt", "lrc", "vtt"])
                .pick_files()
            {
                files.write().subtitle = path;
            }
        });
    }

    pub fn ffmpeg_merge(&self) {
        let file = self.files.read();
        let image = file.image.clone();
        let audio = file.audio.clone();
        let subtitle = file.subtitle.clone();
        let resources = audio.into_iter().zip(subtitle);
        MERGE.store(true, Ordering::Relaxed);
        NUM.store(resources.len(), Ordering::Release);

        tokio::spawn(async move {
            if let Some(ref image) = image {
                for (audio, subtitle) in resources {
                    COUNT.fetch_add(1, Ordering::AcqRel);
                    let image = image.to_string_lossy().to_string();
                    let current = std::env::current_dir().unwrap();
                    let subtitle_cache = Path::new(&uuid::Uuid::new_v4().to_string())
                        .with_extension(subtitle.extension().unwrap());
                    if !current.join(&subtitle_cache).exists() {
                        std::fs::copy(subtitle, current.join(&subtitle_cache)).unwrap();
                    }
                    let output = audio.with_extension("mp4");

                    let Ok(mut child) = merge(
                        audio.to_str().unwrap(),
                        image.as_str(),
                        subtitle_cache.to_str().unwrap(),
                        output.to_str().unwrap(),
                    ) else {
                        MERGE.store(false, Ordering::Relaxed);
                        return;
                    };
                    child.wait().ok();
                    std::fs::remove_file(subtitle_cache).ok();
                    std::fs::remove_file("out.mp4").ok();
                }
            }
            MERGE.store(false, Ordering::Relaxed);
            COUNT.store(0, Ordering::Release);
            NUM.store(0, Ordering::Release);
        });
    }
}
