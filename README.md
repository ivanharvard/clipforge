# ClipForge

A lightweight single-clip video trimmer for Windows and Linux, with one
consistent custom UI on both platforms.

## Stack

- **UI**: [Slint](https://slint.dev/) + Rust
- **Video preview**: libmpv (software-render, works uniformly across
  X11/Wayland/Windows)
- **Processing/export**: FFmpeg + ffprobe
- **Packaging**: AppImage / Flatpak / pacman (Linux), MSI / portable ZIP
  (Windows)

## Workspace layout

- `crates/clipforge-core` — pure logic (media probing, timeline, panel
  state, export/ffmpeg arg building). No UI or mpv dependency.
- `crates/clipforge-player` — libmpv wrapper. All `unsafe`/FFI is contained
  here.
- `crates/clipforge-app` — the Slint UI and the Rust glue that wires it to
  the two crates above.

See [DESIGN.md](DESIGN.md) for the visual design system and
[AGENT.md](AGENT.md) for repository conventions (file size limits, folder
structure, commit style, tooling).

## Building

```sh
cargo build --workspace
cargo run -p clipforge-app
```

Requires `ffmpeg`/`ffprobe` on `PATH` and a system libmpv installation.
