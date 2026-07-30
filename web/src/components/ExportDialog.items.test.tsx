import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import type { ExportStatus } from "../types";
import { ExportDialog } from "./ExportDialog";

describe("per-video export progress", () => {
  it("renders a thumbnail and independent progress bar for every queued video", () => {
    const status: ExportStatus = {
      phase: "running",
      message: "Exporting 1 of 2",
      items: [
        { id: 1, name: "first.mp4", thumbnailUrl: "data:image/jpeg;base64,first", phase: "running", progress: 0.25, message: "Exporting locally…" },
        { id: 2, name: "second.mp4", thumbnailUrl: "data:image/jpeg;base64,second", phase: "pending", progress: 0, message: "Waiting" },
      ],
    };

    const markup = renderToStaticMarkup(
      <ExportDialog status={status} onClose={() => {}} onCancel={() => {}} />,
    );

    expect(markup.match(/class="export-thumbnail"/g)).toHaveLength(2);
    expect(markup.match(/role="progressbar"/g)).toHaveLength(2);
    expect(markup).toContain('aria-label="Export progress for first.mp4" aria-valuenow="25"');
    expect(markup).toContain('aria-label="Export progress for second.mp4" aria-valuenow="0"');
  });
});
