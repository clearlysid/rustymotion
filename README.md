# Rustymotion

A Rust-based renderer for Remotion projects.

> 🚧 WARNING: This is exploratory work only, use at your own risk.

## Design

[Remotion](https://www.remotion.dev) allows you to easily write videos using React.

It bundles a React app exposing some global functions to change the layout, depending on the `composition` and `frame` set. The Remotion Renderer then serves this app bundle on a URL and uses Puppeteer to open it on a Chromium-based browser to take a screenshot. It does this for each frame in a composition. And once all the frames are "screenshotted", FFmpeg stitches them into a video file.

Under the hood Remotion does a lot of edge-case handling and performance improvements to make this process as fast and robust as it can be.

For one of my projects, neither could I use a server and nor was shipping a Chromium binary feasible. So I thought of _emulating_ the Remotion renderer using tools available at hand and see if I can maybe get it working _just enough_ — this repo is the result.

## Approach

1. We start with your [Remotion Bundle](https://www.remotion.dev/docs/terminology#bundle). You can create one for your project using either the CLI (set [log-level](https://www.remotion.dev/docs/renderer/render-media#loglevel) to `verbose`) or the [@remotion/bundler](https://www.remotion.dev/docs/bundler) tool.
2. This bundled site is opened in your browser's native Webview using the excellent [wry](https://github.com/clearlysid/wry) crate from the folks at [Tauri](https://github.com/tauri-apps). It handles Webview management for all the 3 platforms: Windows, macOS and Linux. All Tauri apps also use this under the hood.
3. We run [some Javascript functions](https://github.com/clearlysid/remotion-renderer-rs/blob/main/src/reference.js) in the webview to:
    1. Get a list of compositions and their metadata (dimensions, default props, etc.)
    2. Select one based on input and set it up for rendering.
4. Then, we start off the "frame screenshotting" process using [Wry's Event Loop](https://docs.rs/wry/latest/wry/application/event_loop/struct.EventLoop.html). This project uses my fork of Wry, which adds a `.screenshot()`` method for all platforms. The screenshots are written to a file on disk.
5. We loop over the whole composition, taking screenshots for each frame and finally use [FFmpeg](https://ffmpeg.org) to encode them into an `.mp4` the same way Remotion does by default.

## Instructions

1. `src/main.rs` shows you an example of how to use the rustymotion renderer.
2. most of the renderer's code is in `src/lib.rs`.
3. Relevant findings from my tinkering with the Remotion bundle are in `reference.js`.
4. `bundle` is the bundle from Remotion's [Hello World](https://github.com/remotion-dev/template-helloworld) with minimal design tweaks.
5. Running the project with `cargo run` should give you the video in `out.mp4` (or what you've defined in main.rs).

## Caveats

This project was a hack I made over some sleepless nights. It is not intended for production, nor can I offer any support for it. I hope to present this as a proof-of-concept to the [Remotion team](https://github.com/remotion-dev) and hopefullt spark some new ideas in folks.

Here's a list of oopsies and gotchas in my current approach. Most of them have solutions/workarounds.

1. In the bundle, I've manually inlined `bundle.js` into `index.html` to make it easier to load in the Webview. It is possible to serve the whole bundle on Wry using its custom protocol feature though — which will prevent the need for this, as well as support `public assets` from your bundle.
2. The `.screenshot()` is prone to errors and failing. I plan to properly solve this piece and make an upstream PR in Wry.
3. We save screenshots in the file system and later on use Ffmpeg to encode then at once. This is inefficient, we can instead perhaps capture and encode in parallel. Just store the screenshots in memory and "stream" them to Ffmpeg using the `ffmpeg-next` crate. This should be easily doable, I simply don't know how to — maybe [@Jonny Burger](https://github.com/JonnyBurger) can help 🙈
4. It is assumed FFmpeg is available in your PATH, I don't have ffmpeg bundled in the project but this should also be easily fixable.

There are plenty of other caveats, hopefully the comments in my code clarify how it works.

## Closing

Thanks for checking out this project. I hope it was insightful and maybe gives you more ideas.

I unfortunately can't take reponsibility of getting this working in your setup at the moment — but I'm happy to answer any questions or have on it over on [Twitter](https://twitter.com/clearlysid) or the Remotion Discord. See you there! 👋
