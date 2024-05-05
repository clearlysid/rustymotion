mod composition;
mod utils;

use actix_files::Files;
use actix_web::{App, HttpServer};
use anyhow::{Error, Result};
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png;
use headless_chrome::{Browser, LaunchOptions};
use henx::{VideoEncoder, VideoEncoderOptions};
use scap::frame::{BGRAFrame, Frame};
use std::{path::PathBuf, sync::mpsc, thread};

use composition::Composition;

const BUNDLE_URL_PORT: u16 = 6543;
const BUNDLE_URL_BASE: &str = "127.0.0.1";
const BUNDLE_URL: &str = "http://127.0.0.1:6543/";

#[derive(Debug)]
pub struct RenderOptions {
    pub bundle: String,
    pub output: String,
    pub composition: String,
    pub props: Option<String>,
    pub frames: Option<(u32, u32)>,
}

fn get_render_comp(comp: &String) -> Result<Composition, Error> {
    let browser = Browser::default()?;
    let tab = browser.new_tab()?;

    let mode_script = "window.remotion_setBundleMode({ type: 'evaluation' });";
    let comp_script = format!(
        "window.getStaticCompositions().then(cs => cs.find(c => c.id === '{}')).then(c => JSON.stringify(c))",
        comp
    );

    tab.navigate_to(BUNDLE_URL)?;
    tab.wait_until_navigated()?;
    tab.evaluate(mode_script, false)?;

    let comp_raw = tab.evaluate(&comp_script, true)?.value.unwrap().to_string();
    let comp_clean = utils::unescape_json_string(&comp_raw)?;

    Ok(composition::derive_composition(comp_clean))
}

#[actix_web::main]
async fn serve_remotion_bundle(bundle_path: PathBuf) {
    // TODO: check for port availability
    // fallback to available port gracefully

    HttpServer::new(move || {
        App::new().service(Files::new("/", &bundle_path).index_file("index.html"))
    })
    .bind((BUNDLE_URL_BASE, BUNDLE_URL_PORT))
    .expect("Port blocked")
    .run()
    .await
    .expect("Could't start server");
}

pub fn render(options: RenderOptions) -> Result<(), Error> {
    // 1. Destructure options
    let RenderOptions {
        bundle,
        output,
        composition,
        frames,
        ..
    } = options;

    // TODO: implement frame range in actual capturing
    let (_frame_start, frame_end) = frames.unwrap_or((0, 0));

    // 2. Serve Remotion bundle on an Actix server
    let bundle_path = PathBuf::from(&bundle).canonicalize()?;
    let bundle_server_thread = thread::spawn(|| serve_remotion_bundle(bundle_path));
    println!("✅ Remotion Bundle served at {}", BUNDLE_URL);

    // 3. Get render composition details
    let comp = get_render_comp(&composition)?;
    println!("✅ {} Composition data found!", composition);

    // 4. Render composition
    let Composition {
        width,
        height,
        duration_in_frames,
        ..
    } = comp;

    let frame_duration = match frame_end {
        0 => duration_in_frames,
        _ => frame_end,
    };

    // 5. Create encoder and message channel
    let (encoder_tx, encoder_rx) = mpsc::channel();
    let mut encoder = VideoEncoder::new(VideoEncoderOptions {
        width: width as usize,
        height: height as usize,
        path: output,
    });

    // 6. Create threads for capturing frames
    let capture_thread_count: u32 = 3;
    let frames_per_thread = frame_duration / capture_thread_count;
    let mut capture_thread_handles = Vec::new();

    for i in 0..capture_thread_count {
        let thread_composition = comp.clone();
        let thread_encoder_tx = encoder_tx.clone();

        // Calculate the frame range for this thread
        let start_frame = i * frames_per_thread;
        let end_frame = if i == capture_thread_count - 1 {
            frame_duration
        } else {
            (i + 1) * frames_per_thread
        };

        let handle = thread::spawn(move || {
            // 1. For each capture thread, spawn a headless browser
            let browser = Browser::new(LaunchOptions {
                headless: true,
                enable_gpu: true,
                window_size: Some((width, height)),
                ..Default::default()
            })
            .expect("Failed to launch browser");

            // 2. Open tab and navigate to Remotion bundle
            let tab = browser.new_tab().expect("Failed to create tab");
            tab.navigate_to(BUNDLE_URL)
                .expect("Failed to navigate to remotion bundle");
            tab.wait_until_navigated()
                .expect("Failed to wait for navigation");

            // 3. Prepare composition for screenshoting
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
                thread_composition.id,
                thread_composition.serialized_default_props_with_custom_schema,
                thread_composition.duration_in_frames,
                thread_composition.fps,
                thread_composition.height,
                thread_composition.width
            );
            tab.evaluate(&comp_prep_script, true)
                .expect("Failed to prepare composition");

            // Loop over frames and capture screenshots
            for frame in start_frame..end_frame {
                // Update webpage to current frame
                tab.evaluate(
                    format!(
                        "window.remotion_setFrame({}, '{}');",
                        frame, thread_composition.id
                    )
                    .as_str(),
                    true,
                )
                .expect("Failed to set frame");

                // take screenshot
                let png_data = tab
                    .capture_screenshot(Png, None, None, true)
                    .expect("couldn't capture screenshot");

                println!("Captured frame {}: {:?}", frame, &png_data[0..9]);

                let bgra_data =
                    utils::get_bgra_from_png(png_data).expect("Failed to get BGRA frame data");

                let display_time = utils::create_display_time(frame, thread_composition.fps);

                let final_frame = BGRAFrame {
                    display_time,
                    width: thread_composition.width as i32,
                    height: thread_composition.height as i32,
                    data: bgra_data,
                };

                // TODO: Create a Frame struct from png_data

                thread_encoder_tx
                    .send(final_frame) // Frame should be sent here instead
                    .expect("Failed to send frame");
            }

            drop(thread_encoder_tx); // Close the sender channel of this thread
        });

        capture_thread_handles.push(handle); // Store the handle
    }

    drop(encoder_tx); // Close the main sender channel

    // 7. Collect all the output handles from capture threads
    for handle in capture_thread_handles {
        handle.join().expect("Capture thread panicked");
    }

    // 8. Keep encoding until the ALL sending channels are dropped
    while let Ok(data) = encoder_rx.recv() {
        encoder
            .ingest_next_frame(&Frame::BGRA(data))
            .expect("failed to send frame");
    }

    // 9. Finish encoding
    drop(bundle_server_thread);
    match encoder.finish() {
        Ok(_) => println!("Encoding complete"),
        Err(e) => println!("Error: {:?}", e),
    }

    println!("✅ All done!");

    Ok(())
}
