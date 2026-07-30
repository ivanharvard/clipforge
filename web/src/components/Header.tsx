import { Icon } from "./Icon";

const LATEST_RELEASE = "https://github.com/ivanharvard/clipforge/releases/latest";

export function Header({ canUndo, canRedo, onUndo, onRedo }: { canUndo: boolean; canRedo: boolean; onUndo: () => void; onRedo: () => void }) {
  return (
    <header className="app-header">
      <a className="brand" href="#top" aria-label="ClipForge home">
        <span className="brand-mark"><img src={`${import.meta.env.BASE_URL}app-icon.svg`} alt="" /></span>
        <span>ClipForge</span>
        <span className="web-label">Web</span>
      </a>

      <div className="header-actions">
        <button className="icon-button header-history" type="button" disabled={!canUndo} onClick={onUndo} aria-label="Undo"><Icon name="undo" /></button>
        <button className="icon-button header-history" type="button" disabled={!canRedo} onClick={onRedo} aria-label="Redo"><Icon name="redo" /></button>
        <span className="privacy-note"><Icon name="shield" /> Files stay on your device</span>
        <a className="button button-quiet desktop-download" href={LATEST_RELEASE} target="_blank" rel="noreferrer">
          <Icon name="download" /> Windows
        </a>
        <a className="button button-quiet desktop-download" href={LATEST_RELEASE} target="_blank" rel="noreferrer">
          <Icon name="download" /> Linux
        </a>
        <a className="icon-link" href="https://github.com/ivanharvard/clipforge" target="_blank" rel="noreferrer" aria-label="ClipForge on GitHub">
          <Icon name="github" />
        </a>
      </div>
    </header>
  );
}
