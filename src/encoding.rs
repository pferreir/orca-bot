use std::{
    path::Path,
    process::Command,
};

use anyhow::{anyhow, Context, Result};

pub fn encode<ScreenDir: AsRef<Path>, AudioFile: AsRef<Path>, OutFile: AsRef<Path>>(
    screen_dir: ScreenDir,
    (width, height): (u16, u16),
    audio_file: AudioFile,
    out_file: OutFile,
) -> Result<()> {
    let out = Command::new("ffmpeg")
        .args([
            "-f",
            "image2",
            "-pixel_format",
            "rgba",
            "-video_size",
            &format!("{width}x{height}"),
            "-framerate",
            "60",
            "-c:v",
            "rawvideo",
            "-i",
            screen_dir
                .as_ref()
                .join("out_%05d.rgba")
                .to_str()
                .context("Invalid dir name")?,
            "-ac",
            "2",
            "-ar",
            "44100",
            "-f",
            "f32le",
            "-i",
            audio_file
                .as_ref()
                .to_str()
                .context("Invalid file name")?,
            "-c:v",
            "libx264",
            "-c:a",
            "aac",
            "-y",
            out_file.as_ref().to_str().context("Invalid file name")?,
        ])
        .output()
        .context("Error running FFmpeg")?;

    if out.status.success() {
        log::debug!("{}", &String::from_utf8_lossy(&out.stderr));
        Ok(())
    } else {
        log::error!("{}", &String::from_utf8_lossy(&out.stderr));
        Err(anyhow!("Command returned an error"))
    }

}
