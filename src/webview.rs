use wry::{
    application::{
        dpi::PhysicalSize,
        event_loop::EventLoop,
        window::{Window, WindowBuilder},
    },
    webview::{WebView, WebViewBuilder},
};

use crate::renderer::UserEvent;

pub fn init(event_loop: &EventLoop<UserEvent>, width: u32, height: u32) -> WebView {
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(width, height))
        .build(&event_loop)
        .unwrap();

    WebViewBuilder::new(window).unwrap()
        .with_html(
            r#"
            <html>
            <body style="display:flex;align-items:center;justify-content:center;">
            <div id="box">YOLO</div>
            <style>#box{display:flex;align-items:center;justify-content:center;height:200px;width:200px;background:lightsalmon;font-family:sans-serif;font-weight:bold;font-size:3rem;}</style>
            </body>
            </html>
        "#,
        ).unwrap()
        .with_initialization_script(r#"
            window.addEventListener('DOMContentLoaded', (event) => {
                window.ipc.postMessage('page-loaded');
            });
        "#,)
        .with_ipc_handler(move |_: &Window, req: String| {
					let event = match req.as_str() {
							"page-loaded" => UserEvent::PageLoaded,
							"frame-loaded" => UserEvent::FrameLoaded,
							"frames-complete" => UserEvent::FramesComplete,
							_ => panic!("Unknown event"),
					};

					let _ = proxy.send_event(event);
			})
        .build().unwrap()
}

pub fn fire_event(webview: &WebView, event: UserEvent) {
    let event_key = match event {
        UserEvent::PageLoaded => "page-loaded",
        UserEvent::FrameLoaded => "frame-loaded",
        UserEvent::FramesComplete => "frames-complete",
    };

    let script = format!("window.ipc.postMessage('{}');", event_key);

    webview
        .evaluate_script(&script)
        .expect("Failed to fire event");
}
