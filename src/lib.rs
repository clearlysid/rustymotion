mod composition;
mod ffmpeg;

use composition::Composition;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png;
use headless_chrome::{Browser, LaunchOptions};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{error::Error, fs, io, thread};

#[derive(Debug)]
pub struct RenderOptions {
    pub bundle: String,
    pub output: String,
    pub composition: String,
    pub props: Option<String>,
    pub frames: Option<(u32, u32)>,
}

// Trim double quotes and unescape string
fn unescape_json_string(escaped_json: &str) -> Result<String, serde_json::Error> {
    let trimmed = escaped_json.trim_matches('"');
    let unescaped = serde_json::from_str::<String>(&format!("\"{}\"", trimmed))?;
    Ok(unescaped)
}

fn get_render_comp(
    comp: &String,
    html: &String,
    js: &String,
) -> Result<Composition, Box<dyn Error>> {
    let browser = Browser::default()?;
    let tab = browser.new_tab()?;

    let mode_script = "window.remotion_setBundleMode({ type: 'evaluation' });";
    let comp_script = format!(
        "window.getStaticCompositions().then(cs => cs.find(c => c.id === '{}')).then(c => JSON.stringify(c))",
        comp
    );

    tab.navigate_to(&html)?;
    tab.evaluate(&js, true)?;
    tab.evaluate(mode_script, false)?;

    let comp_raw = tab.evaluate(&comp_script, true)?.value.unwrap().to_string();
    let comp_clean = unescape_json_string(&comp_raw)?;

    Ok(composition::derive_composition(comp_clean))
}

fn read_file_to_string(path: PathBuf) -> io::Result<String> {
    if !path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "File not found"));
    }

    if !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Path is not a file",
        ));
    }

    fs::read_to_string(path)
}

pub fn render(options: RenderOptions) -> Result<(), Box<dyn Error>> {
    let now = Instant::now();

    println!("✅ Rendering composition: {}", options.composition);

    // 1. Validate bundle and options
    let bundle_path = PathBuf::from(&options.bundle)
        .canonicalize()
        .expect("Bundle folder does not exist.");

    let output_file = options.output;
    let composition = options.composition;
    let frame_start = options.frames.unwrap_or((0, 0)).0;
    let frame_end = options.frames.unwrap_or((0, 0)).1;

    println!("✅ Renderer options processed!");

    // 2. Read files and get render composition details
    let html_path = bundle_path.join("index.html");
    let html_url = format!("file://{}", html_path.to_str().unwrap());
    let js_path = bundle_path.join("bundle.js");
    let js_str = match read_file_to_string(js_path) {
        Ok(content) => content,
        Err(e) => return Err(Box::new(e)),
    };
    let comp = match get_render_comp(&composition, &html_url, &js_str) {
        Ok(comp) => comp,
        Err(e) => return Err(e),
    };

    // 3. Render composition
    println!("✅ Capturing Frames!");

    let fps = comp.fps;
    let (width, height) = (comp.width, comp.height);
    let frame_duration = match frame_end {
        0 => comp.duration_in_frames,
        _ => frame_end,
    };
    let frame_dir = Arc::new(Mutex::new(PathBuf::from("./frames")));

    // Get CPU Cores and calculate frames per thread
    let num_threads = num_cpus::get() as u32;
    let frames_per_thread = frame_duration / num_threads as u32;
    let mut handles = Vec::new(); // To store thread handles

    // 3. Prepare composition for rendering
    let comp_script = format!(
        "window.remotion_setBundleMode({{
                    type: 'composition',
                    compositionName: '{}',
                    serializedResolvedPropsWithSchema: '{}',
                    compositionDurationInFrames: {},
                    compositionFps: {},
                    compositionHeight: {},
                    compositionWidth: {},
                }});",
        comp.id,
        comp.serialized_default_props_with_custom_schema,
        comp.duration_in_frames,
        comp.fps,
        comp.height,
        comp.width
    );

    // Spawn threads
    for i in 0..num_threads {
        let thread_comp = comp.id.clone();
        let thread_comp_script = comp_script.clone();
        let thread_js_str = js_str.clone();
        let thread_html_url = html_url.clone();
        // let thread_sender = sender.clone();
        let frame_dir = frame_dir.clone();

        // Calculate the frame range for this thread
        let start_frame = i * frames_per_thread + frame_start;
        let end_frame = if i == num_threads - 1 {
            frame_duration
        } else {
            (i + 1) * frames_per_thread
        };

        let handle = thread::spawn(move || {
            // 1. For each thread, spawn a browser instance
            let browser = Browser::new(LaunchOptions {
                headless: true,
                enable_gpu: true,
                window_size: Some((width, height)),
                ..Default::default()
            })
            .expect("Failed to launch browser");

            // 2. Open tab and navigate to index.html
            let tab = browser.new_tab().expect("Failed to create tab");
            tab.navigate_to(&thread_html_url)
                .expect("Failed to navigate to index.html");
            tab.evaluate(&thread_js_str, true)
                .expect("failed to evaluate bundle.js");
            tab.evaluate(&thread_comp_script, true)
                .expect("Failed to prepare composition");

            // Loop over frames and capture screenshots
            for frame in start_frame..end_frame {
                // Updates bundle app to current frame
                let set_frame_script =
                    format!("window.remotion_setFrame({}, '{}');", frame, thread_comp);
                tab.evaluate(&set_frame_script.as_str(), true)
                    .expect("Failed to set frame");

                println!("Capturing frame: {}", frame);

                // take screenshot
                let png_data = tab
                    .capture_screenshot(Png, None, None, true)
                    .expect("couldn't capture screenshot");
                println!("PNG Data: {:?}", &png_data[0..5]);

                // write ss to file
                let name = format!("frame-{}.png", frame);
                let path = frame_dir.lock().unwrap().join(name);
                std::fs::write(path, png_data).expect("couldn't write file");
            }
        });

        handles.push(handle); // Store the handle
    }

    // Collect all the output handles from threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    println!("✅ Frames captured.");

    // Encode into video
    ffmpeg::encode_video(&output_file, fps, &frame_dir.lock().unwrap())?;
    println!("✅ Video encoded.");

    let elapsed_time = now.elapsed();
    println!("⏰ Rendering took {}s", elapsed_time.as_secs_f64());

    return Ok(());
}
