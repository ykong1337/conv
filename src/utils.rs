use std::{fs, fs::File, io::Write, thread};
use std::path::PathBuf;
use std::process::Command;
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
                    "[{:02}:{:02}.{:02}]{}\n",
                    fragment.start / 100 / 60,
                    fragment.start / 100,
                    fragment.start % 100,
                    fragment.text
                )
                    .as_str()
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

pub fn ffmpeg_merge(audio: Option<PathBuf>, image: Option<PathBuf>, subtitle: Option<PathBuf>) {
    thread::spawn(move || {
        MERGE.store(true, Ordering::Relaxed);
        let mut cmd = Command::new("ffmpeg");
        if let Some(ref image) = image {
            cmd.args([
                "-loop",
                "1",
                "-i",
                image.to_str().unwrap(),
            ]);
        }
        if let (Some(ref audio), Some(ref subtitle)) = (audio, subtitle) {
            let output = audio.with_extension("mp4");
            if output.exists() {
                fs::remove_file(output).unwrap_or(());
            }
            cmd.args([
                "-i",
                audio.to_str().unwrap(),
                "-vf",
                &format!("subtitles={}", subtitle.file_name().unwrap().to_str().unwrap()),
                // "-c:v",
                // "copy",
                // "-c:a",
                // "copy",
                "-shortest",
                audio.with_extension("mp4").to_str().unwrap(),
            ]);
        } else {
            MERGE.store(false, Ordering::Relaxed);
            return;
        }
        if let Ok(mut c) = cmd.spawn() {
            if c.wait().is_err() {
                MERGE.store(false, Ordering::Relaxed);
                return;
            }
        }

        MERGE.store(false, Ordering::Relaxed);
    });
}