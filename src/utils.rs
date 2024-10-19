use std::process::{Child, Command};
use std::sync::atomic::AtomicBool;

pub static MERGE: AtomicBool = AtomicBool::new(false);

#[inline]
pub fn merge(audio: &str, image: &str, subtitle: &str, output: &str) -> std::io::Result<Child> {
    Command::new("ffmpeg")
        .args([
            "-y",
            "-loop",
            "1",
            "-r",
            "30",
            "-i",
            image,
            "-i",
            audio,
            "-c:v",
            "libx264",
            "-c:a",
            "aac",
            "-pix_fmt",
            "yuv420p",
            "-vf",
            "scale=-2:min(1080\\,trunc(ih/2)*2)",
            "-shortest",
            "out.mp4",
        ])
        .status()
        .ok();
    Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            "out.mp4",
            "-vf",
            &format!("subtitles={}", subtitle),
            output,
        ])
        .spawn()
}
