use std::{
    fs::{read_to_string, File},
    io::Write,
    path::PathBuf,
};
use wry::{
    application::{
        dpi::PhysicalSize,
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoopBuilder},
        window::{Window, WindowBuilder},
    },
    webview::{ScreenshotRegion, WebView, WebViewBuilder},
};

use crate::ffmpeg;

pub struct RenderOptions {
    pub bundle: String,
    pub output: String,
    pub composition: String,
    pub props: Option<String>,
    pub frames: Option<String>,
    pub muted: Option<bool>,
}

enum UserEvent {
    PageLoaded,
    FramePainted,
}

fn save_frame_to_file(webview: &WebView, frame: u32) {
    webview
        .screenshot(
            ScreenshotRegion::Visible,
            move |image: wry::Result<Vec<u8>>| {
                let filename = format!("frame-{}.png", frame);
                let image = image.expect("Couldn't get image");
                // println!("image: {:?}", image);

                let mut file = File::create(filename).expect("Couldn't create the file");

                file.write(image.as_slice())
                    .expect("Couldn't write to file");
            },
        )
        .unwrap();
}

pub fn render(options: RenderOptions) -> wry::Result<()> {
    // 1. Validate props and process bundle
    // options.bundle
    // options.composition
    // options.frames
    // options.props

    let bundle_path = PathBuf::from(options.bundle);
    let _html_content = read_to_string(bundle_path).expect("Failed to read HTML file");

    let width = 1920;
    let height = 1080;

    // 2. Create Window and Webview

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(width, height))
        .build(&event_loop)?;

    let ipc_handler = move |_: &Window, req: String| {
        if req == "page-loaded" {
            let _ = proxy.send_event(UserEvent::PageLoaded);
        } else if req == "frame-painted" {
            let _ = proxy.send_event(UserEvent::FramePainted);
        }
    };

    let wv = WebViewBuilder::new(window)?
        .with_html(
            r#"
            <html>
            <body style="display:flex;align-items:center;justify-content:center;">
            <div id="box">YOLO</div>
            <style>#box{display:flex;align-items:center;justify-content:center;height:200px;width:200px;background:lightsalmon;font-family:sans-serif;font-weight:bold;font-size:3rem;}</style>
            </body>
            </html>
        "#,
        )?
        .with_initialization_script(r#"
        (function () {
            window.addEventListener('DOMContentLoaded', (event) => {
                window.ipc.postMessage('page-loaded');
            });
        })();
        "#,)
        .with_ipc_handler(ipc_handler)
        .build()?;

    // 3. Set up Event Loop
    let mut current_frame = 0;
    let max_frames = 120;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
            Event::UserEvent(UserEvent::PageLoaded) => {
                println!("Page loaded!");

                // Kick off frame rendering
                wv.evaluate_script(r#"window.ipc.postMessage('frame-painted');"#)
                    .expect("Failed to send message");
            }
            Event::UserEvent(UserEvent::FramePainted) => {
                if current_frame == max_frames {
                    // Exit the loop
                    println!("All frames painted");

                    // Converting to video
                    ffmpeg::render_screenshots_to_video("test.mp4".into()).unwrap();

                    *control_flow = ControlFlow::Exit;
                    return;
                }

                println!("Frame {} painted!", current_frame);

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
                save_frame_to_file(&wv, current_frame);

                // advance frame
                current_frame += 1;
                wv.evaluate_script(r#"window.ipc.postMessage('frame-painted');"#)
                    .expect("Failed to send message");
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
