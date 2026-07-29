# ClipForge Web Bindings

This crate exposes ClipForge's platform-neutral editing model to JavaScript.
It does not run FFmpeg itself: the web application owns the browser `File`,
executes ffmpeg.wasm in a worker, and passes virtual filesystem names into the
bindings when it builds an export command.

## Build

Install the browser target and `wasm-pack`, then build an ES module package:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
make web-bindings
```

The generated JavaScript, TypeScript declarations, and Wasm binary are written
to `web/src/generated/clipforge-wasm/`. They are build artifacts and are not
committed.

## Browser usage

```ts
import init, {
  ClipForgeProject,
  parse_probe_output,
} from "./generated/clipforge-wasm/clipforge_web_bindings.js";

await init();

const project = new ClipForgeProject(
  file.name,
  video.videoWidth,
  video.videoHeight,
  Math.round(video.duration * 1000),
  30,
);

project.setTrim(1_000, 8_000);
project.rotateClockwise();
project.setCrop(0, 0, 1280, 720, false);
project.setResolution("720p", 0, 0, true);
project.setAudio(1, false, -1, false);
project.setCompression("crf", 23, "automatic", "h264", false, 25);

const args: string[] = JSON.parse(
  project.buildExportArgsJson("input.mp4", "output.mp4"),
);
await ffmpeg.exec(args);

const savedProject = project.toJson();
const restored = ClipForgeProject.fromJson(savedProject);
```

`parse_probe_output(ffprobeJson)` converts raw ffprobe JSON into the normalized
metadata shape used by ClipForge. Millisecond values use JavaScript numbers and
are limited to the unsigned 32-bit range, which supports clips up to roughly 49
days long.

Accepted string choices are:

- Resolution: `original`, `1080p`, `720p`, `480p`, `custom`
- Compression: `crf`, `bitrate`, `target-size`
- Frame-rate limit: `automatic`, `30`, `60`
- Codec: `h264`, `av1`

The prebuilt ffmpeg.wasm core may not contain every codec exposed by ClipForge.
In particular, callers should only offer AV1 after confirming that their chosen
FFmpeg core includes `libaom-av1`.
