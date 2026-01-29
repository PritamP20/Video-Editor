use anyhow::{Context, Result, anyhow};
use std::path::Path;
use std::process::Command;

pub fn combine_videos(inputs: &[std::path::PathBuf], output: &Path) -> Result<()> {
    println!("Combining videos using ffmpeg filter...");

    let mut command = Command::new("ffmpeg");

    for input in inputs {
        command.arg("-i").arg(input);
    }

    let mut filter = String::new();
    for i in 0..inputs.len() {
        use std::fmt::Write;
        write!(filter, "[{}:v][{}:a]", i, i).unwrap();
    }
    use std::fmt::Write;
    write!(filter, "concat=n={}:v=1:a=1[outv][outa]", inputs.len()).unwrap();

    let status = command
        .arg("-filter_complex")
        .arg(&filter)
        .arg("-map")
        .arg("[outv]")
        .arg("-map")
        .arg("[outa]")
        .arg("-y")
        .arg(output)
        .status()
        .context("Failed to execute ffmpeg")?;

    if !status.success() {
        return Err(anyhow!("ffmpeg failed with status: {}", status));
    }

    Ok(())
}

pub fn compress_video(input: &Path, output: &Path, crf: u8) -> Result<()> {
    println!("Compressing video...");
    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(input)
        .arg("-vcodec")
        .arg("libx264")
        .arg("-crf")
        .arg(crf.to_string())
        .arg("-y")
        .arg(output)
        .status()
        .context("Failed to execute ffmpeg")?;

    if !status.success() {
        return Err(anyhow!("ffmpeg failed with status: {}", status));
    }
    Ok(())
}

pub fn add_music(video: &Path, audio: &Path, output: &Path, reduce_original: &str) -> Result<()> {
    println!("Adding music...");

    let has_audio = probe_has_audio(video)?;

    let status = if has_audio {
        let filter = format!(
            "[0:a]volume={}[a0];[1:a]volume=1.0[a1];[a0][a1]amix=inputs=2:duration=first[out]",
            reduce_original
        );
        Command::new("ffmpeg")
            .arg("-i")
            .arg(video)
            .arg("-i")
            .arg(audio)
            .arg("-filter_complex")
            .arg(&filter)
            .arg("-map")
            .arg("0:v")
            .arg("-map")
            .arg("[out]")
            .arg("-y")
            .arg(output)
            .status()
            .context("Failed to execute ffmpeg")?
    } else {
        Command::new("ffmpeg")
            .arg("-i")
            .arg(video)
            .arg("-i")
            .arg(audio)
            .arg("-map")
            .arg("0:v")
            .arg("-map")
            .arg("1:a")
            .arg("-shortest")
            .arg("-y")
            .arg(output)
            .status()
            .context("Failed to execute ffmpeg")?
    };

    if !status.success() {
        return Err(anyhow!("ffmpeg failed with status: {}", status));
    }
    Ok(())
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

pub fn get_info(input: &Path) -> Result<()> {
    let status = Command::new("ffprobe")
        .arg("-hide_banner")
        .arg("-i")
        .arg(input)
        .status()
        .context("Failed to execute ffprobe")?;

    if !status.success() {
        return Err(anyhow!("ffprobe failed"));
    }
    Ok(())
}

pub fn timelapse(input: &Path, output: &Path, speed: f64) -> Result<()> {
    println!("Creating timelapse...");

    let filter = format!("setpts=PTS/{}", speed);

    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(input)
        .arg("-filter:v")
        .arg(&filter)
        .arg("-an")
        .arg("-y")
        .arg(output)
        .status()
        .context("Failed to execute ffmpeg")?;

    if !status.success() {
        return Err(anyhow!("ffmpeg failed with status: {}", status));
    }
    Ok(())
}
