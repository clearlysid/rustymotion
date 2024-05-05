mod composition;
mod utils;

use anyhow::Result;
use composition::Composition;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png;
use headless_chrome::{Browser, LaunchOptions};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
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

fn get_render_comp(comp: &String, url: &String) -> Result<Composition, Box<dyn Error>> {
    let browser = Browser::default()?;
    let tab = browser.new_tab()?;

    let mode_script = "window.remotion_setBundleMode({ type: 'evaluation' });";
    let comp_script = format!(
        "window.getStaticCompositions().then(cs => cs.find(c => c.id === '{}')).then(c => JSON.stringify(c))",
        comp
    );

    tab.navigate_to(&url)?;
    tab.wait_until_navigated()?;
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

#[actix_web::main]
async fn serve_remotion_bundle(bundle_path: PathBuf, port: u16) {
    // TODO: check for port availability
    // fallback to available port gracefully

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            // .wrap(Cors::default().send_wildcard())
            .service(actix_files::Files::new("/", &bundle_path).index_file("index.html"))
    })
    .bind(("127.0.0.1", port))
    .expect("Port blocked")
    .run()
    .await
    .expect("Could't start server");

    // TODO: band karna server
    //https://github.com/actix/examples/pull/240/files#diff-2d6d91cabd1e5f2349cb628a2d27fbe1e25babe2bb226fa8cc3f71be107d359e
}

pub fn render(options: RenderOptions) -> Result<(), Box<dyn Error>> {
    println!("✅ Rendering composition: {}", options.composition);

    // 1. Validate bundle and options
    let bundle_path = PathBuf::from(&options.bundle)
        .canonicalize()
        .expect("Bundle folder does not exist.");

    let output_file = options.output;
    let composition = options.composition;
    let (frame_start, frame_end) = options.frames.unwrap_or((0, 0));

    println!("✅ Renderer options processed!");

    // 2. Serve the bundle on actix server
    const PORT: u16 = 6543;
    let bundle_url = format!("http://127.0.0.1:{}/", PORT);
    thread::spawn(|| serve_remotion_bundle(bundle_path, PORT));
    println!("✅ Remotion Bundle served at {}", bundle_url);

    // 3. Get render composition details
    let render_comp = get_render_comp(&composition, &bundle_url);

    println!("✅ Render composition details fetched!");

    // 4. Render composition
    match render_comp {
        Ok(comp) => {
            println!("✅ Capturing Frames!");

            let fps = comp.fps;
            let width = comp.width;
            let height = comp.height;
            let frame_duration = match frame_end {
                0 => comp.duration_in_frames,
                _ => frame_end,
            };
            let frame_dir = Arc::new(Mutex::new(PathBuf::from("./frames")));

            // Get CPU Cores and calculate frames per thread
            let num_threads = num_cpus::get() as u32;
            let frames_per_thread = frame_duration / num_threads as u32;
            let mut handles = Vec::new(); // To store thread handles

            // Spawn threads
            for i in 0..num_threads {
                let thread_comp = comp.id.clone();
                let thread_comp_clone = comp.clone();
                let thread_bundle_index_url = bundle_url.clone();
                // let thread_sender = sender.clone();
                let frame_dir = frame_dir.clone();

                // Calculate the frame range for this thread
                let start_frame = i * frames_per_thread;
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
                    tab.navigate_to(&thread_bundle_index_url)
                        .expect("Failed to navigate to index.html");
                    tab.wait_until_navigated()
                        .expect("Failed to wait for navigation");

                    // 3. Prepare composition for rendering
                    let comp_prep_script = format!(
                        "window.remotion_setBundleMode({{
                            type: 'composition',
                            compositionName: '{}',
                            serializedResolvedPropsWithSchema: '{}',
                            compositionDurationInFrames: {},
                            compositionFps: {},
                            compositionHeight: {},
                            compositionWidth: {},
                        }});",
                        thread_comp_clone.id,
                        thread_comp_clone.serialized_default_props_with_custom_schema,
                        thread_comp_clone.duration_in_frames,
                        thread_comp_clone.fps,
                        thread_comp_clone.height,
                        thread_comp_clone.width
                    );
                    tab.evaluate(&comp_prep_script, true)
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
                        let name = format!("frame-{}.png", frame);
                        let path = frame_dir.lock().unwrap().join(name);
                        std::fs::write(path, png_data).expect("couldn't write file");
                        // println!("PNG Data: {:?}", &png_data[0..5]);
                    }
                });

                handles.push(handle); // Store the handle
            }

            // Collect all the output handles from threads
            for handle in handles {
                handle.join().expect("Thread panicked");
            }

            println!("Frames captured.");

            // Encode into video
            encoder::encode_video(&output_file, fps, &frame_dir.lock().unwrap())?;
            println!("Video encoded.");
            return Ok(());
        }
        Err(e) => Err(e),
    }
}
