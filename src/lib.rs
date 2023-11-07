use headless_chrome::protocol::cdp::Page;
use headless_chrome::{Browser, LaunchOptions};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

mod composition;
mod ffmpeg;

pub struct RenderOptions {
    pub bundle: String,
    pub output: String,
    pub composition: String,
    pub props: Option<String>,
    pub frames: Option<String>,
}

pub enum UserEvent {
    PageLoaded,
    FrameLoaded,
    FramesComplete,
    GetCompositions(String),
}

fn unescape_json_string(escaped_json: &str) -> Result<String, serde_json::Error> {
    // Trim the leading and trailing double quotes, then unescape the string
    let trimmed = escaped_json.trim_matches('"');
    let unescaped = serde_json::from_str::<String>(&format!("\"{}\"", trimmed))?;
    Ok(unescaped)
}

pub fn render(options: RenderOptions) -> Result<(), Box<dyn Error>> {
    // 1. Validate: bundle, composition, frames, props
    println!("Rendering with options: {:?}", options.bundle);
    let bundle_path = PathBuf::from(&options.bundle)
        .canonicalize()
        .expect("Failed to find bundle");
    let output_file = options.output;
    let composition = options.composition;
    let frame_start = 0;
    let frame_end = 30;

    // 2. Create Browser and tab
    let browser = Browser::new(LaunchOptions {
        headless: true,
        enable_gpu: true,
        window_size: Some((1920, 1080)),
        ..Default::default()
    })
    .expect("failed to launch browser");
    let tab = browser.new_tab().expect("Failed to create tab");

    // load the contents of index.html and navigate there
    let html_file_url = format!("file://{}", bundle_path.to_str().unwrap());
    tab.navigate_to(&html_file_url)?;

    // load and evaluate the javascript
    // TODO: get js snippet from bundle dynamically
    let js = fs::read_to_string("./bundle/bundle.js").expect("Failed to read index.js");
    tab.evaluate(js.as_str(), false).expect("couldn't run js");

    // Get data about the Remotion bundle
    tab.evaluate(
        "window.remotion_setBundleMode({ type: 'evaluation' });",
        false,
    )
    .expect("couldn't set bundlemode to evaluation");

    // TODO: figure out waiting mechanism.
    // wait for 2 seconds
    // Wait::with_sleep(std::time::Duration::from_secs(2));

    let compositions = tab
        .evaluate(
            "window.getStaticCompositions().then(comps => JSON.stringify(comps))",
            true,
        )
        .expect("couldn't get comps")
        .value
        .unwrap()
        .to_string();

    let escaped_comps = unescape_json_string(&compositions).expect("couldn't unescape comps");
    let comps = composition::derive(escaped_comps);

    let comp = comps
        .iter()
        .find(|c| c.id == composition)
        .expect("No matching Composition found");

    // 3. Set up Event Loop
    let mut frame_current = frame_start;
    let mut frame_duration = frame_end;
    let mut fps = 30;

    // TODO: write this into a temp directory
    let frame_dir = Path::new("./frames");
    if !frame_dir.exists() {
        fs::create_dir(frame_dir).expect("Failed to create frame directory");
    }

    // TODO: figure out how to apply these
    let width = comp.width;
    let height = comp.height;

    // Update frame duration and fps for the event loop
    frame_duration = comp.durationInFrames;
    fps = comp.fps;

    // Prepare composition for rendering
    let composition_prep_script = format!(
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
        comp.serializedDefaultPropsWithCustomSchema,
        comp.durationInFrames,
        comp.fps,
        comp.height,
        comp.width
    );

    tab.evaluate(&composition_prep_script, true)
        .expect("couldn't set composition");

    // sleep for 2 seconds main thread
    std::thread::sleep(std::time::Duration::from_secs(1));

    // 4. Capture Frames
    for frame in 1..frame_duration {
        println!("Writing Frame: {}", frame);

        // Updates bundle app to current frame
        let set_frame_command = format!("window.remotion_setFrame({}, '{}');", frame, composition);
        tab.evaluate(set_frame_command.as_str(), true)
            .expect("couldn't set frame");

        // take screenshot
        let png_data =
            tab.capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true)?;
        let name = format!("frame-{}.png", frame);
        let path = frame_dir.join(name);
        std::fs::write(path, png_data)?;
    }

    println!("Frames captured.");
    tab.close(true).expect("Failed to close tab");

    // 5. Encode into video
    ffmpeg::encode_video(&output_file, fps, frame_dir)?;
    println!("Video encoded.");
    return Ok(());
}
