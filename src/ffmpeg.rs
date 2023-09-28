use std::io::{Error, ErrorKind};
use std::process::Command;
use std::thread::sleep;

pub fn render_screenshots_to_video(output_file: String) -> Result<(), Error> {
    // wait for 2 seconds
    sleep(std::time::Duration::from_secs(1));

    let output = Command::new("ffmpeg")
        .arg("-framerate")
        .arg("30") // 30 frames in 1 second
        .arg("-i")
        .arg("frame-%d.png") // Input file pattern
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

    println!("Successfully created {}", output_file);
    Ok(())
}
