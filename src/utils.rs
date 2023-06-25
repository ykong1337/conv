use std::{env, fs::File, io::{BufRead, BufReader, Write}};
use std::path::{Path, PathBuf};

fn is_number(s: &str) -> bool {
    s.chars().all(|c| c.is_digit(10))
}

// 将srt时间格式转换为lrc时间格式
fn format_time(time: &str) -> String {
    let msec = &time[9..12];
    let sec = &time[6..8];
    let min = &time[3..5];

    format!("[{}:{}.{:.2}]", min, sec, msec)
}

fn srt2lrc(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output_filename = Path::new(path).with_extension("lrc");

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let lines = reader.lines().filter_map(|l| l.ok());

    let lrc_content = lines.fold(String::new(), |lrc, line| {
        if is_number(&line) {
            // 行号
            lrc
        } else if line.is_empty() {
            // 空行
            lrc
        } else {
            // 时间和文本内容
            let time = line.split_whitespace().nth(0).unwrap();
            let text = line.splitn(2, char::is_whitespace).nth(1).unwrap();

            let lrc_time = format_time(time);
            format!("{}{}\n", lrc, lrc_time + " " + text)
        }
    });

    let mut output_file = File::create(output_filename)?;
    output_file.write_all(lrc_content.as_bytes())?;

    Ok(())
}
