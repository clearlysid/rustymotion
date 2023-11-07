mod composition;
mod ffmpeg;

use composition::Composition;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png;
use headless_chrome::{Browser, LaunchOptions};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

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

    tab.close(false)?;

    Ok(composition::derive_composition(comp_clean))
}

pub fn render(options: RenderOptions) -> Result<(), Box<dyn Error>> {
    // 1. Validate bundle and options
    println!("Rendering with options: {:?}", options);
    let bundle_path = PathBuf::from(&options.bundle)
        .canonicalize()
        .expect("Bundle folder does not exist.");

    let output_file = options.output;
    let composition = options.composition;
    let frame_start = options.frames.unwrap_or((0, 0)).0;
    let frame_end = options.frames.unwrap_or((0, 0)).1;

    // 2. Read files and get render composition details
    let bundle_index_path = bundle_path.join("index.html");
    let bundle_index_str = bundle_index_path.to_str().unwrap();
    let bundle_index_url = format!("file://{}", bundle_index_str);
    let bundle_js_path = bundle_path.join("bundle.js");
    let bundle_js_str = fs::read_to_string(bundle_js_path)?;
    let render_comp = get_render_comp(&composition, &bundle_index_url, &bundle_js_str);

    // 3. Render composition
    match render_comp {
        Ok(comp) => {
            let fps = comp.fps;
            let width = comp.width;
            let height = comp.height;
            let frame_duration = match frame_end {
                0 => comp.duration_in_frames,
                _ => frame_end,
            };

            // 2. Create Browser and tab
            let browser = Browser::new(LaunchOptions {
                headless: true,
                enable_gpu: true,
                window_size: Some((width, height)),
                ..Default::default()
            })?;
            let tab = browser.new_tab()?;

            tab.navigate_to(&bundle_index_url)?;
            tab.evaluate(&bundle_js_str, true)?;

            // TODO: write this into a temp directory
            let frame_dir = Path::new("./frames");
            if !frame_dir.exists() {
                fs::create_dir(frame_dir).expect("Failed to create frame directory");
            }

            // 7. Prepare composition for rendering
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
                comp.id,
                comp.serialized_default_props_with_custom_schema,
                comp.duration_in_frames,
                comp.fps,
                comp.height,
                comp.width
            );
            tab.evaluate(&comp_prep_script, true)?;

            // sleep for 2 seconds main thread
            std::thread::sleep(std::time::Duration::from_secs(1));

            // 8. Capture Frames
            for frame in frame_start..frame_duration {
                println!("Writing Frame: {}", frame);

                // Updates bundle app to current frame
                let set_frame_script =
                    format!("window.remotion_setFrame({}, '{}');", frame, composition);
                tab.evaluate(&set_frame_script.as_str(), true)?;

                // take screenshot
                let png_data = tab.capture_screenshot(Png, None, None, true)?;
                println!("PNG Data: {:?}", &png_data[0..5]);

                let name = format!("frame-{}.png", frame);
                let path = frame_dir.join(name);
                std::fs::write(path, png_data)?;
            }

            println!("Frames captured.");
            tab.close(true).expect("Failed to close tab");

            // 9. Encode into video
            ffmpeg::encode_video(&output_file, fps, frame_dir)?;
            println!("Video encoded.");
            return Ok(());
        }
        Err(e) => Err(e),
    }
}
