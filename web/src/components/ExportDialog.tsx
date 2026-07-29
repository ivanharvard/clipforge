import type { ExportStatus } from "../types";
import { Icon } from "./Icon";

interface ExportDialogProps {
  status: ExportStatus;
  onClose: () => void;
  onCancel: () => void;
}

export function ExportDialog({ status, onClose, onCancel }: ExportDialogProps) {
  if (status.phase === "idle") return null;
  const running = status.phase === "loading" || status.phase === "running";

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
        <div className="dialog-actions">
          {running ? <button className="button button-quiet" type="button" onClick={onCancel}>Cancel</button> : <button className="button button-primary" type="button" onClick={onClose}>Close</button>}
        </div>
      </section>
    </div>
  );
}
