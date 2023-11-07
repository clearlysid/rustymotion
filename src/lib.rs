use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png;
use headless_chrome::{Browser, LaunchOptions};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

mod composition;
mod ffmpeg;

#[derive(Debug)]
pub struct RenderOptions {
    pub bundle: String,
    pub output: String,
    pub composition: String,
    pub props: Option<String>,
    pub frames: Option<(u32, u32)>,
}

fn unescape_json_string(escaped_json: &str) -> Result<String, serde_json::Error> {
    // Trim the leading and trailing double quotes, then unescape the string
    let trimmed = escaped_json.trim_matches('"');
    let unescaped = serde_json::from_str::<String>(&format!("\"{}\"", trimmed))?;
    Ok(unescaped)
}

pub fn render(options: RenderOptions) -> Result<(), Box<dyn Error>> {
    // 1. Validate bundle and composition, deconstruct frames and options
    println!("Rendering with options: {:?}", options);
    let bundle_path = PathBuf::from(&options.bundle)
        .canonicalize()
        .expect("Bundle folder does not exist.");

    let output_file = options.output;
    let composition = options.composition;
    let frame_start = options.frames.unwrap_or((0, 0)).0;
    let frame_end = options.frames.unwrap_or((0, 0)).1;

    // 2. Create Browser and tab
    let browser = Browser::new(LaunchOptions {
        headless: true,
        enable_gpu: true,
        window_size: Some((1080, 1080)),
        ..Default::default()
    })?;
    let tab = browser.new_tab()?;

    // 3. Load HTML and JS
    let bundle_index_path = bundle_path.join("index.html");
    let bundle_index_str = bundle_index_path.to_str().unwrap();
    let bundle_index_url = format!("file://{}", bundle_index_str);

    tab.navigate_to(&bundle_index_url)?;

    let bundle_js_path = bundle_path.join("bundle.js");
    let bundle_js_str = fs::read_to_string(bundle_js_path)?;

    tab.evaluate(&bundle_js_str, true)?;

    // 4. Get data about the Remotion bundle
    tab.evaluate(
        "window.remotion_setBundleMode({ type: 'evaluation' });",
        false,
    )?;

    // 5. Get composition data
    let comps_raw = tab
        .evaluate(
            "window.getStaticCompositions().then(comps => JSON.stringify(comps))",
            true,
        )?
        .value
        .unwrap()
        .to_string();

    let comps_sanitized = unescape_json_string(&comps_raw)?;
    let comps = composition::derive(comps_sanitized);

    let comp = comps
        .iter()
        .find(|c| c.id == composition)
        .expect("No matching Composition found");

    // 6. Set up Frame Loop
    let fps = comp.fps;
    let frame_duration = match frame_end {
        0 => comp.duration_in_frames,
        _ => frame_end,
    };

    // let width = comp.width;
    // let height = comp.height;

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
        let set_frame_script = format!("window.remotion_setFrame({}, '{}');", frame, composition);
        tab.evaluate(&set_frame_script.as_str(), true)?;

        // take screenshot
        let png_data = tab.capture_screenshot(Png, None, None, true)?;
        println!("PNG Data: {:?}", &png_data[0..5]);

        // let name = format!("frame-{}.png", frame);
        // let path = frame_dir.join(name);
        // std::fs::write(path, png_data)?;
    }

    println!("Frames captured.");
    tab.close(true).expect("Failed to close tab");

    // 9. Encode into video
    ffmpeg::encode_video(&output_file, fps, frame_dir)?;
    println!("Video encoded.");
    return Ok(());
}
