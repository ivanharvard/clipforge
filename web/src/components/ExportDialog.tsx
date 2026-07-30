import { useEffect, useState } from "react";

import { DESKTOP_DOWNLOAD_URL } from "../lib/download";
import type { ExportStatus } from "../types";
import { Icon } from "./Icon";

const SLOW_EXPORT_DELAY_MS = 10_000;

interface ExportDialogProps {
  status: ExportStatus;
  onClose: () => void;
  onCancel: () => void;
}

export function ExportDialog({ status, onClose, onCancel }: ExportDialogProps) {
  const running = status.phase === "loading" || status.phase === "running";
  const [showDesktopPrompt, setShowDesktopPrompt] = useState(false);

  useEffect(() => {
    if (!running) {
      setShowDesktopPrompt(false);
      return;
    }

    const timer = window.setTimeout(() => setShowDesktopPrompt(true), SLOW_EXPORT_DELAY_MS);
    return () => window.clearTimeout(timer);
  }, [running]);

  if (status.phase === "idle") return null;

  return (
    <div className="dialog-backdrop" role="presentation">
      <section className="export-dialog" role="dialog" aria-modal="true" aria-labelledby="export-title">
        <div className={`dialog-status ${status.phase}`}>
          {status.phase === "success" ? <Icon name="check" /> : status.phase === "error" ? <Icon name="x" /> : <Icon name="archive" />}
        </div>
        <h2 id="export-title">{status.phase === "success" ? "Export complete" : status.phase === "error" ? "Export failed" : "Exporting clip"}</h2>
        <p>{status.message}</p>
        <div className="progress-track" aria-label="Export progress" aria-valuenow={Math.round(status.progress * 100)} role="progressbar">
          <span style={{ width: `${status.progress * 100}%` }} />
        </div>
        {running && showDesktopPrompt ? (
          <aside className="slow-export-prompt" aria-label="Faster desktop export">
            <strong>Taking too long?</strong>
            <span>Install ClipForge to export faster with your computer's native media tools.</span>
            <a className="button button-primary" href={DESKTOP_DOWNLOAD_URL} target="_blank" rel="noreferrer">
              <Icon name="download" /> Download ClipForge
            </a>
          </aside>
        ) : null}
        <div className="dialog-actions">
          {running ? <button className="button button-quiet" type="button" onClick={onCancel}>Cancel</button> : <button className="button button-primary" type="button" onClick={onClose}>Close</button>}
        </div>
      </section>
    </div>
  );
}
