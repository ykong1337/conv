use std::{env, fs, io, thread};
use std::io::Error;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

#[derive(Debug, Clone, Default)]
struct Files {
    audio: Option<PathBuf>,
    image: Option<PathBuf>,
    subtitle: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct Conv {
    rt: Arc<Runtime>,
    files: Arc<Mutex<Files>>,
}

impl Conv {
    pub fn new() -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        Self {
            rt: Arc::new(rt),
            files: Arc::new(Mutex::new(Files::default())),
        }
    }

    fn open_audio(&self) {
        let mut audio = self.files.clone();
        // thread::spawn(move || {
        if let Some(path) = rfd::FileDialog::new()
            .pick_file() {
            audio.lock().unwrap().audio = Some(path);
        }
        // });
    }

    fn open_image(&self) {
        let mut image = self.files.clone();
        thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .pick_file() {
                image.lock().unwrap().image = Some(path);
            }
        });
    }

    fn open_subtitle(&self) {
        let mut subtitle = self.files.clone();
        thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .pick_file() {
                subtitle.lock().unwrap().subtitle = Some(path);
            }
        });
    }
}

fn main() -> io::Result<()> {
    let current = env::current_dir()?;
    let mut con = Conv::new();
    con.rt.spawn(async move {
        println!("OK");
    });

    con.open_audio();

    let output = current.join("output.mp4");
    if output.exists() {
        fs::remove_file(output).unwrap_or(());
    }
    let ffmpeg = current.join("ffmpeg");
    let mut cmd = Command::new(&ffmpeg);

    if let Some(ref image) = con.files.lock().unwrap().image {
        cmd
            .arg("-loop")
            .arg("1")
            .arg("-i")
            .arg(image);
    }

    if let Some(ref audio) = con.files.lock().unwrap().audio {
        cmd
            .arg("-i")
            .arg(audio);
    } else {
        return Err(Error::from(ErrorKind::Other));
    }

    cmd
        .arg("-shortest")
        .arg("output.mp4");

    let exit_status = cmd
        .spawn()?
        .wait()?;

    return match exit_status.success() {
        true => Ok(()),
        false => Err(Error::from(ErrorKind::Unsupported)),
    };
}
