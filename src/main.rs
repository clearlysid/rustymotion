use rustymotion::{render, RenderOptions};

fn main() {
    // Provide the options for rendering
    let options = RenderOptions {
        bundle: "bundles/transitions-video".into(),
        composition: "Main".into(),
        output: "out.mp4".into(),
        frames: Some((0, 50)),
        props: None,
    };

    // Render the project
    render(options).expect("Error rendering project");
}
