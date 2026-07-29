import type { EditorSettings } from "../../types";
import { Icon } from "../Icon";
import { Panel } from "../Panel";

interface AudioPanelProps {
  audio: EditorSettings["audio"];
  onChange: (audio: EditorSettings["audio"]) => void;
}

export function AudioPanel({ audio, onChange }: AudioPanelProps) {
  const update = (patch: Partial<EditorSettings["audio"]>) => onChange({ ...audio, ...patch });

  return (
    <Panel title="Audio" icon={<Icon name="volume" />}>
      <label className="range-field">
        <span>Volume <output>{Math.round(audio.volume * 100)}%</output></span>
        <input type="range" min="0" max="2" step="0.01" value={audio.volume} disabled={audio.muted} onChange={(event) => update({ volume: event.currentTarget.valueAsNumber })} />
      </label>
      <div className="panel-footer">
        <label className="check-control"><input type="checkbox" checked={audio.muted} onChange={(event) => update({ muted: event.currentTarget.checked })} />Mute</label>
      </div>
    </Panel>
  );
}
