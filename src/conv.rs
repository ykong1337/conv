use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use eframe::CreationContext;
use egui::FontFamily::Proportional;
use egui::FontId;
use egui::TextStyle::{Body, Button, Heading, Monospace, Name, Small};
use tokio::runtime::Runtime;
use crate::config::{Language, Model};
use crate::font::load_fonts;
use crate::utils::{MERGE, merge, WHISPER};
use crate::whisper::Whisper;

#[derive(Clone)]
pub struct Conv {
    pub rt: Arc<Runtime>,
    pub files: Arc<Mutex<Files>>,
    pub config: Config,
}

#[derive(Clone)]
pub struct Config {
    pub lang: Language,
    pub model: Model,
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

        Box::new(Self {
            rt: Arc::new(Runtime::new().unwrap()),
            files: Default::default(),
            config: Config { lang: Language::Auto, model: Model::Medium },
        })
    }

    pub fn open_audio(&self, files: Arc<Mutex<Files>>) {
        self.rt.spawn(async move {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Audio File", &["mp3", "wav"])
                .pick_file() {
                files.lock().unwrap().audio = Some(path);
            }
        });
    }

    pub fn open_image(&self, files: Arc<Mutex<Files>>) {
        self.rt.spawn(async move {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Image File", &["jpg", "png"])
                .pick_file() {
                files.lock().unwrap().image = Some(path);
            }
        });
    }

    pub fn open_subtitle(&self, files: Arc<Mutex<Files>>) {
        self.rt.spawn(async move {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Subtitle File", &["srt", "lrc", "vtt"])
                .pick_file() {
                files.lock().unwrap().subtitle = Some(path);
            }
        });
    }

    pub fn whisper(&self) {
        let file = self.files.lock().unwrap();
        let audio = file.audio.clone();
        let model = self.config.model;
        let lang = self.config.lang;
        self.rt.spawn(async move {
            if let Some(ref audio) = audio {
                if let Ok(ref mut w) = Whisper::new(lang, model).await {
                    WHISPER.store(true, Ordering::Relaxed);
                    if let Ok(ref t) = w.transcribe(audio, false, false) {
                        let lrc = t.as_lrc();
                        let path_lrc = audio.with_extension("lrc");
                        let mut file = File::create(path_lrc).unwrap();
                        file.write_all(lrc.as_bytes()).unwrap();
                    }
                }
            }

            WHISPER.store(false, Ordering::Relaxed);
        });
    }

    pub fn ffmpeg_merge(&self) {
        let file = self.files.lock().unwrap();
        let image = file.image.clone();
        let audio = file.audio.clone();
        let subtitle = file.subtitle.clone();
        self.rt.spawn(async move {
            MERGE.store(true, Ordering::Relaxed);
            if let (Some(ref image), Some(ref audio), Some(ref subtitle)) = (image, audio, subtitle) {
                let current = std::env::current_dir().unwrap();
                let subtitle_cache = Path::new(&uuid::Uuid::new_v4().to_string()).with_extension(subtitle.extension().unwrap());
                if !current.join(&subtitle_cache).exists() {
                    std::fs::copy(subtitle, current.join(&subtitle_cache)).unwrap();
                }
                let output = audio.with_extension("mp4");

                if let Ok(child) = merge(
                    audio.to_str().unwrap(),
                    image.to_str().unwrap(),
                    subtitle_cache.to_str().unwrap(),
                    output.to_str().unwrap(),
                ).as_mut() {
                    if child.wait().is_err() {
                        MERGE.store(false, Ordering::Relaxed);
                        return;
                    }
                } else {
                    MERGE.store(false, Ordering::Relaxed);
                    return;
                }
                if std::fs::remove_file(current.join(subtitle_cache)).is_err() {
                    MERGE.store(false, Ordering::Relaxed);
                    return;
                }
            } else {
                MERGE.store(false, Ordering::Relaxed);
                return;
            }

            MERGE.store(false, Ordering::Relaxed);
        });
    }
}
