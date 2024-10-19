use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use crate::font::load_fonts;
use crate::utils::{merge, MERGE};
use crossbeam::atomic::AtomicCell;
use eframe::CreationContext;
use egui::FontFamily::Proportional;
use egui::FontId;
use egui::TextStyle::{Body, Button, Heading, Monospace, Name, Small};
use parking_lot::RwLock;

pub static COUNT: AtomicCell<usize> = AtomicCell::new(0);
pub static ALL: AtomicCell<usize> = AtomicCell::new(0);
pub static CURRENT_DIR: LazyLock<PathBuf> = LazyLock::new(|| std::env::current_dir().unwrap());

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
            files.write().image = rfd::FileDialog::new()
                .add_filter("Image File", &["png"])
                .pick_file();
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
        let files = self.files.clone();

        tokio::spawn(async move {
            let files = files;
            let files_lock = files.read();
            let resources = files_lock.audio.iter().zip(files_lock.subtitle.iter());
            MERGE.store(true);
            ALL.store(resources.len());

            if let Some(ref image) = files_lock.image {
                for (audio, subtitle) in resources {
                    COUNT.fetch_add(1);
                    let subtitle_cache = Path::new(&nanoid::nanoid!())
                        .with_extension(subtitle.extension().unwrap_or_default());
                    if !CURRENT_DIR.join(&subtitle_cache).exists() {
                        std::fs::copy(subtitle, CURRENT_DIR.join(&subtitle_cache)).ok();
                    }
                    let output = audio.with_extension("mp4");

                    merge(
                        audio.to_str().unwrap_or_default(),
                        image.to_str().unwrap_or_default(),
                        subtitle_cache.to_str().unwrap_or_default(),
                        output.to_str().unwrap_or_default(),
                    )
                    .ok();
                    if !MERGE.load() {
                        break;
                    }
                    std::fs::remove_file(subtitle_cache).ok();
                    std::fs::remove_file("out.mp4").ok();
                }
            }
            MERGE.store(false);
            COUNT.store(0);
            ALL.store(0);
        });
    }
}
