# Rustymotion

A Rust-based renderer for Remotion projects.

> 🚧 WARNING: This is exploratory work only, use at your own risk.

## Design

[Remotion](https://www.remotion.dev) allows you to easily write videos using React.

It bundles a React app exposing some global functions to change the layout, depending on the `composition` and `frame` set. The Remotion Renderer then serves this app bundle on a URL and uses Puppeteer to open it on a Chromium-based browser to take a screenshot. All the frames in a composition are "screenshotted", and then FFmpeg stitches them into a video file.

For one of my projects, I could neither use a server and nor was happy to ship a Chromium binary. So I thought of _emulating_ the Remotion renderer using what I had on hand — this repo is the result of it working _just enough_.

## Approach

1. We start with your [Bundle](https://www.remotion.dev/docs/terminology#bundle). You can create one using either the Remotion CLI (set [log-level](https://www.remotion.dev/docs/renderer/render-media#loglevel) to `verbose`) or the [@remotion/bundler](https://www.remotion.dev/docs/bundler) tool. There's one included in the project.
2. This bundled site is opened in your browser's native Webview using the excellent [wry](https://github.com/clearlysid/wry) crate. It handles Webview management across platforms and is used by [Tauri](https://github.com/tauri-apps) apps under the hood.
3. We run some JS commands in the webview to:
    1. Get a list of compositions and their metadata (dimensions, default props, etc.)
    2. Select one based on input and set it up for rendering.
4. Then, we start off the "frame screenshotting" process using [Wry's Event Loop](https://docs.rs/wry/latest/wry/application/event_loop/struct.EventLoop.html). This project uses my fork of Wry, which adds a `.screenshot()` method. The screenshots are written to a file on disk.
5. We finally use [FFmpeg](https://ffmpeg.org) to encode all the screenshots into an `.mp4` the same way Remotion does.
6. Profit ???

## Instructions

1. See `src/main.rs` for how to use the renderer.
2. Most of the renderer's code is in `src/lib.rs`.
3. Relevant findings from my tinkering with the Remotion bundle are in `reference.js`.
4. The included `bundle` is from Remotion's [Hello World](https://github.com/remotion-dev/template-helloworld) with minimal design tweaks.
5. Running `cargo run` should hopefully give you the video in `out.mp4` if all goes well.

## Caveats

I made this hack over some sleepless nights — it is not intended for production, nor can I offer any support for it. I just hope to present this as a proof-of-concept to the [Remotion team](https://github.com/remotion-dev) and hopefully spark some new ideas in folks.

Here's a list of oopsies/gotchas in my current approach. Most of them have solutions:

1. I've manually inlined `bundle.js` into its `index.html` to make it easier to load. It is possible to serve the whole bundle using Wry's `custom protocol` feature and even support `public assets`.
2. The `.screenshot()` is prone to failing. I plan to properly solve this piece and make an upstream PR in Wry.
3. We save screenshots in the file system and later on use Ffmpeg to encode them at once.
    1. This is inefficient. We can instead likely capture and encode in parallel. Just store the screenshots in memory and "stream" them to Ffmpeg using the `ffmpeg-next` crate.
    2. This should be easily doable, I simply don't know how to — maybe [@Jonny Burger](https://github.com/JonnyBurger) can help 🙈
4. It is assumed FFmpeg is available in your PATH, I don't have it bundled in the project but this should also be easily fixable with point 3.1.
5. Your OS could impose security rules/restrictions on what your webview can do and this might impact your rendering.

There are plenty of other caveats, but hopefully the comments in my code clarify how things are working.

## Closing

Thanks for checking out this project. I hope it was insightful and gives you some ideas.

I unfortunately can't take reponsibility of getting this working in your setup at the moment — but I'm happy to answer any questions or have on it over on [Twitter](https://twitter.com/clearlysid) or the Remotion Discord. See you there! 👋
