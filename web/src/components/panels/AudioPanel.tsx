import type { EditorSettings } from "../../types";
import { Icon } from "../Icon";
import { Panel } from "../Panel";

interface AudioPanelProps {
  audio: EditorSettings["audio"];
  onChange: (audio: EditorSettings["audio"]) => void;
  status?: string;
}

export function AudioPanel({ audio, onChange, status }: AudioPanelProps) {
  const update = (patch: Partial<EditorSettings["audio"]>) => onChange({ ...audio, ...patch });

  return (
    <Panel title="Audio" icon={<Icon name="volume" />}>
      <label className="range-field">
        <span>Volume <output>{Math.round(audio.volume * 100)}%</output></span>
        <input type="range" min="0" max="2" step="0.01" value={audio.volume} disabled={audio.muted} onChange={(event) => update({ volume: event.currentTarget.valueAsNumber })} />
      </label>
      <label className="select-field compact-top">
        <span>Audio track</span>
        <select value={audio.trackIndex} disabled={audio.tracks.length < 2} onChange={(event) => update({ trackIndex: Number(event.currentTarget.value) })}>
          {audio.tracks.length === 0 ? <option value={0}>Default track</option> : audio.tracks.map((track, ordinal) => (
            <option value={ordinal} key={track.index}>
              Track {ordinal + 1} · {track.title || track.language || track.codec.toUpperCase()} · {track.channels} ch
            </option>
          ))}
        </select>
      </label>
      {status ? <p className="field-hint audio-status" role="status">{status}</p> : null}
      <div className="panel-footer">
        <label className="check-control"><input type="checkbox" checked={audio.muted} onChange={(event) => update({ muted: event.currentTarget.checked })} />Mute</label>
        <label className="check-control"><input type="checkbox" checked={audio.normalize} onChange={(event) => update({ normalize: event.currentTarget.checked })} />Normalize</label>
      </div>
    </Panel>
  );
}
