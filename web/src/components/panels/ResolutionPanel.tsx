import type { EditorSettings, ResolutionPreset } from "../../types";
import { Icon } from "../Icon";
import { Panel } from "../Panel";

interface ResolutionPanelProps {
  resolution: EditorSettings["resolution"];
  onChange: (resolution: EditorSettings["resolution"]) => void;
}

export function ResolutionPanel({ resolution, onChange }: ResolutionPanelProps) {
  const update = (patch: Partial<EditorSettings["resolution"]>) => onChange({ ...resolution, ...patch });

  return (
    <Panel title="Resolution" icon={<Icon name="archive" />}>
      <label className="select-field">
        <span>Output size</span>
        <select value={resolution.preset} onChange={(event) => update({ preset: event.currentTarget.value as ResolutionPreset })}>
          <option value="original">Keep current</option>
          <option value="1080p">1080p</option>
          <option value="720p">720p</option>
          <option value="480p">480p</option>
          <option value="custom">Custom</option>
        </select>
      </label>
      {resolution.preset === "custom" ? (
        <div className="field-grid compact-top">
          <label>Width <input type="number" min="2" value={resolution.customWidth} onChange={(event) => update({ customWidth: event.currentTarget.valueAsNumber || 2 })} /></label>
          <label>Height <input type="number" min="2" value={resolution.customHeight} onChange={(event) => update({ customHeight: event.currentTarget.valueAsNumber || 2 })} /></label>
        </div>
      ) : null}
    </Panel>
  );
}
