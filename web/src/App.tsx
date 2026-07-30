import { useEffect, useRef, useState } from "react";

import { ExportDialog } from "./components/ExportDialog";
import { Header } from "./components/Header";
import { QueueBar } from "./components/QueueBar";
import { SettingsSidebar } from "./components/SettingsSidebar";
import { Timeline } from "./components/Timeline";
import { VideoStage } from "./components/VideoStage";
import { useClipEditor } from "./hooks/useClipEditor";
import { initializeBindings } from "./lib/wasm";
import { audioNeedsResync } from "./media/audioSync";
import { cancelExport, exportVideo, prepareAudioPreview, probeMedia } from "./media/exportVideo";
import type { ExportItemStatus, ExportStatus } from "./types";

const idleExport: ExportStatus = { phase: "idle", message: "", items: [] };

function updateExportItem(items: ExportItemStatus[], index: number, update: Partial<ExportItemStatus>) {
  return items.map((item, itemIndex) => itemIndex === index ? { ...item, ...update } : item);
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  if (error instanceof Event) return `${error.type} while loading the local video engine`;
  try {
    return JSON.stringify(error);
  } catch {
    return "The browser could not export this clip.";
  }
}

export function App() {
  const [bindingsReady, setBindingsReady] = useState(false);
  const [bindingError, setBindingError] = useState("");
  const [playheadMs, setPlayheadMs] = useState(0);
  const [exportStatus, setExportStatus] = useState<ExportStatus>(idleExport);
  const exportCancelledRef = useRef(false);
  const videoRef = useRef<HTMLVideoElement>(null);
  const audioRef = useRef<HTMLAudioElement>(null);
  const probedClipsRef = useRef(new Set<number>());
  const lastWorkingTrackRef = useRef(0);
  const [audioPreviewUrl, setAudioPreviewUrl] = useState<string | null>(null);
  const [audioStatus, setAudioStatus] = useState("");
  const [audioError, setAudioError] = useState("");
  const editor = useClipEditor();
  const activeEditorReady = editor.settings !== null;

  useEffect(() => {
    initializeBindings()
      .then(() => setBindingsReady(true))
      .catch((error: unknown) => setBindingError(error instanceof Error ? error.message : "Could not load the editor"));
  }, []);

  useEffect(() => {
    const video = videoRef.current;
    const audio = editor.settings?.audio;
    if (!video || !audio) return;
    video.volume = audioPreviewUrl ? 1 : Math.min(1, audio.volume);
    video.muted = audio.muted || Boolean(audioPreviewUrl);
    if (audioRef.current) audioRef.current.muted = audio.muted;
  }, [editor.settings?.audio, audioPreviewUrl]);

  useEffect(() => {
    const clipId = editor.activeId;
    const file = editor.activeFile;
    if (!bindingsReady || clipId === null || !file || !editor.settings || probedClipsRef.current.has(clipId)) return;
    probedClipsRef.current.add(clipId);
    let cancelled = false;
    setAudioStatus("Reading audio tracks…");
    probeMedia(file).then(({ tracks, normalizedJson }) => {
      if (!cancelled) editor.setAudioTracks(tracks, normalizedJson);
    }).catch((error: unknown) => {
      if (!cancelled) setAudioError(errorMessage(error));
    }).finally(() => {
      if (!cancelled) setAudioStatus("");
    });
    return () => { cancelled = true; };
  }, [activeEditorReady, bindingsReady, editor.activeFile, editor.activeId, editor.setAudioTracks]);

  useEffect(() => {
    const audio = editor.settings?.audio;
    const clipId = editor.activeId;
    const file = editor.activeFile;
    if (!audio || clipId === null || !file || (audio.tracks.length < 2 && !audio.normalize)) {
      setAudioPreviewUrl(null);
      return;
    }
    let cancelled = false;
    videoRef.current?.pause();
    setAudioError("");
    setAudioStatus("Preparing audio track…");
    const debounce = window.setTimeout(() => {
      prepareAudioPreview(clipId, file, audio.trackIndex, audio.volume, audio.normalize).then((url) => {
        if (cancelled) return;
        lastWorkingTrackRef.current = audio.trackIndex;
        setAudioPreviewUrl(url);
      }).catch((error: unknown) => {
        if (cancelled) return;
        setAudioError(errorMessage(error));
        if (audio.trackIndex !== lastWorkingTrackRef.current) {
          editor.setAudio({ ...audio, trackIndex: lastWorkingTrackRef.current });
        }
      }).finally(() => {
        if (!cancelled) setAudioStatus("");
      });
    }, 300);
    return () => {
      cancelled = true;
      window.clearTimeout(debounce);
    };
  }, [editor.activeFile, editor.activeId, editor.settings?.audio, editor.setAudio]);

  useEffect(() => {
    const video = videoRef.current;
    const audio = audioRef.current;
    if (!video || !audio || !audioPreviewUrl) return;
    const synchronize = () => {
      if (audioNeedsResync(video.currentTime, audio.currentTime)) audio.currentTime = video.currentTime;
    };
    const play = () => { synchronize(); void audio.play(); };
    const pause = () => audio.pause();
    const seek = () => { audio.currentTime = video.currentTime; };
    const rate = () => { audio.playbackRate = video.playbackRate; };
    video.addEventListener("play", play);
    video.addEventListener("pause", pause);
    video.addEventListener("seeking", seek);
    video.addEventListener("timeupdate", synchronize);
    video.addEventListener("ratechange", rate);
    return () => {
      video.removeEventListener("play", play);
      video.removeEventListener("pause", pause);
      video.removeEventListener("seeking", seek);
      video.removeEventListener("timeupdate", synchronize);
      video.removeEventListener("ratechange", rate);
      audio.pause();
    };
  }, [audioPreviewUrl]);

  useEffect(() => setPlayheadMs(0), [editor.activeIndex]);

  const startExport = async () => {
    if (editor.queue.length === 0) return;
    exportCancelledRef.current = false;
    setExportStatus({
      phase: "loading",
      message: "Preparing the video queue…",
      items: editor.queue.map((item) => ({
        id: item.id,
        name: item.name,
        thumbnailUrl: item.thumbnailUrl,
        phase: "pending",
        progress: 0,
        message: "Waiting",
      })),
    });
    let activeExportIndex = -1;
    try {
      const jobs = await editor.prepareQueue();
      if (exportCancelledRef.current) return;
      setExportStatus((current) => ({
        ...current,
        items: jobs.map((job) => ({
          id: job.id,
          name: job.file.name,
          thumbnailUrl: job.thumbnailUrl,
          phase: "pending",
          progress: 0,
          message: "Waiting",
        })),
      }));
      for (let index = 0; index < jobs.length; index += 1) {
        if (exportCancelledRef.current) return;
        const job = jobs[index];
        activeExportIndex = index;
        setExportStatus((current) => ({
          ...current,
          items: updateExportItem(current.items, index, {
            phase: "loading",
            message: "Preparing",
          }),
        }));
        await exportVideo(job.file, job.project, {
          onPhase: (message) => {
            if (!exportCancelledRef.current) {
              setExportStatus((current) => ({
                ...current,
                phase: message.startsWith("Loading") ? "loading" : "running",
                message: `Exporting ${index + 1} of ${jobs.length}`,
                items: updateExportItem(current.items, index, {
                  phase: message.startsWith("Loading") ? "loading" : "running",
                  message,
                }),
              }));
            }
          },
          onProgress: (progress) => {
            if (!exportCancelledRef.current) {
              setExportStatus((current) => ({
                ...current,
                items: updateExportItem(current.items, index, { progress }),
              }));
            }
          },
        });
        if (exportCancelledRef.current) return;
        setExportStatus((current) => ({
          ...current,
          items: updateExportItem(current.items, index, {
            phase: "success",
            progress: 1,
            message: "Saved to downloads",
          }),
        }));
      }
      if (!exportCancelledRef.current) {
        setExportStatus((current) => ({
          ...current,
          phase: "success",
          message: jobs.length === 1 ? "Your video was saved to downloads." : `${jobs.length} videos were saved to downloads.`,
        }));
      }
    } catch (error) {
      if (exportCancelledRef.current) return;
      const message = errorMessage(error);
      setExportStatus((current) => ({
        ...current,
        phase: "error",
        message,
        items: activeExportIndex < 0 ? current.items : updateExportItem(current.items, activeExportIndex, {
          phase: "error",
          message,
        }),
      }));
    }
  };

  const cancel = () => {
    exportCancelledRef.current = true;
    cancelExport();
    setExportStatus(idleExport);
  };

  const loaded = editor.clip && editor.settings
    ? { clip: editor.clip, settings: editor.settings }
    : null;
  const exporting = exportStatus.phase === "loading" || exportStatus.phase === "running";

  return (
    <div className="app" id="top">
      <Header canUndo={editor.canUndo} canRedo={editor.canRedo} onUndo={editor.undo} onRedo={editor.redo} />
      <main className="workspace">
        {bindingError ? <div className="notice error">{bindingError}</div> : null}
        {audioError ? <div className="notice error">Audio preview: {audioError}</div> : null}
        {!bindingsReady && !bindingError ? <div className="notice">Loading ClipForge…</div> : null}

        <QueueBar items={editor.queue} activeIndex={editor.activeIndex} exporting={exporting} onAddFiles={editor.addFiles} onSelect={editor.activateClip} onRemove={editor.removeActiveClip} onExport={startExport} />

        <div className={`editor-shell${loaded ? " has-clip" : ""}`}>
          <VideoStage clip={editor.clip} pendingUrl={editor.pendingUrl} settings={editor.settings} videoRef={videoRef} onChooseFiles={editor.addFiles} onMetadata={editor.loadMetadata} onClose={editor.removeActiveClip} onTimeUpdate={setPlayheadMs} onCropChange={editor.setCrop} playbackStatus={audioStatus === "Preparing audio track…" ? audioStatus : undefined} />
          {loaded ? (
            <SettingsSidebar settings={loaded.settings} sourceWidth={loaded.clip.width} sourceHeight={loaded.clip.height} onRotationChange={editor.setRotation} onFlipsChange={editor.setFlips} onCropChange={editor.setCrop} onResolutionChange={editor.setResolution} onAudioChange={editor.setAudio} onCompressionChange={editor.setCompression} compressionApplyAll={editor.compressionApplyAll} onCompressionApplyAllChange={editor.setCompressionApplyAll} onToolEnabledChange={editor.setToolEnabled} onToolMove={editor.moveTool} audioStatus={audioStatus} />
          ) : (
            <aside className="intro-sidebar">
              <div><span>01</span><p><strong>Edit without uploading</strong>Your source video never leaves this browser.</p></div>
              <div><span>02</span><p><strong>Use only what you need</strong>Trim, crop, resize, rotate, and compress.</p></div>
              <div><span>03</span><p><strong>Export directly</strong>The finished MP4 downloads to your device.</p></div>
            </aside>
          )}
          {loaded ? <Timeline videoRef={videoRef} durationMs={loaded.clip.durationMs} playheadMs={playheadMs} trim={loaded.settings.trim} onTrimChange={editor.setTrim} disabled={audioStatus === "Preparing audio track…"} /> : null}
        </div>
      </main>
      <audio ref={audioRef} src={audioPreviewUrl ?? undefined} preload="auto" className="visually-hidden" />
      <footer><span>ClipForge Web · Runs locally</span><a href="https://github.com/ivanharvard/clipforge" target="_blank" rel="noreferrer">Source on GitHub</a></footer>
      <ExportDialog status={exportStatus} onCancel={cancel} onClose={() => setExportStatus(idleExport)} />
    </div>
  );
}
