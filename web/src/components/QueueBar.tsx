import { useRef } from "react";

import { Icon } from "./Icon";

interface QueueItem {
  id: number;
  name: string;
  ready: boolean;
}

interface QueueBarProps {
  items: QueueItem[];
  activeIndex: number;
  exporting: boolean;
  onAddFiles: (files: File[]) => void;
  onSelect: (index: number) => void;
  onRemove: () => void;
  onExport: () => void;
}

export function QueueBar({ items, activeIndex, exporting, onAddFiles, onSelect, onRemove, onExport }: QueueBarProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const chooseFiles = (files: FileList | null) => {
    if (files) onAddFiles(Array.from(files));
  };

  return (
    <section className="queue-bar" aria-label="Video queue">
      <div className="queue-actions">
        <button className="button button-quiet" type="button" onClick={() => inputRef.current?.click()}>
          <Icon name="plus" /> Add videos
        </button>
        <input ref={inputRef} className="visually-hidden" aria-label="Add videos to queue" type="file" accept="video/*,.mkv,.avi,.mov,.m4v" multiple onChange={(event) => {
          chooseFiles(event.currentTarget.files);
          event.currentTarget.value = "";
        }} />
        {items.length > 0 ? <span className="queue-count">{items.length} {items.length === 1 ? "video" : "videos"}</span> : null}
      </div>

      <div className="queue-list">
        {items.map((item, index) => (
          <button className={`queue-item${index === activeIndex ? " active" : ""}`} type="button" aria-current={index === activeIndex ? "true" : undefined} onClick={() => onSelect(index)} key={item.id}>
            <span className={`queue-status${item.ready ? " ready" : ""}`} />
            <span>{item.name}</span>
            <small>{index + 1}</small>
          </button>
        ))}
      </div>

      <div className="queue-actions queue-actions-end">
        <button className="icon-button queue-nav" type="button" disabled={activeIndex <= 0} onClick={() => onSelect(activeIndex - 1)} aria-label="Previous video"><Icon name="chevronLeft" /></button>
        <button className="icon-button queue-nav" type="button" disabled={activeIndex < 0 || activeIndex >= items.length - 1} onClick={() => onSelect(activeIndex + 1)} aria-label="Next video"><Icon name="chevronRight" /></button>
        <button className="text-button remove-video" type="button" disabled={activeIndex < 0} onClick={onRemove}>Remove</button>
        <button className="button button-primary export-button" type="button" disabled={items.length === 0 || activeIndex < 0 || exporting} onClick={onExport}>
          <Icon name="download" /> {items.length > 1 ? "Export queue" : "Export video"}
        </button>
      </div>
    </section>
  );
}
