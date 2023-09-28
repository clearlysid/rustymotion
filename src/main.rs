use wry::{
    application::{
        dpi::{PhysicalPosition, PhysicalSize},
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    webview::WebViewBuilder,
};

#[cfg(target_os = "macos")]
use wry::webview::WebviewExtMacOS;

#[cfg(target_os = "windows")]
use wry::webview::WebviewExtWindows;

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
        .build(&event_loop)?;

    let _webview = WebViewBuilder::new(window)?
        .with_url("https://www.headout.com")?
        .with_focused(true)
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                println!("Wry has started!");
                println!("Window size: {}x{}", width, height);
                println!("Window position: {}x{}", x_pos, y_pos);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
