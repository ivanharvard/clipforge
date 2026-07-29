import type { FFmpeg } from "@ffmpeg/ffmpeg";
import type { ClipForgeProject } from "../lib/wasm";

import { outputName, virtualInputName } from "../lib/format";

const CORE_BASE_URL = "https://cdn.jsdelivr.net/npm/@ffmpeg/core@0.12.10/dist/esm";

let ffmpegInstance: FFmpeg | null = null;
let loaded = false;

interface ExportCallbacks {
  onPhase: (message: string) => void;
  onProgress: (progress: number) => void;
}

async function ffmpeg(callbacks: ExportCallbacks): Promise<FFmpeg> {
  const [{ FFmpeg }, { toBlobURL }] = await Promise.all([
    import("@ffmpeg/ffmpeg"),
    import("@ffmpeg/util"),
  ]);
  ffmpegInstance ??= new FFmpeg();
  if (!loaded) {
    callbacks.onPhase("Loading the video engine…");
    await ffmpegInstance.load({
      coreURL: await toBlobURL(`${CORE_BASE_URL}/ffmpeg-core.js`, "text/javascript"),
      wasmURL: await toBlobURL(`${CORE_BASE_URL}/ffmpeg-core.wasm`, "application/wasm"),
    });
    loaded = true;
  }
  return ffmpegInstance;
}

export async function exportVideo(
  file: File,
  project: ClipForgeProject,
  callbacks: ExportCallbacks,
): Promise<string> {
  const inputName = virtualInputName(file.name);
  const renderedName = "output.mp4";
  const args = JSON.parse(
    project.buildExportArgsJson(inputName, renderedName),
  ) as string[];
  const [{ fetchFile }, engine] = await Promise.all([
    import("@ffmpeg/util"),
    ffmpeg(callbacks),
  ]);
  const progressHandler = ({ progress }: { progress: number }) => {
    callbacks.onProgress(Math.min(1, Math.max(0, progress)));
  };

  engine.on("progress", progressHandler);
  try {
    callbacks.onPhase("Preparing your clip…");
    await engine.writeFile(inputName, await fetchFile(file));
    callbacks.onPhase("Exporting locally…");
    const exitCode = await engine.exec(args);
    if (exitCode !== 0) {
      throw new Error(`FFmpeg exited with status ${exitCode}`);
    }

    callbacks.onPhase("Saving your video…");
    const data = await engine.readFile(renderedName);
    if (typeof data === "string") {
      throw new Error("FFmpeg returned an unexpected text output");
    }
    const blob = new Blob([new Uint8Array(data)], { type: "video/mp4" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = outputName(file.name);
    anchor.click();
    setTimeout(() => URL.revokeObjectURL(url), 30_000);
    callbacks.onProgress(1);
    return anchor.download;
  } finally {
    engine.off("progress", progressHandler);
    await Promise.allSettled([
      engine.deleteFile(inputName),
      engine.deleteFile(renderedName),
    ]);
  }
}

export function cancelExport() {
  ffmpegInstance?.terminate();
  ffmpegInstance = null;
  loaded = false;
}
