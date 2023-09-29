use std::{
    fs::{read_to_string, remove_dir_all, File},
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

use crate::{ffmpeg, webview};

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
}

pub fn render(options: RenderOptions) -> wry::Result<()> {
    // 1. Validate props and process bundle
    // bundle, composition, frames, props

    let bundle_path = PathBuf::from(options.bundle);
    let output_file = "out.mp4";
    let frame_start = 0;
    let frame_end = 30;

    let _html_content = read_to_string(bundle_path).expect("Failed to read HTML file");

    let width = 1920;
    let height = 1080;

    // 2. Create Webview
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let wv = webview::init(&event_loop, width, height);

    // 3. Set up Event Loop
    let mut frame_current = frame_start;

    // TODO: write this into a temp directory
    let frame_dir = Path::new("./frames");

    if !frame_dir.exists() {
        std::fs::create_dir(frame_dir).expect("Failed to create frame directory");
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::NewEvents(StartCause::Init) => println!("Render started!"),
            Event::UserEvent(UserEvent::PageLoaded) => {
                webview::fire_event(&wv, UserEvent::FrameLoaded);
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
                            // println!("image: {:?}", image);

                            let filename = format!("frame-{}.png", frame_current);
                            let path = frame_dir.join(filename);
                            let mut file = File::create(path).expect("Couldn't create the file");

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
                let output = ffmpeg::encode_video(output_file, "30", frame_dir)
                    .expect("Failed to render video");

                // delete frames_dir
                remove_dir_all(frame_dir).expect("Failed to delete frame directory");

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
