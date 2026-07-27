# ClipForge Design System

This document defines the visual language for ClipForge. The goal is a single,
consistent, restrained custom look that is identical on Windows and Linux —
not a GNOME app awkwardly ported to Windows, and not a Windows app pretending
to be native GTK. Every token and component rule below applies equally on
both platforms; the only per-OS difference permitted is system font selection
(section 5).

## 1. Design Principles

- One custom look, everywhere. The app never adopts GNOME header bars,
  Windows ribbons, or other platform chrome.
- Flat surfaces with subtle borders instead of heavy shadows or gradients.
  Depth is communicated with a 1px border and a small background-color step,
  not drop shadows.
- Restraint over decoration: no unnecessary ornamentation, gloss, or
  skeuomorphism. Every visual element earns its place by conveying state or
  structure.
- Consistency over platform-native mimicry: a control looks and behaves the
  same in every build, because muscle memory should transfer between OSes.

## 2. Spacing Scale

Base unit: 4px. All padding, gaps, and margins use one of:

| Token | Value | Typical use |
|-------|-------|-------------|
| `space-1` | 4px | icon-to-label gap, tight control padding |
| `space-2` | 8px | control internal padding, small gaps |
| `space-3` | 12px | panel section padding |
| `space-4` | 16px | panel padding, gap between sidebar panels |
| `space-6` | 24px | gap between major regions (preview / sidebar) |
| `space-8` | 32px | outer window padding, top/bottom bar height padding |

No arbitrary pixel values in component code — always reference a spacing
token from `theme.slint`.

## 3. Corner Radius

| Token | Value | Applies to |
|-------|-------|------------|
| `radius-control` | 6px | buttons, inputs, sliders, dropdowns |
| `radius-panel` | 8px | sidebar panels, dialogs, modal containers |
| `radius-none` | 0px | title bar, outer window edges (flush against the OS window frame) |

## 4. Color Roles (Light / Dark)

Colors are defined as semantic roles, never as raw hex values in component
files. Both themes must meet WCAG AA contrast (4.5:1 for body text, 3:1 for
large text/icons) against their paired background.

| Role | Light | Dark | Usage |
|------|-------|------|-------|
| `bg` | `#F5F5F6` | `#1B1C1E` | window background |
| `surface` | `#FFFFFF` | `#242528` | panel / card background |
| `surface-alt` | `#ECECEE` | `#2C2D30` | recessed areas, scrubber track |
| `border` | `#DCDCDF` | `#3A3B3F` | 1px separators, control outlines |
| `text-primary` | `#1A1A1C` | `#F0F0F1` | primary text/icons |
| `text-secondary` | `#6B6C70` | `#9A9BA0` | labels, timecodes, hints |
| `accent` | `#3A7DFF` | `#5B93FF` | primary action, playhead, active state |
| `accent-hover` | `#2E67E0` | `#7AA6FF` | hover/pressed state of accent elements |
| `danger` | `#E5484D` | `#F16C71` | destructive actions, error state |
| `success` | `#2FA968` | `#4CCB84` | export success state |
| `focus-ring` | `#3A7DFF` @ 40% opacity | `#5B93FF` @ 40% opacity | keyboard focus outline |

(Exact hex values are a starting point and may be refined once implemented in
`theme.slint`; the role names and pairing rule are the contract components
must follow.)

## 5. Typography

- Windows: `Segoe UI`.
- Linux: system sans-serif (`sans-serif` generic family, resolved by
  fontconfig), so the app matches whatever the distro has configured rather
  than forcing a bundled font that looks foreign.
- Timecodes and numeric fields (in/out points, resolution width/height) use a
  monospace fallback (`Consolas` on Windows, `monospace` generic on Linux) so
  digits don't shift width as they change.
- Fixed type scale, no arbitrary sizes:

| Token | Size | Weight | Usage |
|-------|------|--------|-------|
| `text-xs` | 11px | regular | secondary labels, hints |
| `text-sm` | 13px | regular | control labels, body |
| `text-md` | 14px | medium | panel section headers |
| `text-lg` | 16px | medium | dialog titles |
| `text-mono` | 13px | regular | timecodes, numeric inputs |

## 6. Iconography

- Single custom icon set, stroke-based (not filled), 1.5px stroke weight, on
  a 20×20 grid.
- SVG sources live in `crates/clipforge-app/icons/src/` and are the single
  source of truth; exported multi-resolution PNG/ICO for packaging live under
  `packaging/shared/icons/`.
- States: 100% opacity default, `text-secondary`-toned by default, switches
  to `accent` on active/selected, 40% opacity when disabled.

