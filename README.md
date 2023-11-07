# Rustymotion

A Rust-based renderer for Remotion projects.

![](https://github.com/clearlysid/rustymotion/assets/30227512/abac28b3-5166-430e-ad82-1148cc4f2149)

> 🚧 WARNING: This is exploratory work only, use at your own risk.

## Design

[Remotion](https://www.remotion.dev) allows you to easily write videos using React.

It exposes some functions from a React app to change the layout based on a specified `composition` and `frame`. The Remotion Renderer then serves this app on a URL and uses Puppeteer to loop over it and screenshot each frame. These are then stitched into a video file using FFmpeg.

I wanted to avoid a Node.js or server-based setup for one of my projects and have tried _emulating_ the Remotion renderer using what I had on hand — this repo is the result of it working _just enough_.

## Approach

1. We start with your [Bundle](https://www.remotion.dev/docs/terminology#bundle). You can create one using either the Remotion CLI (set [log-level](https://www.remotion.dev/docs/renderer/render-media#loglevel) to `verbose`) or the [@remotion/bundler](https://www.remotion.dev/docs/bundler) tool.
2. The `headless-chrome-rust` project is used to get composition details and screenshot frames.

## Instructions

1. See `src/main.rs` for how to use the renderer.
2. Most of the renderer's code is in `src/lib.rs`.
3. Relevant findings from my tinkering with the Remotion bundle are in `reference.js`.
4. Running `cargo run` should hopefully give you a video in `out.mp4` if all goes well.

## TODO

1.

## Closing

Thanks for checking out this project. I made this hack (again) over some sleepless nights. It is not intended for production, nor can I offer any support for it. I just hope to present this as a POC to the [Remotion team](https://github.com/remotion-dev) and hopefully spark some new ideas.

I unfortunately can't take reponsibility of getting this working in your setup at the moment — but I'm happy to answer any questions or have on it over on [Twitter](https://twitter.com/clearlysid) or the Remotion Discord. See you there! 👋

PS — this is a different approach than what I did initially, you can check out my first attempt at this using [wry](https://github.com/clearlysid/wry) crate in the `wry` branch. Please refer to that branch's `readme` for more info.
