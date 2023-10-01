use core::panic;
use std::fs;
use std::path::PathBuf;
use wry::{
    application::{
        dpi::PhysicalSize,
        event_loop::EventLoop,
        window::{Window, WindowBuilder},
    },
    webview::{WebView, WebViewBuilder},
};

use crate::UserEvent;

pub fn init(event_loop: &EventLoop<UserEvent>, entry_point: PathBuf) -> WebView {
    let width = 1920;
    let height = 1080;

    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(width, height))
        .build(&event_loop)
        .unwrap();

    let html = fs::read_to_string(entry_point).expect("Failed to read HTML file");

    WebViewBuilder::new(window)
        .unwrap()
        .with_html(html)
        .unwrap()
        .with_initialization_script(
            r#"
            window.addEventListener('DOMContentLoaded', (event) => {
                window.ipc.postMessage('page-loaded');
            });
        "#,
        )
        .with_ipc_handler(move |_: &Window, req: String| {
            if req.starts_with("get-compositions:") {
                let comps = req.replace("get-compositions:", "");
                let _ = proxy.send_event(UserEvent::GetCompositions(comps));
            } else {
                let event = match req.as_str() {
                    "page-loaded" => UserEvent::PageLoaded,
                    "frame-loaded" => UserEvent::FrameLoaded,
                    "frames-complete" => UserEvent::FramesComplete,
                    _ => panic!("Unknown event: {}", req),
                };

                let _ = proxy.send_event(event);
            }
        })
        .build()
        .unwrap()
}

pub fn fire_event(webview: &WebView, event: UserEvent, delay: Option<u32>) {
    let event_key = match event {
        UserEvent::PageLoaded => "page-loaded",
        UserEvent::FrameLoaded => "frame-loaded",
        UserEvent::FramesComplete => "frames-complete",
        _ => "none",
    };

    let mut script = format!("window.ipc.postMessage('{}');", event_key);

    if delay.is_some() {
        let delay = delay.unwrap();
        script = format!(
            "setTimeout(() => {{ window.ipc.postMessage('{}'); }}, {});",
            event_key, delay
        );
    }

    webview
        .evaluate_script(&script)
        .expect("Failed to fire event");
}
