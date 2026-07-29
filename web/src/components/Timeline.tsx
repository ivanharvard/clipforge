import { useEffect, useState, type RefObject } from "react";

import { formatDuration } from "../lib/format";
import type { EditorSettings } from "../types";
import { Icon } from "./Icon";

interface TimelineProps {
  videoRef: RefObject<HTMLVideoElement | null>;
  durationMs: number;
  playheadMs: number;
  trim: EditorSettings["trim"];
  onTrimChange: (inMs: number, outMs: number) => void;
}

export function Timeline({ videoRef, durationMs, playheadMs, trim, onTrimChange }: TimelineProps) {
  const [playing, setPlaying] = useState(false);
  const selectionLeft = (trim.inMs / durationMs) * 100;
  const selectionWidth = ((trim.outMs - trim.inMs) / durationMs) * 100;
  const playheadLeft = (playheadMs / durationMs) * 100;

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    const handlePlay = () => setPlaying(true);
    const handlePause = () => setPlaying(false);
    video.addEventListener("play", handlePlay);
    video.addEventListener("pause", handlePause);
    return () => {
      video.removeEventListener("play", handlePlay);
      video.removeEventListener("pause", handlePause);
    };
  }, [videoRef]);

  const togglePlayback = async () => {
    const current = videoRef.current;
    if (!current) return;
    if (current.paused) await current.play();
    else current.pause();
  };

  const seek = (milliseconds: number) => {
    if (videoRef.current) videoRef.current.currentTime = milliseconds / 1000;
  };

  return (
    <section className="timeline" aria-label="Clip timeline">
      <button className="transport-button" type="button" onClick={togglePlayback} aria-label={playing ? "Pause" : "Play"}>
        <Icon name={playing ? "pause" : "play"} />
      </button>
      <time>{formatDuration(playheadMs)}</time>
      <label className="time-field">In<input type="number" min="0" max={trim.outMs - 1} value={trim.inMs} onChange={(event) => onTrimChange(Math.max(0, event.currentTarget.valueAsNumber || 0), trim.outMs)} /></label>
      <div className="trim-track">
        <div className="selection" style={{ left: `${selectionLeft}%`, width: `${selectionWidth}%` }} />
        <div className="playhead" style={{ left: `${playheadLeft}%` }} />
        <input aria-label="Trim start" type="range" min="0" max={durationMs} step="10" value={trim.inMs} onChange={(event) => {
          const next = Math.min(event.currentTarget.valueAsNumber, trim.outMs - 1);
          onTrimChange(next, trim.outMs);
          seek(next);
        }} />
        <input aria-label="Trim end" type="range" min="0" max={durationMs} step="10" value={trim.outMs} onChange={(event) => {
          const next = Math.max(event.currentTarget.valueAsNumber, trim.inMs + 1);
          onTrimChange(trim.inMs, next);
          seek(next);
        }} />
      </div>
      <label className="time-field">Out<input type="number" min={trim.inMs + 1} max={durationMs} value={trim.outMs} onChange={(event) => onTrimChange(trim.inMs, Math.min(durationMs, event.currentTarget.valueAsNumber || durationMs))} /></label>
      <time>{formatDuration(durationMs)}</time>
    </section>
  );
}
