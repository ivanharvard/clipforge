import type { EditorSettings } from "../../types";
import { Icon } from "../Icon";
import { Panel } from "../Panel";

interface CropPanelProps {
  crop: EditorSettings["crop"];
  sourceWidth: number;
  sourceHeight: number;
  onChange: (crop: EditorSettings["crop"]) => void;
}

export function CropPanel({ crop, sourceWidth, sourceHeight, onChange }: CropPanelProps) {
  const update = (patch: Partial<EditorSettings["crop"]>) => {
    const next = { ...crop, ...patch };
    next.width = Math.max(2, Math.min(next.width, sourceWidth - next.x));
    next.height = Math.max(2, Math.min(next.height, sourceHeight - next.y));
    next.x = Math.max(0, Math.min(next.x, sourceWidth - next.width));
    next.y = Math.max(0, Math.min(next.y, sourceHeight - next.height));
    onChange(next);
  };

  const reset = () => onChange({ x: 0, y: 0, width: sourceWidth, height: sourceHeight, aspectLocked: false });

  return (
    <Panel title="Crop" icon={<Icon name="crop" />}>
      <div className="field-grid">
        <label>X <input type="number" min="0" max={sourceWidth - crop.width} value={crop.x} onChange={(event) => update({ x: event.currentTarget.valueAsNumber || 0 })} /></label>
        <label>Y <input type="number" min="0" max={sourceHeight - crop.height} value={crop.y} onChange={(event) => update({ y: event.currentTarget.valueAsNumber || 0 })} /></label>
        <label>Width <input type="number" min="2" max={sourceWidth - crop.x} value={crop.width} onChange={(event) => update({ width: event.currentTarget.valueAsNumber || 2 })} /></label>
        <label>Height <input type="number" min="2" max={sourceHeight - crop.y} value={crop.height} onChange={(event) => update({ height: event.currentTarget.valueAsNumber || 2 })} /></label>
      </div>
      <div className="panel-footer">
        <label className="check-control"><input type="checkbox" checked={crop.aspectLocked} onChange={(event) => update({ aspectLocked: event.currentTarget.checked })} />Lock ratio</label>
        <button className="text-button" type="button" onClick={reset}>Reset</button>
      </div>
    </Panel>
  );
}
