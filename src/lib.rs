use std::fs;
use std::{
    io::Write,
    path::{Path, PathBuf},
};
use wry::{
    application::{
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoopBuilder},
    },
    webview::ScreenshotRegion,
};

mod composition;
mod ffmpeg;
mod webview;

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

pub fn render(options: RenderOptions) -> wry::Result<()> {
    // 1. Validate: bundle, composition, frames, props

    println!("Rendering with options: {:?}", options.bundle);
    let bundle_path = PathBuf::from(options.bundle);
    let output_file = options.output;
    let composition = options.composition;
    let frame_start = 0;
    let frame_end = 30;

    // 2. Create Webview
    let event_loop: wry::application::event_loop::EventLoop<UserEvent> =
        EventLoopBuilder::<UserEvent>::with_user_event().build();
    let wv = webview::init(&event_loop, bundle_path);

    // 3. Set up Event Loop
    let mut frame_current = frame_start;
    let mut frame_duration = frame_end;
    let mut fps = 30;

    // TODO: write this into a temp directory
    let frame_dir = Path::new("./frames");

    if !frame_dir.exists() {
        fs::create_dir(frame_dir).expect("Failed to create frame directory");
    }

    // 4. Run Event Loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::NewEvents(StartCause::Init) => println!("Render started!"),
            Event::UserEvent(UserEvent::PageLoaded) => {
                // We can run Remotion commands here to try stuff
                // See reference.js to know what we can do
                wv.evaluate_script(
                    r#"
                    window.remotion_setBundleMode({ type: 'evaluation' });
                    setTimeout(() => {
                        window.getStaticCompositions().then(comps => JSON.stringify(comps)).then(json => {
                            window.ipc.postMessage(`get-compositions:${json}`);
                        });
                    }, 1000);
                    "#
                ).unwrap();
            }
            Event::UserEvent(UserEvent::GetCompositions(compositions)) => {
                let comps = composition::derive(compositions);
                let comp = comps.iter().find(|c| c.id == composition).expect("No matching Composition found");

                // TODO: figure out how to apply these
                // We need to change the webview size to the following
                // let width = comp.width;
                // let height = comp.height;

                // Update frame duration and fps for the event loop
                frame_duration = comp.duration_in_frames;
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
                    comp.serialized_default_props_with_custom_schema,
                    comp.duration_in_frames,
                    comp.fps,
                    comp.height,
                    comp.width
                );

                // println!("Prepping composition: {}", composition_prep_script);
                wv.evaluate_script(&composition_prep_script).unwrap();

                webview::fire_event(&wv, UserEvent::FrameLoaded, Some(200));
            }
            Event::UserEvent(UserEvent::FrameLoaded) => {
                if frame_current == frame_duration {
                    webview::fire_event(&wv, UserEvent::FramesComplete, None);
                } else {
                    println!("Writing Frame: {}", frame_current);

                    // Updates bundle app to current frame
                    let set_frame_command = format!(
                        "window.remotion_setFrame({}, '{}');",
                        frame_current, composition
                    );
                    wv.evaluate_script(set_frame_command.as_str()).unwrap();

                    // Save frame to file
                    wv.screenshot(
                        ScreenshotRegion::Visible,
                        move |image: wry::Result<Vec<u8>>| {
                            let image = image.expect("Couldn't get image");
                            let name = format!("frame-{}.png", frame_current);
                            let path = frame_dir.join(name);
                            let mut file =
                                fs::File::create(path).expect("Couldn't create the file");

                            file.write(image.as_slice())
                                .expect("Couldn't write to file");
                        },
                    )
                    .unwrap();

                    // Advance frame
                    frame_current += 1;
                    webview::fire_event(&wv, UserEvent::FrameLoaded, Some(200));
                }
            }
            Event::UserEvent(UserEvent::FramesComplete) => {
                println!("All frames painted");

                // Encode frames to video with FFmpeg
                let output = ffmpeg::encode_video(output_file.as_str(), fps, frame_dir)
                    .expect("Failed to render video");

                // Delete frames_dir
                fs::remove_dir_all(frame_dir).expect("Failed to delete frame directory");

                println!("Rendered: {}", output);

                // Exit the loop
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
