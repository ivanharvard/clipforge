import { useRef, useState, type DragEvent, type KeyboardEvent, type PointerEvent, type RefObject } from "react";

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
  onCropChange: (crop: EditorSettings["crop"]) => void;
  playbackStatus?: string;
}

type CropHandle = "move" | "nw" | "ne" | "sw" | "se";

export function VideoStage({ clip, pendingUrl, settings, videoRef, onChooseFiles, onMetadata, onClose, onTimeUpdate, onCropChange, playbackStatus }: VideoStageProps) {
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
  const cropEnabled = settings?.pipeline.find((stage) => stage.kind === "crop")?.enabled ?? false;
  const transformEnabled = settings?.pipeline.find((stage) => stage.kind === "transform")?.enabled ?? false;
  const cropStyle = crop && clip ? {
    left: `${(crop.x / clip.width) * 100}%`,
    top: `${(crop.y / clip.height) * 100}%`,
    width: `${(crop.width / clip.width) * 100}%`,
    height: `${(crop.height / clip.height) * 100}%`,
  } : undefined;
  const transform = settings && transformEnabled
    ? `rotate(${settings.rotation}deg) scale(${settings.flipHorizontal ? -1 : 1}, ${settings.flipVertical ? -1 : 1})`
    : undefined;

  const startCropDrag = (event: PointerEvent<HTMLElement>, handle: CropHandle) => {
    if (!crop || !clip) return;
    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);
    const startX = event.clientX;
    const startY = event.clientY;
    const initial = { ...crop };
    const viewport = event.currentTarget.closest(".video-transform")?.getBoundingClientRect();
    if (!viewport?.width || !viewport.height) return;
    const scaleX = clip.width / viewport.width;
    const scaleY = clip.height / viewport.height;

    const move = (pointer: globalThis.PointerEvent) => {
      const dx = Math.round((pointer.clientX - startX) * scaleX);
      const dy = Math.round((pointer.clientY - startY) * scaleY);
      let next = { ...initial };
      if (handle === "move") {
        next.x = Math.max(0, Math.min(clip.width - next.width, initial.x + dx));
        next.y = Math.max(0, Math.min(clip.height - next.height, initial.y + dy));
      } else {
        const west = handle === "nw" || handle === "sw";
        const north = handle === "nw" || handle === "ne";
        const fixedX = west ? initial.x + initial.width : initial.x;
        const fixedY = north ? initial.y + initial.height : initial.y;
        const movingX = Math.max(0, Math.min(clip.width, west ? initial.x + dx : initial.x + initial.width + dx));
        let movingY = Math.max(0, Math.min(clip.height, north ? initial.y + dy : initial.y + initial.height + dy));
        if (initial.aspectLocked) {
          const ratio = initial.width / initial.height;
          const width = Math.max(2, Math.abs(fixedX - movingX));
          movingY = fixedY + (north ? -1 : 1) * Math.round(width / ratio);
          movingY = Math.max(0, Math.min(clip.height, movingY));
        }
        next.x = Math.min(fixedX, movingX);
        next.y = Math.min(fixedY, movingY);
        next.width = Math.max(2, Math.abs(fixedX - movingX));
        next.height = Math.max(2, Math.abs(fixedY - movingY));
      }
      onCropChange(next);
    };
    const finish = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", finish);
      window.removeEventListener("pointercancel", finish);
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", finish);
    window.addEventListener("pointercancel", finish);
  };

  const nudgeCrop = (event: KeyboardEvent<HTMLDivElement>) => {
    if (!crop || !clip || !["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(event.key)) return;
    event.preventDefault();
    const amount = event.shiftKey ? 10 : 1;
    const dx = event.key === "ArrowLeft" ? -amount : event.key === "ArrowRight" ? amount : 0;
    const dy = event.key === "ArrowUp" ? -amount : event.key === "ArrowDown" ? amount : 0;
    onCropChange({
      ...crop,
      x: Math.max(0, Math.min(clip.width - crop.width, crop.x + dx)),
      y: Math.max(0, Math.min(clip.height - crop.height, crop.y + dy)),
    });
  };

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
          {cropStyle && cropEnabled ? (
            <div className="crop-guide" style={cropStyle} role="group" aria-label="Crop area; use arrow keys to move" tabIndex={0} onKeyDown={nudgeCrop} onPointerDown={(event) => startCropDrag(event, "move")}>
              {(["nw", "ne", "sw", "se"] as CropHandle[]).map((handle) => (
                <button key={handle} type="button" className={`crop-handle ${handle}`} aria-label={`Resize crop from ${handle}`} onPointerDown={(event) => {
                  event.stopPropagation();
                  startCropDrag(event, handle);
                }} />
              ))}
            </div>
          ) : null}
          {playbackStatus ? <div className="video-preparing" role="status">{playbackStatus}</div> : null}
        </div>
      </div>
    </section>
  );
}
