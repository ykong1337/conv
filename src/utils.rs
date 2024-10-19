use crossbeam::atomic::AtomicCell;
use std::process::Command;

pub static MERGE: AtomicCell<bool> = AtomicCell::new(false);

#[inline]
pub fn merge(audio: &str, image: &str, subtitle: &str, output: &str) -> anyhow::Result<()> {
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
            "-pix_fmt",
            "yuv420p",
            "-vf",
            "scale=-2:min(1080\\,trunc(ih/2)*2)",
            "-shortest",
            "out.mp4",
        ])
        .status()?;

    Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            "out.mp4",
            "-vf",
            &format!("subtitles={}", subtitle),
            output,
        ])
        .status()?;

    Ok(())
}
