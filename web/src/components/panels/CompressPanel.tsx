import type { CompressionMode, EditorSettings, FrameRateLimit, VideoCodec } from "../../types";
import { Icon } from "../Icon";
import { Panel } from "../Panel";

interface CompressPanelProps {
  compression: EditorSettings["compression"];
  onChange: (compression: EditorSettings["compression"]) => void;
}

const valueConfig: Record<CompressionMode, { label: string; min: number; max: number; step: number }> = {
  crf: { label: "CRF", min: 0, max: 51, step: 1 },
  bitrate: { label: "Bitrate (kbps)", min: 64, max: 50_000, step: 100 },
  "target-size": { label: "Target size (MB)", min: 1, max: 10_000, step: 1 },
};

export function CompressPanel({ compression, onChange }: CompressPanelProps) {
  const update = (patch: Partial<EditorSettings["compression"]>) => onChange({ ...compression, ...patch });
  const changeMode = (mode: CompressionMode) => update({ mode, value: mode === "crf" ? 23 : mode === "bitrate" ? 2_500 : 10 });
  const config = valueConfig[compression.mode];

  return (
    <Panel title="Compress" icon={<Icon name="archive" />}>
      <div className="segment-control" role="group" aria-label="Compression mode">
        {(["crf", "bitrate", "target-size"] as const).map((mode) => (
          <button type="button" className={compression.mode === mode ? "active" : ""} onClick={() => changeMode(mode)} key={mode}>
            {mode === "target-size" ? "Size" : mode === "bitrate" ? "Bitrate" : "CRF"}
          </button>
        ))}
      </div>
      <label className="number-field"><span>{config.label}</span><input type="number" min={config.min} max={config.max} step={config.step} value={compression.value} onChange={(event) => update({ value: event.currentTarget.valueAsNumber || config.min })} /></label>
      <div className="field-grid compact-top">
        <label>Codec
          <select value={compression.codec} onChange={(event) => update({ codec: event.currentTarget.value as VideoCodec })}>
            <option value="h264">H.264</option>
            <option value="av1" disabled>AV1 (custom core)</option>
          </select>
        </label>
        <label>Frame rate
          <select value={compression.frameRateLimit} onChange={(event) => update({ frameRateLimit: event.currentTarget.value as FrameRateLimit })}>
            <option value="automatic">Auto</option>
            <option value="30">30 fps</option>
            <option value="60">60 fps</option>
          </select>
        </label>
      </div>
      <label className="check-control compact-top"><input type="checkbox" checked={compression.extraQuality} onChange={(event) => update({ extraQuality: event.currentTarget.checked })} />Slower, higher-quality encode</label>
    </Panel>
  );
}
