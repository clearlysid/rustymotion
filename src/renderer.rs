use std::{
    fs::{read_to_string, File},
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
    // options.bundle
    // options.composition
    // options.frames
    // options.props

    let bundle_path = PathBuf::from(options.bundle);
    let _html_content = read_to_string(bundle_path).expect("Failed to read HTML file");
    let max_frames = 120;
    let output_file = "out.mpt";

    let width = 1920;
    let height = 1080;

    // 2. Create Webview
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let wv = webview::init(&event_loop, width, height);

    // 3. Set up Event Loop
    let mut current_frame = 0;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => println!("Render started!"),
            Event::UserEvent(UserEvent::PageLoaded) => {
                webview::fire_event(&wv, UserEvent::FrameLoaded);
            }
            Event::UserEvent(UserEvent::FrameLoaded) => {
                if current_frame == max_frames {
                    webview::fire_event(&wv, UserEvent::FramesComplete);
                } else {
                    println!("Running Frame: {}", current_frame);

                    // Updates contents to current frame
                    let set_frame_command = format!(
                        r#"
                     document.getElementById('box').style.transform = 'rotate({}deg)';
                     "#,
                        current_frame * 2
                    );
                    // format!("remotion_setFrame({})", frame);
                    wv.evaluate_script(set_frame_command.as_str()).unwrap();

                    // save frame to file
                    wv.screenshot(
                        ScreenshotRegion::Visible,
                        move |image: wry::Result<Vec<u8>>| {
                            let filename = format!("frame-{}.png", current_frame);
                            let image = image.expect("Couldn't get image");
                            // println!("image: {:?}", image);

                            // write image to file in ./frames
                            let path = Path::new("./frames");
                            if !path.exists() {
                                std::fs::create_dir(path).expect("Couldn't create directory");
                            }

                            let filename = path.join(filename);
                            let mut file =
                                File::create(filename).expect("Couldn't create the file");

                            file.write(image.as_slice())
                                .expect("Couldn't write to file");
                        },
                    )
                    .unwrap();

                    // advance frame
                    current_frame += 1;
                    webview::fire_event(&wv, UserEvent::FrameLoaded);
                }
            }
            Event::UserEvent(UserEvent::FramesComplete) => {
                println!("All frames painted");

                // 4. Render frames to video
                let output = ffmpeg::encode_video(output_file).expect("Failed to render video");
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
