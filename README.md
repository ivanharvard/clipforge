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
- `crates/clipforge-web-bindings` — browser-safe Wasm bindings for projects,
  editing operations, normalized probe metadata, and ffmpeg.wasm arguments.

See [DESIGN.md](DESIGN.md) for the visual design system and
[AGENT.md](AGENT.md) for repository conventions (file size limits, folder
structure, commit style, tooling).

## Building

```sh
make setup   # checks for cargo, ffmpeg, ffprobe, libmpv, rustfmt, clippy
make build   # cargo build --workspace
make run     # cargo run -p clipforge-app (add CLIP=/path/to/file.mp4 to open a clip)
make check   # fmt-check + clippy + test, same as CI
make web-bindings # build the browser package (requires wasm-pack)
```

Run `make help` for the full list of targets, including packaging
(`package-appimage`, `package-pacman`, `package-msi`) and a user-level
`install`/`uninstall` for Linux. Requires `ffmpeg`/`ffprobe` on `PATH` and a
system libmpv installation.

### On Windows

See COMPILING.md.
