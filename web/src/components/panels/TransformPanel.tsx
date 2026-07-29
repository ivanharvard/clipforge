import { Icon } from "../Icon";
import { Panel } from "../Panel";

interface TransformPanelProps {
  rotation: number;
  flipHorizontal: boolean;
  flipVertical: boolean;
  onRotationChange: (rotation: number) => void;
  onFlipsChange: (horizontal: boolean, vertical: boolean) => void;
}

export function TransformPanel({
  rotation,
  flipHorizontal,
  flipVertical,
  onRotationChange,
  onFlipsChange,
}: TransformPanelProps) {
  return (
    <Panel title="Transform" icon={<Icon name="rotateCw" />}>
      <div className="control-group">
        <span className="control-caption">Rotate</span>
        <div className="button-row">
          <button className="icon-button" type="button" onClick={() => onRotationChange(rotation - 90)} aria-label="Rotate left">
            <Icon name="rotateCcw" />
          </button>
          <button className="icon-button" type="button" onClick={() => onRotationChange(rotation + 90)} aria-label="Rotate right">
            <Icon name="rotateCw" />
          </button>
          <output className="value-chip">{rotation}°</output>
        </div>
      </div>
      <div className="control-group">
        <span className="control-caption">Flip</span>
        <div className="button-row">
          <button className={`icon-button${flipHorizontal ? " active" : ""}`} type="button" onClick={() => onFlipsChange(!flipHorizontal, flipVertical)} aria-pressed={flipHorizontal} aria-label="Flip horizontally">
            <Icon name="flipHorizontal" />
          </button>
          <button className={`icon-button${flipVertical ? " active" : ""}`} type="button" onClick={() => onFlipsChange(flipHorizontal, !flipVertical)} aria-pressed={flipVertical} aria-label="Flip vertically">
            <Icon name="flipVertical" />
          </button>
        </div>
      </div>
    </Panel>
  );
}
