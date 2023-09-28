use std::{fs::File, io::Write};
use wry::{
    application::{
        dpi::{PhysicalPosition, PhysicalSize},
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoopBuilder},
        window::{Window, WindowBuilder},
    },
    webview::{ScreenshotRegion, WebView, WebViewBuilder},
};

enum UserEvent {
    PageLoaded,
}

fn save_frame_to_file(webview: &WebView, frame: u32) {
    webview
        .screenshot(
            ScreenshotRegion::Visible,
            move |image: wry::Result<Vec<u8>>| {
                let filename = format!("frame_{}.png", frame);
                let image = image.expect("Couldn't get image");
                let mut file = File::create(filename).expect("Couldn't create the file");
                file.write(image.as_slice())
                    .expect("Couldn't write to file");
            },
        )
        .unwrap();
}

fn main() -> wry::Result<()> {
    const FRAMES: u32 = 30;

    let width = 1920;
    let height = 1080;
    let x_pos = 0;
    let y_pos = 0;

    let size = PhysicalSize::new(width, height);
    let position = PhysicalPosition::new(x_pos, y_pos);

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_decorations(false)
        .with_inner_size(size)
        .with_position(position)
        .with_always_on_top(true)
        .with_focused(true)
        .build(&event_loop)?;

    let ipc_handler = move |_: &Window, req: String| {
        if req == "page-loaded" {
            let _ = proxy.send_event(UserEvent::PageLoaded);
        }
    };

    let _webview = WebViewBuilder::new(window)?
        .with_html(r#"<html><body><h1>Frame 1</h1></body></html>"#)?
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
    // _webview.open_devtools();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                println!("Wry has started!");
            }
            Event::UserEvent(UserEvent::PageLoaded) => {
                println!("Page loaded!");

                // Save frame to file
                save_frame_to_file(&_webview, 1);

                let set_frame_command = format!("remotion_setFrame({})", 2);

                _webview
                    .evaluate_script(set_frame_command.as_str())
                    .unwrap();

                // _webview
                //     .evaluate_script_with_callback("document.readyState", |result| {
                //         println!("String: {:?}", result)
                //     })
                //     .unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
