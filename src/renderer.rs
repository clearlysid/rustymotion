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

use crate::{composition, ffmpeg, webview};

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
    let bundle_path = PathBuf::from("../bundle/index.html");
    let output_file = "out.mp4";
    let frame_start = 0;
    let frame_end = 30;
    let composition = "HelloWorld";

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

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::NewEvents(StartCause::Init) => println!("Render started!"),
            Event::UserEvent(UserEvent::PageLoaded) => {
                // We can run Remotion commands here to try stuff
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
                // println!("Got compositions: {}", compositions);
                let comps = composition::derive(compositions);
                let comp = comps.iter().find(|c| c.id == composition).expect("No matching Composition found");

                println!("DurationInFrames: {:?}", comp.durationInFrames);

                // TODO: figure out how to apply these
                // let width = comp.width;
                // let height = comp.height;

                frame_duration = comp.durationInFrames;
                fps = comp.fps;
                // let default_props = comp.serializedDefaultPropsWithCustomSchema.clone();




                 // webview::fire_event(&wv, UserEvent::FrameLoaded);
            }
            Event::UserEvent(UserEvent::FrameLoaded) => {
                if frame_current == frame_end {
                    webview::fire_event(&wv, UserEvent::FramesComplete);
                } else {
                    println!("Writing Frame: {}", frame_current);

                    // Updates contents to current frame
                    let set_frame_command = format!(
                        r#"
                     document.getElementById('box').style.transform = 'rotate({}deg)';
                     "#,
                        frame_current * 2
                    );
                    // format!("remotion_setFrame({})", frame);
                    wv.evaluate_script(set_frame_command.as_str()).unwrap();

                    // save frame to file
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

                    // advance frame
                    frame_current += 1;
                    webview::fire_event(&wv, UserEvent::FrameLoaded);
                }
            }
            Event::UserEvent(UserEvent::FramesComplete) => {
                println!("All frames painted");

                // 4. Render frames to video
                let output = ffmpeg::encode_video(output_file, fps, frame_dir)
                    .expect("Failed to render video");

                // delete frames_dir
                fs::remove_dir_all(frame_dir).expect("Failed to delete frame directory");

                println!("Rendered: {}", output);

                // 5. Exit the loop
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
