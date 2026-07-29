import { useRef, useState, type DragEvent, type RefObject } from "react";

import type { ClipSource, EditorSettings } from "../types";
import { fileSize } from "../lib/format";
import { Icon } from "./Icon";

interface VideoStageProps {
  clip: ClipSource | null;
  pendingUrl: string | null;
  settings: EditorSettings | null;
  videoRef: RefObject<HTMLVideoElement | null>;
  onChooseFiles: (files: File[]) => void;
  onMetadata: (metadata: { durationMs: number; width: number; height: number }) => void;
  onClose: () => void;
  onTimeUpdate: (milliseconds: number) => void;
}

export function VideoStage({ clip, pendingUrl, settings, videoRef, onChooseFiles, onMetadata, onClose, onTimeUpdate }: VideoStageProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [dragging, setDragging] = useState(false);
  const sourceUrl = clip?.url ?? pendingUrl;

  const isVideoFile = (file: File) =>
    file.type.startsWith("video/") || /\.(avi|m4v|mkv|mov|mp4|webm)$/i.test(file.name);

  const acceptDrop = (event: DragEvent) => {
    event.preventDefault();
    setDragging(false);
    const files = Array.from(event.dataTransfer.files).filter(isVideoFile);
    onChooseFiles(files);
  };

  if (!sourceUrl) {
    return (
      <section className={`video-stage empty${dragging ? " dragging" : ""}`} onDragEnter={() => setDragging(true)} onDragLeave={() => setDragging(false)} onDragOver={(event) => event.preventDefault()} onDrop={acceptDrop}>
        <button className="empty-action" type="button" onClick={() => inputRef.current?.click()}>
          <span className="empty-icon"><Icon name="file" /></span>
          <strong>Open a clip to begin</strong>
          <span>Drop a video here or choose one from your device</span>
          <span className="button button-primary">Choose video</span>
        </button>
        <input ref={inputRef} className="visually-hidden" aria-label="Choose videos" type="file" accept="video/*,.mkv,.avi,.mov,.m4v" multiple onChange={(event) => {
          const files = Array.from(event.currentTarget.files ?? []).filter(isVideoFile);
          onChooseFiles(files);
          event.currentTarget.value = "";
        }} />
      </section>
    );
  }

  const crop = settings?.crop;
  const cropStyle = crop && clip ? {
    left: `${(crop.x / clip.width) * 100}%`,
    top: `${(crop.y / clip.height) * 100}%`,
    width: `${(crop.width / clip.width) * 100}%`,
    height: `${(crop.height / clip.height) * 100}%`,
  } : undefined;
  const transform = settings
    ? `rotate(${settings.rotation}deg) scale(${settings.flipHorizontal ? -1 : 1}, ${settings.flipVertical ? -1 : 1})`
    : undefined;

  return (
    <section className="video-stage loaded">
      <div className="video-toolbar">
        <div>
          <strong>{clip?.file.name ?? "Reading video…"}</strong>
          {clip ? <span>{clip.width}×{clip.height} · {fileSize(clip.file.size)}</span> : null}
        </div>
        <button className="icon-button" type="button" onClick={onClose} aria-label="Close video"><Icon name="x" /></button>
      </div>
      <div className="video-viewport">
        <div className="video-transform" style={{ transform }}>
          <video
            ref={videoRef}
            src={sourceUrl}
            playsInline
            preload="metadata"
            onLoadedMetadata={(event) => {
              const video = event.currentTarget;
              onMetadata({ durationMs: Math.round(video.duration * 1000), width: video.videoWidth, height: video.videoHeight });
            }}
            onTimeUpdate={(event) => onTimeUpdate(Math.round(event.currentTarget.currentTime * 1000))}
          />
          {cropStyle ? <div className="crop-guide" style={cropStyle}><span /><span /><span /><span /></div> : null}
        </div>
      </div>
    </section>
  );
}
