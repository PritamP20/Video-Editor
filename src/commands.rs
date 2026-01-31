use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::io::{BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};

pub enum ProgressInfo {
    Log(String),
    Percentage(f64),
}

fn run_ffmpeg_with_progress<F>(mut command: Command, mut callback: F) -> Result<()>
where
    F: FnMut(ProgressInfo),
{
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command.spawn().context("Failed to start ffmpeg")?;

    // FFmpeg typically writes progress info to stderr
    if let Some(stderr) = child.stderr.take() {
        let mut reader = BufReader::new(stderr);
        // Match Duration: 00:00:00.00
        let duration_regex = Regex::new(r"Duration: (\d+):(\d+):(\d+(?:\.\d+)?)").unwrap();
        // Match time=00:00:00.00
        let time_regex = Regex::new(r"time=(\d+):(\d+):(\d+(?:\.\d+)?)").unwrap();

        let mut total_duration_secs = 0.0;
        let mut buf = Vec::new();
        let mut byte = [0u8; 1];

        loop {
            match reader.read(&mut byte) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let b = byte[0];
                    if b == b'\n' || b == b'\r' {
                        if !buf.is_empty() {
                            let line = String::from_utf8_lossy(&buf).to_string();

                            // Only log if it's a significant line or every N lines to avoid spam?
                            // For now, logging everything might fill the TUI logs too fast if only \r updates.
                            // But original code logged everything.
                            // To improve TUI responsiveness, maybe filter "time=" lines from Logs?
                            // The original code: callback(ProgressInfo::Log(line.clone()));

                            if !line.starts_with("frame=") {
                                callback(ProgressInfo::Log(line.clone()));
                            }

                            if let Some(caps) = duration_regex.captures(&line) {
                                let h: f64 = caps[1].parse().unwrap_or(0.0);
                                let m: f64 = caps[2].parse().unwrap_or(0.0);
                                let s: f64 = caps[3].parse().unwrap_or(0.0);
                                total_duration_secs = h * 3600.0 + m * 60.0 + s;
                            }

                            if total_duration_secs > 0.0 {
                                if let Some(caps) = time_regex.captures(&line) {
                                    let h: f64 = caps[1].parse().unwrap_or(0.0);
                                    let m: f64 = caps[2].parse().unwrap_or(0.0);
                                    let s: f64 = caps[3].parse().unwrap_or(0.0);
                                    let current_secs = h * 3600.0 + m * 60.0 + s;

                                    let percentage = (current_secs / total_duration_secs).min(1.0);
                                    callback(ProgressInfo::Percentage(percentage));
                                }
                            }

                            buf.clear();
                        }
                    } else {
                        buf.push(b);
                    }
                }
                Err(_) => break,
            }
        }
    }

    let status = child.wait()?;

    if !status.success() {
        return Err(anyhow!("ffmpeg failed with status: {}", status));
    }

    callback(ProgressInfo::Percentage(1.0));
    Ok(())
}

pub fn combine_videos<F>(
    inputs: &[std::path::PathBuf],
    output: &Path,
    mut callback: F,
) -> Result<()>
where
    F: FnMut(ProgressInfo),
{
    callback(ProgressInfo::Log(
        "Combining videos using ffmpeg filter...".to_string(),
    ));

    let has_audio = if let Some(first) = inputs.first() {
        probe_has_audio(first)?
    } else {
        return Err(anyhow!("No input files provided"));
    };

    let mut command = Command::new("ffmpeg");

    for input in inputs {
        command.arg("-i").arg(input);
    }

    let mut filter = String::new();
    for i in 0..inputs.len() {
        use std::fmt::Write;
        if has_audio {
            write!(filter, "[{}:v][{}:a]", i, i).unwrap();
        } else {
            write!(filter, "[{}:v]", i).unwrap();
        }
    }
    use std::fmt::Write;
    if has_audio {
        write!(filter, "concat=n={}:v=1:a=1[outv][outa]", inputs.len()).unwrap();
    } else {
        write!(filter, "concat=n={}:v=1:a=0[outv]", inputs.len()).unwrap();
    }

    command
        .arg("-filter_complex")
        .arg(&filter)
        .arg("-map")
        .arg("[outv]");

    if has_audio {
        command.arg("-map").arg("[outa]");
    }

    command.arg("-y").arg(output);

    run_ffmpeg_with_progress(command, callback)
}

pub fn compress_video<F>(input: &Path, output: &Path, crf: u8, mut callback: F) -> Result<()>
where
    F: FnMut(ProgressInfo),
{
    callback(ProgressInfo::Log("Compressing video...".to_string()));
    let mut command = Command::new("ffmpeg");
    command
        .arg("-i")
        .arg(input)
        .arg("-vcodec")
        .arg("libx264")
        .arg("-crf")
        .arg(crf.to_string())
        .arg("-y")
        .arg(output);

    run_ffmpeg_with_progress(command, callback)
}

pub fn add_music<F>(
    video: &Path,
    audio: &Path,
    output: &Path,
    reduce_original: &str,
    mut callback: F,
) -> Result<()>
where
    F: FnMut(ProgressInfo),
{
    callback(ProgressInfo::Log("Adding music...".to_string()));

    let has_audio = probe_has_audio(video)?;

    let mut command = Command::new("ffmpeg");
    if has_audio {
        let filter = format!(
            "[0:a]volume={}[a0];[1:a]volume=1.0[a1];[a0][a1]amix=inputs=2:duration=first[out]",
            reduce_original
        );
        command
            .arg("-i")
            .arg(video)
            .arg("-i")
            .arg(audio)
            .arg("-filter_complex")
            .arg(&filter)
            .arg("-map")
            .arg("0:v")
            .arg("-map")
            .arg("[out]");
    } else {
        command
            .arg("-i")
            .arg(video)
            .arg("-i")
            .arg(audio)
            .arg("-map")
            .arg("0:v")
            .arg("-map")
            .arg("1:a")
            .arg("-shortest");
    };

    command.arg("-y").arg(output);

    run_ffmpeg_with_progress(command, callback)
}

fn probe_has_audio(path: &Path) -> Result<bool> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-select_streams")
        .arg("a")
        .arg("-show_entries")
        .arg("stream=codec_type")
        .arg("-of")
        .arg("csv=p=0")
        .arg(path)
        .output()
        .context("Failed to run ffprobe")?;

    if !output.status.success() {
        return Err(anyhow!("ffprobe failed"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(!stdout.trim().is_empty())
}

pub fn get_info<F>(input: &Path, mut callback: F) -> Result<()>
where
    F: FnMut(ProgressInfo),
{
    let output = Command::new("ffprobe")
        .arg("-hide_banner")
        .arg("-i")
        .arg(input)
        .output()
        .context("Failed to execute ffprobe")?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    callback(ProgressInfo::Log(stderr.to_string()));

    if !output.status.success() {
        return Err(anyhow!("ffprobe failed"));
    }
    Ok(())
}

pub fn timelapse<F>(input: &Path, output: &Path, speed: f64, mut callback: F) -> Result<()>
where
    F: FnMut(ProgressInfo),
{
    callback(ProgressInfo::Log("Creating timelapse...".to_string()));

    let filter = format!("setpts=PTS/{}", speed);

    let mut command = Command::new("ffmpeg");
    command
        .arg("-i")
        .arg(input)
        .arg("-filter:v")
        .arg(&filter)
        .arg("-an")
        .arg("-y")
        .arg(output);

    run_ffmpeg_with_progress(command, callback)
}
