import { useEffect, useRef, useState } from "react";

import { ExportDialog } from "./components/ExportDialog";
import { Header } from "./components/Header";
import { QueueBar } from "./components/QueueBar";
import { SettingsSidebar } from "./components/SettingsSidebar";
import { Timeline } from "./components/Timeline";
import { VideoStage } from "./components/VideoStage";
import { useClipEditor } from "./hooks/useClipEditor";
import { initializeBindings } from "./lib/wasm";
import { cancelExport, exportVideo } from "./media/exportVideo";
import type { ExportStatus } from "./types";

const idleExport: ExportStatus = { phase: "idle", progress: 0, message: "" };

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
  const editor = useClipEditor();

  useEffect(() => {
    initializeBindings()
      .then(() => setBindingsReady(true))
      .catch((error: unknown) => setBindingError(error instanceof Error ? error.message : "Could not load the editor"));
  }, []);

  useEffect(() => {
    const video = videoRef.current;
    const audio = editor.settings?.audio;
    if (!video || !audio) return;
    video.volume = Math.min(1, audio.volume);
    video.muted = audio.muted;
  }, [editor.settings?.audio]);

  useEffect(() => setPlayheadMs(0), [editor.activeIndex]);

  const startExport = async () => {
    if (editor.queue.length === 0) return;
    exportCancelledRef.current = false;
    setExportStatus({ phase: "loading", progress: 0, message: "Preparing the video queue…" });
    try {
      const jobs = await editor.prepareQueue();
      for (let index = 0; index < jobs.length; index += 1) {
        const job = jobs[index];
        await exportVideo(job.file, job.project, {
          onPhase: (message) => {
            if (!exportCancelledRef.current) {
              setExportStatus((current) => ({
                ...current,
                phase: message.startsWith("Loading") ? "loading" : "running",
                message: `${job.file.name} · ${index + 1} of ${jobs.length} · ${message}`,
              }));
            }
          },
          onProgress: (progress) => {
            if (!exportCancelledRef.current) {
              setExportStatus((current) => ({ ...current, progress: (index + progress) / jobs.length }));
            }
          },
        });
      }
      if (!exportCancelledRef.current) {
        setExportStatus({ phase: "success", progress: 1, message: jobs.length === 1 ? "Your video was saved to downloads." : `${jobs.length} videos were saved to downloads.` });
      }
    } catch (error) {
      if (exportCancelledRef.current) return;
      setExportStatus({ phase: "error", progress: 0, message: errorMessage(error) });
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
      <Header />
      <main className="workspace">
        {bindingError ? <div className="notice error">{bindingError}</div> : null}
        {!bindingsReady && !bindingError ? <div className="notice">Loading ClipForge…</div> : null}

        <QueueBar items={editor.queue} activeIndex={editor.activeIndex} exporting={exporting} onAddFiles={editor.addFiles} onSelect={editor.activateClip} onRemove={editor.removeActiveClip} onExport={startExport} />

        <div className={`editor-shell${loaded ? " has-clip" : ""}`}>
          <VideoStage clip={editor.clip} pendingUrl={editor.pendingUrl} settings={editor.settings} videoRef={videoRef} onChooseFiles={editor.addFiles} onMetadata={editor.loadMetadata} onClose={editor.removeActiveClip} onTimeUpdate={setPlayheadMs} />
          {loaded ? (
            <SettingsSidebar settings={loaded.settings} sourceWidth={loaded.clip.width} sourceHeight={loaded.clip.height} onRotationChange={editor.setRotation} onFlipsChange={editor.setFlips} onCropChange={editor.setCrop} onResolutionChange={editor.setResolution} onAudioChange={editor.setAudio} onCompressionChange={editor.setCompression} compressionApplyAll={editor.compressionApplyAll} onCompressionApplyAllChange={editor.setCompressionApplyAll} />
          ) : (
            <aside className="intro-sidebar">
              <div><span>01</span><p><strong>Edit without uploading</strong>Your source video never leaves this browser.</p></div>
              <div><span>02</span><p><strong>Use only what you need</strong>Trim, crop, resize, rotate, and compress.</p></div>
              <div><span>03</span><p><strong>Export directly</strong>The finished MP4 downloads to your device.</p></div>
            </aside>
          )}
          {loaded ? <Timeline videoRef={videoRef} durationMs={loaded.clip.durationMs} playheadMs={playheadMs} trim={loaded.settings.trim} onTrimChange={editor.setTrim} /> : null}
        </div>
      </main>
      <footer><span>ClipForge Web · Runs locally</span><a href="https://github.com/ivanharvard/clipforge" target="_blank" rel="noreferrer">Source on GitHub</a></footer>
      <ExportDialog status={exportStatus} onCancel={cancel} onClose={() => setExportStatus(idleExport)} />
    </div>
  );
}
