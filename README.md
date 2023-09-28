# remotion-renderer-rs

A Rust-based renderer for Remotion projects.

> EXPLORATORY ONLY — Use at your own risk 🚧

For a project where I wanted to use Remotion, shipping a chromium/browser binary was not a very appealing idea. Instead I thought if I tinker around and see how the Remotion renderer works, I can maybe get it working "just enough" a little differently.

### TO-DOs

-   [ ] Get Remotion Bundle working
-   [ ] Encode images to video using FFmpeg API (not CLI)
-   [ ] Code structure for rendering

#### Options in POC

1. Bundle location / URL (mandatory)
2. Output location (default: out folder in cwd)
3. Composition (mandatory, but can improve ux in v2)
4. Input Props (serialized JSON string)
5. Frame Range (Example: --frames=0-9 to select the first 10 frames, leaving blank = all frames)
6. Muted (false)

#### Consider in Phase 2

1. Height
2. Width
3. Codec (low-prio) — use h264/h265
4. Audio Codec (low-prio) — selected basis Codec
5. Audio Bitrate
6. Video Bitrate
7. Image Format
8. Pixel Format
9. Env Variables
10. Jpeg quality
11. Enforce Audio Track
12. Overwrite
13. Sequence
14. Public folder
