use std::{env, io};
use std::io::{Error, ErrorKind, Write};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Default)]
struct Files {
    audio: Option<PathBuf>,
}

impl Files {
    fn open_audio(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .pick_file() {
            self.audio = Some(path);
        }
    }
}

fn main() -> io::Result<()> {
    let mut files = Files::default();
    files.open_audio();

    return if let Some(audio) = files.audio.as_ref() {
        let ffmpeg = env::current_dir()?
            .join("ffmpeg");
        let mut cmd = Command::new(&ffmpeg)
            .arg("-i")
            .arg(audio)

            .arg("-shortest")

            .arg("output.mp4")
            .spawn()?;

        let exit_status = cmd.wait()?;

        match exit_status.success() {
            true => {
                println!("{:?}", cmd);
                Ok(())
            },
            false => Err(Error::from(ErrorKind::Unsupported)),
        }
    } else {
        Err(Error::from(ErrorKind::Unsupported))
    };
}
