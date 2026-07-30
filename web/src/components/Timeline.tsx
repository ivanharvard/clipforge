import { useEffect, useState, type RefObject } from "react";

import { formatDuration, parseTimestamp } from "../lib/format";
import type { EditorSettings } from "../types";
import { Icon } from "./Icon";

interface TimelineProps {
  videoRef: RefObject<HTMLVideoElement | null>;
  durationMs: number;
  playheadMs: number;
  trim: EditorSettings["trim"];
  onTrimChange: (inMs: number, outMs: number) => void;
  disabled?: boolean;
}

function TimeField({ label, value, maximum, onCommit }: { label: string; value: number; maximum: number; onCommit: (value: number) => void }) {
  const [draft, setDraft] = useState(formatDuration(value));
  const [invalid, setInvalid] = useState(false);
  useEffect(() => setDraft(formatDuration(value)), [value]);

  const commit = () => {
    const parsed = parseTimestamp(draft);
    if (parsed === null) {
      setInvalid(true);
      setDraft(formatDuration(value));
      return;
    }
    setInvalid(false);
    onCommit(Math.max(0, Math.min(maximum, parsed)));
  };

  return (
    <label className={`timeline-time-field${invalid ? " invalid" : ""}`}>
      <span>{label}</span>
      <input aria-invalid={invalid} inputMode="decimal" value={draft} onChange={(event) => setDraft(event.currentTarget.value)} onBlur={commit} onKeyDown={(event) => {
        if (event.key === "Enter") event.currentTarget.blur();
        if (event.key === "Escape") {
          setDraft(formatDuration(value));
          setInvalid(false);
          event.currentTarget.blur();
        }
      }} />
    </label>
  );
}

export function Timeline({ videoRef, durationMs, playheadMs, trim, onTrimChange, disabled = false }: TimelineProps) {
  const [playing, setPlaying] = useState(false);
  const safeDuration = Math.max(1, durationMs);
  const selectionLeft = (trim.inMs / safeDuration) * 100;
  const selectionWidth = ((trim.outMs - trim.inMs) / safeDuration) * 100;
  const playheadLeft = (playheadMs / safeDuration) * 100;

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
      <div className="timeline-transport-row">
        <button className="transport-button" type="button" disabled={disabled} onClick={togglePlayback} aria-label={playing ? "Pause" : "Play"}>
          <Icon name={playing ? "pause" : "play"} />
        </button>
        <time>{formatDuration(playheadMs)}</time>
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
      </div>
      <div className="timeline-fields-row">
        <TimeField label="Start" value={trim.inMs} maximum={Math.max(0, trim.outMs - 1)} onCommit={(value) => onTrimChange(value, trim.outMs)} />
        <TimeField label="End" value={trim.outMs} maximum={durationMs} onCommit={(value) => onTrimChange(trim.inMs, Math.max(trim.inMs + 1, value))} />
      </div>
    </section>
  );
}