## 7. Title Bar

- One compact custom title bar, height 36px, flush with the window's top
  edge (`radius-none`).
- Draggable region covers the bar except interactive controls.
- Custom-drawn window controls (minimize/maximize/close) rendered with the
  app's own icon set — not OS-native buttons — so control order and styling
  is identical on Windows and Linux (left-to-right: minimize, maximize,
  close, matching Windows convention, applied uniformly on both OSes for
  consistency).
- Left side: app icon + "Open Clip" action. Right side: "Export" action, then
  window controls.

## 8. Component Anatomy — Preview Pane

- Fills the left ~65% of the main content area.
- Video is letterboxed (centered, aspect-ratio preserved) against `surface`
  background — never stretched or cropped to fill the pane.
- Empty state (no clip loaded): centered placeholder icon + "Open a clip to
  begin" text in `text-secondary`.

## 9. Component Anatomy — Sidebar Shell

- Fixed vertical stack of the 5 panels (Transform, Crop, Resolution, Audio,
  Compress), each using the shared `panel_section` shell.
- Each panel has a header row (icon + label, `text-md`) and a body with
  `space-4` padding.
- Panels are not collapsible in v1 (fixed-stack, not accordion) — all 5 are
  visible at once, scrolling as a single vertical list if the window is
  short.
- Sidebar width is fixed (not user-resizable in v1).

## 10. Component Anatomy — Transform Panel

- Rotate left / rotate right buttons (90° increments) as a horizontal icon
  button pair.
- Flip horizontal / flip vertical as a horizontal toggle-icon-button pair.
- Reset-to-default icon button aligned right of the header.

## 11. Component Anatomy — Crop Panel

- Four numeric fields (X, Y, Width, Height) in a 2×2 grid, monospace type.
- Aspect-lock toggle (link icon) between Width and Height.
- Reset action returns to full-frame crop.

## 12. Component Anatomy — Resolution Panel

- Preset dropdown (e.g. Original, 1080p, 720p, 480p, Custom).
- Custom width/height numeric fields (monospace), shown/enabled only when
  "Custom" is selected.
- Aspect-lock link icon shared visually with the Crop panel for consistency.

## 13. Component Anatomy — Audio Panel

- Volume slider (`labeled_slider` component) with percentage readout.
- Mute toggle icon button.
- Track selector dropdown (for multi-audio-track sources).
- Normalize-audio toggle.

## 14. Component Anatomy — Compress Panel

- Quality-mode switch: CRF / target bitrate / target file size (segmented
  control, three options).
- One numeric input matching the selected mode (CRF value, bitrate in
  kbps, or target size in MB).
- Estimated output size readout in `text-secondary`, updates as inputs
  change.

## 15. Component Anatomy — Timeline / Scrubber Bar

- Full-width bar beneath the preview/sidebar split, height 56px.
- Track: `surface-alt` background, `radius-control` ends.
- Playhead: a thin `accent`-colored vertical line with a circular handle.
- In/out handles: distinct filled markers in `accent`, draggable, clamped so
  in ≤ out.
- Time labels: current in-point (left), duration/out-point (right), both in
  `text-mono`.
- Centered play/pause icon button below the track.

## 16. Component Anatomy — Export Dialog

- Modal (`radius-panel`, centered, dims background).
- Destination path field + browse button.
- Progress bar using `accent` fill on `surface-alt` track.
- Cancel button (secondary style) while running; Close button on
  success/error.
- Success state: `success`-colored check icon + summary line. Error state:
  `danger`-colored icon + error message text.

## 17. Theming Mechanism

- `theme.slint` exposes a `global Theme` singleton with all color/spacing/
  radius tokens as properties, plus a `dark: bool` property.
- On startup, `clipforge-app::theme.rs` detects the OS light/dark preference
  and sets `Theme.dark` accordingly.
- A user override (light/dark/system) is persisted (e.g. in a small config
  file) and takes precedence over OS detection on subsequent launches.

## 18. States & Feedback

Every interactive control defines these states consistently:

- **Hover**: background steps from `surface` to `surface-alt`.
- **Pressed**: background uses `accent-hover` (for accent controls) or a
  darker `surface-alt` step (for neutral controls).
- **Disabled**: 40% opacity, no hover/pressed response.
- **Focus** (keyboard navigation): `focus-ring` outline, 2px, offset 1px.

## 19. Motion

- Motion is minimal and fast: 100–150ms ease-out transitions on hover and
  panel-expand only.
- The playhead and scrubber position updates are never animated or eased —
  they must track playback/drag input exactly, with zero perceived lag.
