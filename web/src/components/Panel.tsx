import { createContext, useContext, useState, type DragEvent, type KeyboardEvent, type ReactNode } from "react";

import type { ToolKind } from "../types";
import { Icon } from "./Icon";

interface ToolFrame {
  kind: ToolKind;
  enabled: boolean;
  position: number;
  onToggle: (enabled: boolean) => void;
  onMove: (destination: number) => void;
  onDrop: (kind: ToolKind, destination: number) => void;
}

const ToolFrameContext = createContext<ToolFrame | null>(null);

export function ToolFrameProvider({ value, children }: { value: ToolFrame; children: ReactNode }) {
  return <ToolFrameContext value={value}>{children}</ToolFrameContext>;
}

interface PanelProps {
  title: string;
  icon: ReactNode;
  children: ReactNode;
}

export function Panel({ title, icon, children }: PanelProps) {
  const frame = useContext(ToolFrameContext);
  const [expanded, setExpanded] = useState(true);
  if (!frame) return null;

  const drop = (event: DragEvent<HTMLElement>) => {
    event.preventDefault();
    const sourceKind = event.dataTransfer.getData("text/clipforge-tool") as ToolKind;
    if (sourceKind) frame.onDrop(sourceKind, frame.position);
  };
  const moveWithKeyboard = (event: KeyboardEvent<HTMLButtonElement>) => {
    if (event.key === "ArrowUp") {
      event.preventDefault();
      frame.onMove(frame.position - 1);
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      frame.onMove(frame.position + 1);
    }
  };

  return (
    <section className={`panel${frame.enabled ? "" : " disabled"}`} onDragOver={(event) => event.preventDefault()} onDrop={drop}>
      <div className="panel-title">
        <button
          className="tool-handle"
          type="button"
          draggable
          aria-label={`Reorder ${title}; use arrow keys or drag`}
          onKeyDown={moveWithKeyboard}
          onDragStart={(event) => {
            event.dataTransfer.effectAllowed = "move";
            event.dataTransfer.setData("text/clipforge-tool", frame.kind);
          }}
        >⠿</button>
        <label className="tool-enable">
          <input type="checkbox" checked={frame.enabled} onChange={(event) => frame.onToggle(event.currentTarget.checked)} />
          <span className="visually-hidden">Enable {title}</span>
        </label>
        <button className="panel-disclosure" type="button" onClick={() => setExpanded((value) => !value)} aria-expanded={expanded}>
          <span className="panel-icon">{icon}</span>
          <span>{title}</span>
          <Icon name={expanded ? "chevronDown" : "chevronRight"} />
        </button>
      </div>
      {expanded && frame.enabled ? <div className="panel-body">{children}</div> : null}
    </section>
  );
}
