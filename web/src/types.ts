export type ResolutionPreset = "original" | "1080p" | "720p" | "480p" | "custom";
export type CompressionMode = "crf" | "bitrate" | "target-size";
export type FrameRateLimit = "automatic" | "30" | "60";
export type VideoCodec = "h264" | "av1";
export type ToolKind = "compress" | "transform" | "crop" | "resolution" | "audio";

export interface ToolStage {
  kind: ToolKind;
  enabled: boolean;
}

export interface AudioTrack {
  index: number;
  codec: string;
  channels: number;
  sampleRate: number;
  language: string;
  title: string;
  isDefault: boolean;
}

export const defaultPipeline = (): ToolStage[] => [
  { kind: "compress", enabled: true },
  { kind: "transform", enabled: true },
  { kind: "crop", enabled: true },
  { kind: "resolution", enabled: true },
  { kind: "audio", enabled: true },
];

export interface ClipSource {
  file: File;
  url: string;
  durationMs: number;
  width: number;
  height: number;
}

export interface EditorSettings {
  rotation: number;
  flipHorizontal: boolean;
  flipVertical: boolean;
  crop: {
    x: number;
    y: number;
    width: number;
    height: number;
    aspectLocked: boolean;
  };
  resolution: {
    preset: ResolutionPreset;
    customWidth: number;
    customHeight: number;
    aspectLocked: boolean;
  };
  audio: {
    volume: number;
    muted: boolean;
    normalize: boolean;
    trackIndex: number;
    tracks: AudioTrack[];
  };
  compression: {
    mode: CompressionMode;
    value: number;
    frameRateLimit: FrameRateLimit;
    codec: VideoCodec;
    extraQuality: boolean;
    tolerancePercent: number;
  };
  trim: {
    inMs: number;
    outMs: number;
  };
  pipeline: ToolStage[];
}

export type ExportPhase = "idle" | "loading" | "running" | "success" | "error";

export interface ExportStatus {
  phase: ExportPhase;
  progress: number;
  message: string;
}

export function defaultSettings(width: number, height: number, durationMs: number): EditorSettings {
  return {
    rotation: 0,
    flipHorizontal: false,
    flipVertical: false,
    crop: { x: 0, y: 0, width, height, aspectLocked: false },
    resolution: {
      preset: "original",
      customWidth: width,
      customHeight: height,
      aspectLocked: true,
    },
    audio: { volume: 1, muted: false, normalize: false, trackIndex: 0, tracks: [] },
    compression: {
      mode: "target-size",
      value: 10,
      frameRateLimit: "automatic",
      codec: "h264",
      extraQuality: false,
      tolerancePercent: 25,
    },
    trim: { inMs: 0, outMs: durationMs },
    pipeline: defaultPipeline(),
  };
}
