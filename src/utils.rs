use std::process::{Child, Command};
use std::sync::atomic::AtomicBool;

pub static WHISPER: AtomicBool = AtomicBool::new(false);
pub static DOWNLOADING: AtomicBool = AtomicBool::new(false);
pub static MERGE: AtomicBool = AtomicBool::new(false);

#[inline]
pub fn merge(audio: &str, image: &str, subtitle: &str, output: &str) -> std::io::Result<Child> {
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