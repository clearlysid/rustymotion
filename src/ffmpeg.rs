use std::io::{Error, ErrorKind};
use std::process::Command;

pub fn encode_video(output_file: &str) -> Result<String, Error> {
    let output = Command::new("ffmpeg")
        .arg("-framerate")
        .arg("30") // 30 frames in 1 second
        .arg("-i")
        .arg("./frames/frame-%d.png") // Input file pattern
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
