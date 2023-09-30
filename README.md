# Rustymotion

A Rust-based renderer for Remotion projects.

> EXPLORATORY WORK ONLY: Use at your own risk 🚧

For a project where I wanted to use [Remotion](https://www.remotion.dev), shipping a chromium/browser binary was not a very appealing idea. Instead I thought of tinkering around with the Remotion renderer works, and see if I can maybe get it working "just enough" without it.

## Design

Remotion bundles a React app exposing some global functions to change the layout of the app, depending on the "composition" and "frame" that are set. The Remotion Renderer then serves this app bundle on a URL and uses Puppeteer to open it on a Chromium-based browser to take a screenshot. It does this for each frame in a composition, and once all the frames are "screenshotted", Remotion uses FFmpeg to stitch them into a video file.

Under the hood there are a lot of edge-cases, quirks and performance improvements that are done to make this process as fast and robust as it can be.

## Approach

1. We start with your [Remotion Bundle](https://www.remotion.dev/docs/terminology#bundle). You can create one for your project using either the CLI (set [log-level](https://www.remotion.dev/docs/renderer/render-media#loglevel) to "verbose") or the [@remotion/bundler](https://www.remotion.dev/docs/bundler) tool.
2. This bundled site is opened in your browser's native Webview with the excellent [wry](https://github.com/clearlysid/wry) crate from the folks at [Tauri](https://github.com/tauri-apps). It handles Webview management for all the 3 platforms: Windows, macOS and Linux. All Tauri apps also use under the hood.
3. We then run [some Javascript functions](https://github.com/clearlysid/remotion-renderer-rs/blob/main/src/reference.js) in the webview to:
    1. Get a list of compositions and their metadata
    2. Select one based on input and set it up for rendering.
4. Then, we start off the "frame screenshotting" process within [Wry's Event Loop](https://docs.rs/wry/latest/wry/application/event_loop/struct.EventLoop.html). This project using my fork of Wry, which adds the `.screenshot()`` method across all platforms. This screenshots is written to a file on disk.
5. We loop over the whole duration taking screenshots and finally use [FFmpeg](https://ffmpeg.org) to encode them into an `.mp4` file the same way Remotion would do by default.

##

### TO-DOs

-   [x] Get Remotion Bundle working
-   [x] Encode images to video using FFmpeg
-   [x] Code structure for rendering
