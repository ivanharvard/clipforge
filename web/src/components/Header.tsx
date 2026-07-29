import { Icon } from "./Icon";

const LATEST_RELEASE = "https://github.com/ivanharvard/clipforge/releases/latest";

export function Header() {
  return (
    <header className="app-header">
      <a className="brand" href="#top" aria-label="ClipForge home">
        <span className="brand-mark"><Icon name="scissors" /></span>
        <span>ClipForge</span>
        <span className="web-label">Web</span>
      </a>

      <div className="header-actions">
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
