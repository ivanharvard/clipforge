import type { EditorSettings, FrameRateLimit, VideoCodec } from "../../types";
import { Icon } from "../Icon";
import { Panel } from "../Panel";

interface CompressPanelProps {
  compression: EditorSettings["compression"];
  applyToAll: boolean;
  onChange: (compression: EditorSettings["compression"]) => void;
  onApplyToAllChange: (enabled: boolean) => void;
}

export function CompressPanel({ compression, applyToAll, onChange, onApplyToAllChange }: CompressPanelProps) {
  const update = (patch: Partial<EditorSettings["compression"]>) => onChange({
    ...compression,
    ...patch,
    mode: "target-size",
  });
  const minimumSize = compression.value * (1 - compression.tolerancePercent / 100);

  return (
    <Panel title="Compress" icon={<Icon name="archive" />}>
      <p className="panel-hint">Trim first, then encode to a predictable output size.</p>
      <label className="number-field">
        <span>Target size <output>{minimumSize.toFixed(1)}–{compression.value.toFixed(0)} MiB</output></span>
        <input aria-label="Target size in MiB" type="number" min="1" max="10000" step="1" value={compression.value} onChange={(event) => update({ value: event.currentTarget.valueAsNumber || 1 })} />
      </label>
      <label className="select-field compact-top">
        <span>Frame rate limit</span>
        <select value={compression.frameRateLimit} onChange={(event) => update({ frameRateLimit: event.currentTarget.value as FrameRateLimit })}>
          <option value="automatic">Automatic</option>
          <option value="30">30 FPS</option>
          <option value="60">60 FPS</option>
        </select>
      </label>
      <p className="field-hint">Automatic preserves the source frame rate.</p>
      <label className="select-field compact-top">
        <span>Codec</span>
        <select value={compression.codec} onChange={(event) => update({ codec: event.currentTarget.value as VideoCodec })}>
          <option value="h264">H.264</option>
          <option value="av1" disabled>AV1 (desktop only)</option>
        </select>
      </label>
      <label className="check-control compact-top"><input type="checkbox" checked={compression.extraQuality} onChange={(event) => update({ extraQuality: event.currentTarget.checked })} />Extra quality (slower)</label>
      <label className="number-field compact-top">
        <span>Below-target tolerance <output>{compression.tolerancePercent}%</output></span>
        <input aria-label="Below-target tolerance percent" type="number" min="0" max="100" step="1" value={compression.tolerancePercent} onChange={(event) => update({ tolerancePercent: Math.min(100, Math.max(0, event.currentTarget.valueAsNumber || 0)) })} />
      </label>
      <p className="field-hint">Allows the final file to be this much smaller than the target.</p>
      <label className="check-control compact-top"><input type="checkbox" checked={applyToAll} onChange={(event) => onApplyToAllChange(event.currentTarget.checked)} />Apply to all queued videos</label>
    </Panel>
  );
}
