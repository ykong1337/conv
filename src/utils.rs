use std::{fs::File, io::Write};
use std::path::Path;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use whisper_cli::{Model, Transcript, Whisper};
use crate::ui::{Conv, Files};

pub static WHISPER: AtomicBool = AtomicBool::new(false);
pub static MERGE: AtomicBool = AtomicBool::new(false);

fn as_lrc(t: &Transcript) -> String {
    t.word_utterances
        .as_ref()
        .unwrap_or(&t.utterances)
        .iter()
        .fold(String::new(), |lrc, fragment| {
            lrc +
                format!(
                    "[{:02}:{:02}.{:02}]\n",
                    fragment.start / 100 / 60,
                    fragment.start / 100 % 60,
                    fragment.start % 100,
                ).as_str() +
                format!(
                    "[{:02}:{:02}.{:02}]{}\n",
                    fragment.start / 100 / 60,
                    fragment.start / 100 % 60,
                    fragment.start % 100,
                    fragment.text
                ).as_str()
        })
}


impl Conv {
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
        let size = self.config.size;
        let lang = self.config.lang;
        self.rt.spawn(async move {
            WHISPER.store(true, Ordering::Relaxed);
            if let Some(ref audio) = audio {
                let mut w = Whisper::new(Model::new(size), Some(lang)).await;
                if let Ok(ref t) = w.transcribe(audio, false, false) {
                    let lrc = as_lrc(t);
                    let srt = t.as_srt();
                    let path_lrc = audio.with_extension("lrc");
                    let path_srt = audio.with_extension("srt");
                    let mut file = File::create(path_lrc).unwrap();
                    file.write_all(lrc.as_bytes()).unwrap();
                    let mut file = File::create(path_srt).unwrap();
                    file.write_all(srt.as_bytes()).unwrap();
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
                let subtitle_cache = Path::new("temp").with_extension(subtitle.extension().unwrap());
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

#[inline]
fn merge(audio: &str, image: &str, subtitle: &str, output: &str) -> std::io::Result<Child> {
    Command::new("ffmpeg")
        .args([
            "-y",
            "-loop",
            "1",
            "-i",
            image,
            "-i",
            audio,
            "-vf",
            &format!("subtitles={}", subtitle),
            "-c:v",
            "libx264",
            "-c:a:1",
            "copy",
            "-c:a:2",
            "copy",
            "-c:a:3",
            "copy",
            "-pix_fmt",
            "yuv420p",
            "-shortest",
            output,
        ])
        .spawn()
}