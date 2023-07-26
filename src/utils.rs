use std::{fs::File, io::Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::runtime::Runtime;
use whisper_cli::{Language, Model, Size, Transcript, Whisper};

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

pub fn whisper(rt: Arc<Runtime>, path: PathBuf, lang: Language, size: Size) {
    rt.spawn(async move {
        WHISPER.store(true, Ordering::Relaxed);
        let mut w = Whisper::new(Model::new(size), Some(lang)).await;
        if let Ok(ref t) = w.transcribe(&path, false, false) {
            let lrc = as_lrc(t);
            let srt = t.as_srt();
            let path_lrc = path.with_extension("lrc");
            let path_srt = path.with_extension("srt");
            let mut file = File::create(path_lrc).unwrap();
            file.write_all(lrc.as_bytes()).unwrap();
            let mut file = File::create(path_srt).unwrap();
            file.write_all(srt.as_bytes()).unwrap();
        }
        WHISPER.store(false, Ordering::Relaxed);
    });
}

pub fn ffmpeg_merge(rt: Arc<Runtime>, audio: Option<PathBuf>, image: Option<PathBuf>, subtitle: Option<PathBuf>) {
    rt.spawn(async move {
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