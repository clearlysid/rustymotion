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

pub struct RenderOptions {
    pub bundle: String,
    pub output: String,
    pub composition: String,
    pub props: Option<String>,
    pub frames: Option<String>,
    pub muted: Option<bool>,
}

const FRAMES: u32 = 30;

enum UserEvent {
    PageLoaded,
}

fn save_frame_to_file(webview: &WebView, frame: u32) {
    webview
        .screenshot(
            ScreenshotRegion::Visible,
            move |image: wry::Result<Vec<u8>>| {
                let filename = format!("frame-{}.png", frame);
                let image = image.expect("Couldn't get image");
                println!("image: {:?}", image);

                let mut file = File::create(filename).expect("Couldn't create the file");

                file.write(image.as_slice())
                    .expect("Couldn't write to file");
            },
        )
        .unwrap();
}

pub fn render(options: RenderOptions) -> wry::Result<()> {
    // 1. Validate props
    // options.bundle
    // options.composition
    // options.frames
    // options.props

    // 2. Process the Bundle
    let bundle_path = PathBuf::from(options.bundle);

    let html_content = read_to_string(bundle_path).expect("Failed to read HTML file");

    println!("HTML content: {}", html_content);

    let width = 1920;
    let height = 1080;

    // 2. Create Webview

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(width, height))
        .build(&event_loop)?;

    let ipc_handler = move |_: &Window, req: String| {
        if req == "page-loaded" {
            let _ = proxy.send_event(UserEvent::PageLoaded);
        }
    };

    let _webview = WebViewBuilder::new(window)?
        .with_html(
            r#"
            <html>
            <body style="display:flex;align-items:center;justify-content:center;">
            <div style="height:200px;width:200px;background:lightsalmon;">YOLO</div>
            </body>
            </html>
        "#,
        )?
        .with_initialization_script(
            r#"
                (function () {
                    window.addEventListener('DOMContentLoaded', (event) => {
                        window.ipc.postMessage('page-loaded');
                    });
                })();
                "#,
        )
        .with_ipc_handler(ipc_handler)
        .build()?;

    // 3. Set up Event Loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                println!("Wry has started!");

            },
            Event::UserEvent(UserEvent::PageLoaded) => {
                println!("Page loaded!");

                for frame in 0..FRAMES {
                    // Save frame to file
                    println!("Saving frame {}", frame);
                    let set_frame_command =
                        format!(r#"
                        document.body.innerHTML = '<div style="height:200px;width:200px;background:lightsalmon;transform:rotate({}deg);">YOLO</div>'
                        "#, frame * 2);
                        // format!("remotion_setFrame({})", frame);

                    _webview
                        .evaluate_script(set_frame_command.as_str())
                        .unwrap();

                    save_frame_to_file(&_webview, frame);                    
                }

                // sleep(Duration::from_millis(1000));
                // ffmpeg::render_screenshots_to_video("test.mp4".into()).unwrap();

                // exit the app
                // *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
