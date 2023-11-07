use rustymotion::{render, RenderOptions};

fn main() {
    // Provide the options for rendering
    let options = RenderOptions {
        bundle: "bundles/hello-world".into(),
        composition: "HelloWorld".into(),
        output: "out.mp4".into(),
        frames: None,
        props: None,
    };

    // Render the project
    render(options).expect("Error rendering project");
}
