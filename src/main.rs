use rustymotion::{render, RenderOptions};

fn main() -> Result<(), ()> {
    // Provide the options for rendering
    // Currently the "props" and "frames" options do nothing
    let options = RenderOptions {
        bundle: "bundle/index.html".into(),
        output: "out".into(),
        composition: "HelloWorld".into(),
        props: None,
        frames: None,
    };

    // Render the project
    render(options).expect("Error rendering project");

    Ok(())
}
