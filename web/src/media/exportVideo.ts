import type { FFmpeg } from "@ffmpeg/ffmpeg";
import type { ClipForgeProject } from "../lib/wasm";
import { parse_probe_output } from "../lib/wasm";
import type { AudioTrack } from "../types";

import { outputName, virtualInputName } from "../lib/format";

const CORE_BASE_URL = "https://cdn.jsdelivr.net/npm/@ffmpeg/core@0.12.10/dist/esm";

let ffmpegInstance: FFmpeg | null = null;
let loaded = false;
let operationTail: Promise<void> = Promise.resolve();
const audioPreviewCache = new Map<number, Map<string, string>>();

interface ExportCallbacks {
  onPhase: (message: string) => void;
  onProgress: (progress: number) => void;
}

async function ffmpeg(callbacks: ExportCallbacks = { onPhase: () => {}, onProgress: () => {} }): Promise<FFmpeg> {
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

async function exclusive<T>(operation: () => Promise<T>): Promise<T> {
  const previous = operationTail;
  let release = () => {};
  operationTail = new Promise<void>((resolve) => { release = resolve; });
  await previous;
  try {
    return await operation();
  } finally {
    release();
  }
}

function bytesToText(value: Uint8Array | string): string {
  return typeof value === "string" ? value : new TextDecoder().decode(value);
}

function fallbackProbeJson(logs: string[]): string | null {
  const streams = new Map<number, Record<string, unknown>>();
  for (const line of logs) {
    const audio = /Stream #0:(\d+)(?:\[[^\]]+\])?(?:\(([^)]+)\))?: Audio: ([^,\s]+).*?, (\d+) Hz, ([^,]+)/.exec(line);
    if (!audio) continue;
    const streamIndex = Number(audio[1]);
    if (streams.has(streamIndex)) continue;
    const channelLabel = audio[5].trim().toLowerCase();
    const channels = channelLabel === "mono" ? 1 : channelLabel === "stereo" ? 2 : channelLabel.startsWith("5.1") ? 6 : 0;
    streams.set(streamIndex, {
      index: streamIndex,
      codec_type: "audio",
      codec_name: audio[3],
      channels,
      sample_rate: audio[4],
      tags: { language: audio[2] ?? "" },
      disposition: { default: line.includes("(default)") ? 1 : 0 },
    });
  }
  return streams.size > 0 ? JSON.stringify({ format: { duration: null }, streams: [...streams.values()] }) : null;
}

export async function probeMedia(file: File): Promise<{ tracks: AudioTrack[]; normalizedJson: string }> {
  return exclusive(async () => {
    let engine = await ffmpeg();
    const { fetchFile } = await import("@ffmpeg/util");
    const inputName = `probe-${virtualInputName(file.name)}`;
    const outputName = "probe.json";
    const logs: string[] = [];
    const logHandler = ({ message }: { message: string }) => { logs.push(message); };
    engine.on("log", logHandler);
    try {
      await engine.writeFile(inputName, await fetchFile(file));
      const exitCode = await engine.ffprobe([
        "-v", "error", "-show_format", "-show_streams", "-of", "json",
        inputName, "-o", outputName,
      ]);
      let raw: string;
      if (exitCode === 0) {
        raw = bytesToText(await engine.readFile(outputName));
      } else {
        engine.off("log", logHandler);
        engine.terminate();
        ffmpegInstance = null;
        loaded = false;
        engine = await ffmpeg();
        engine.on("log", logHandler);
        await engine.writeFile(inputName, await fetchFile(file));
        logs.length = 0;
        const fallbackExit = await engine.exec(["-i", inputName, "-f", "null", "-"]);
        if (fallbackExit !== 0) {
          const detail = logs.slice(-3).join(" ").trim();
          throw new Error(`Could not inspect audio streams${detail ? `: ${detail}` : ""}`);
        }
        const fallback = fallbackProbeJson(logs);
        if (!fallback) throw new Error(`Could not parse audio streams: ${logs.slice(-12).join(" | ")}`);
        raw = fallback;
      }
      const normalizedJson = parse_probe_output(raw);
      const normalized = JSON.parse(normalizedJson) as { audio: Array<{
        index: number; codec: string; channels: number; sample_rate: number;
        language: string; title: string; is_default: boolean;
      }> };
      return {
        normalizedJson,
        tracks: normalized.audio.map((track) => ({
          index: track.index,
          codec: track.codec,
          channels: track.channels,
          sampleRate: track.sample_rate,
          language: track.language,
          title: track.title,
          isDefault: track.is_default,
        })),
      };
    } finally {
      engine.off("log", logHandler);
      await Promise.allSettled([engine.deleteFile(inputName), engine.deleteFile(outputName)]);
    }
  });
}

export async function prepareAudioPreview(
  clipId: number,
  file: File,
  trackIndex: number,
  volume: number,
  normalize: boolean,
): Promise<string> {
  const cacheKey = `${trackIndex}:${volume.toFixed(2)}:${normalize}`;
  const cached = audioPreviewCache.get(clipId)?.get(cacheKey);
  if (cached) return cached;

  return exclusive(async () => {
    const engine = await ffmpeg();
    const { fetchFile } = await import("@ffmpeg/util");
    const inputName = `audio-${clipId}-${virtualInputName(file.name)}`;
    const outputName = `audio-${clipId}-${trackIndex}.m4a`;
    const filters = [`volume=${Math.max(0, Math.min(2, volume))}`];
    if (normalize) filters.push("loudnorm=I=-16:TP=-1.5:LRA=11");
    try {
      await engine.writeFile(inputName, await fetchFile(file));
      const exitCode = await engine.exec([
        "-i", inputName, "-map", `0:a:${trackIndex}`, "-vn",
        "-af", filters.join(","), "-c:a", "aac", "-b:a", "128k", outputName,
      ]);
      if (exitCode !== 0) throw new Error(`Audio preparation exited with status ${exitCode}`);
      const data = await engine.readFile(outputName);
      if (typeof data === "string") throw new Error("The audio engine returned unexpected text output");
      const url = URL.createObjectURL(new Blob([new Uint8Array(data)], { type: "audio/mp4" }));
      let clipCache = audioPreviewCache.get(clipId);
      if (!clipCache) {
        clipCache = new Map();
        audioPreviewCache.set(clipId, clipCache);
      }
      clipCache.set(cacheKey, url);
      return url;
    } finally {
      await Promise.allSettled([engine.deleteFile(inputName), engine.deleteFile(outputName)]);
    }
  });
}

export function revokeAudioPreviews(clipId: number) {
  const cache = audioPreviewCache.get(clipId);
  if (!cache) return;
  for (const url of cache.values()) URL.revokeObjectURL(url);
  audioPreviewCache.delete(clipId);
}

export async function exportVideo(
  file: File,
  project: ClipForgeProject,
  callbacks: ExportCallbacks,
): Promise<string> {
  return exclusive(async () => {
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
  });
}

export function cancelExport() {
  ffmpegInstance?.terminate();
  ffmpegInstance = null;
  loaded = false;
  operationTail = Promise.resolve();
}
