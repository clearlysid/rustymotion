use std::io::{Error, ErrorKind};
use std::path::Path;
use std::process::Command;

pub fn encode_video(output_file: &str, fps: u32, frame_dir: &Path) -> Result<String, Error> {
    // check if frame dir is empty
    if frame_dir.read_dir()?.next().is_none() {
        return Err(Error::new(ErrorKind::Other, "Frame directory is empty"));
    }

    let fps = format!("{}", fps);
    let frames = format!("{}/frame-%d.png", frame_dir.display());

    let output = Command::new("ffmpeg")
        .args(&[
            "-framerate",
            fps.as_str(),
            // "-i", "frames/frame-%d.png",
            "-i",
            frames.as_str(),
            "-c:v",
            "libx264",
            "-r",
            "30",
            "-pix_fmt",
            "yuv420p",
            "output.mp4",
        ])
        .output()?;

    if !output.status.success() {
        eprintln!("Failed to run ffmpeg: {:?}", output);
        return Err(Error::new(ErrorKind::Other, "ffmpeg process failed"));
    }

    Ok(output_file.to_string())
}
