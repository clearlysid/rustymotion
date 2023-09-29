use std::io::{Error, ErrorKind};
use std::path::Path;
use std::process::Command;

pub fn encode_video(output_file: &str, fps: &str, frame_dir: &Path) -> Result<String, Error> {
    // check if frame dir exists
    if !frame_dir.exists() {
        return Err(Error::new(
            ErrorKind::Other,
            "Frame directory does not exist",
        ));
    }

    // check if frame dir is empty
    if frame_dir.read_dir()?.next().is_none() {
        return Err(Error::new(ErrorKind::Other, "Frame directory is empty"));
    }

    // create frames string
    let frames = format!("{}/frame-%d.png", frame_dir.display());

    // run ffmpeg
    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-framerate")
        .arg(fps) // 30 frames in 1 second
        .arg("-i")
        .arg(frames) // Input file pattern
        .arg("-c:v")
        .arg("libx264") // Use the x264 codec for video
        .arg("-vf")
        .arg("fps=30") // Ensure the output video has 30 fps
        .arg("-pix_fmt")
        .arg("yuv420p") // Pixel format required for many players
        .arg(&output_file) // Output file name
        .output()?;

    if !output.status.success() {
        eprintln!("Failed to run ffmpeg: {:?}", output);
        return Err(Error::new(ErrorKind::Other, "ffmpeg process failed"));
    }

    Ok(output_file.to_string())
}
