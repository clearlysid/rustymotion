mod ffmpeg;
mod renderer;
mod webview;

use renderer::{render, RenderOptions};

fn main() -> Result<(), ()> {
    let options = RenderOptions {
        bundle: "bundle/index.html".to_string(),
        output: "out".to_string(),
        composition: "HelloWorld".to_string(),
        props: None,
        frames: None,
    };

    render(options).expect("Error rendering project");

    Ok(())
}
