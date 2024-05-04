use rustymotion::{render, RenderOptions};

fn main() {
    // Provide the options for rendering
    let options = RenderOptions {
        bundle: "bundles/template-three-bundle".into(),
        composition: "Scene".into(),
        output: "out.mp4".into(),
        frames: None,
        props: None,
    };

    // Render the project
    render(options).expect("Error rendering project");
}
