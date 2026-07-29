export type ResolutionPreset = "original" | "1080p" | "720p" | "480p" | "custom";
export type CompressionMode = "crf" | "bitrate" | "target-size";
export type FrameRateLimit = "automatic" | "30" | "60";
export type VideoCodec = "h264" | "av1";

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
    audio: { volume: 1, muted: false, normalize: false },
    compression: {
      mode: "target-size",
      value: 10,
      frameRateLimit: "automatic",
      codec: "h264",
      extraQuality: false,
      tolerancePercent: 25,
    },
    trim: { inMs: 0, outMs: durationMs },
  };
}
