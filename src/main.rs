use std::{fs::File, io::Write};
use wry::{
    application::{
        dpi::{PhysicalPosition, PhysicalSize},
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    webview::{ScreenshotRegion, WebViewBuilder},
};

fn main() -> wry::Result<()> {
    let width = 800; // physical pixels
    let height = 600; // physical pixels

    let x_pos = 0;
    let y_pos = 0;

    let size = PhysicalSize::new(width, height);
    let position = PhysicalPosition::new(x_pos, y_pos);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_decorations(false)
        .with_inner_size(size)
        .with_position(position)
        .with_always_on_top(true)
        .with_focused(true)
        .build(&event_loop)?;

    let _webview = WebViewBuilder::new(window)?
        .with_url("https://www.google.com")?
        .with_devtools(true)
        // .with_initialization_script("console.log('init script');")
        .build()?;

    _webview.open_devtools();

    _webview
        .screenshot(ScreenshotRegion::Visible, |image: wry::Result<Vec<u8>>| {
            let image = image.expect("No image?");
            let mut file = File::create("baaaaar.png").expect("Couldn't create the dang file");
            file.write(image.as_slice())
                .expect("Couldn't write the dang file");
        })
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {}
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
