import type { ReactNode } from "react";
import type { EditorSettings, ToolKind } from "../types";
import { ToolFrameProvider } from "./Panel";
import { AudioPanel } from "./panels/AudioPanel";
import { CompressPanel } from "./panels/CompressPanel";
import { CropPanel } from "./panels/CropPanel";
import { ResolutionPanel } from "./panels/ResolutionPanel";
import { TransformPanel } from "./panels/TransformPanel";

interface SettingsSidebarProps {
  settings: EditorSettings;
  sourceWidth: number;
  sourceHeight: number;
  onRotationChange: (rotation: number) => void;
  onFlipsChange: (horizontal: boolean, vertical: boolean) => void;
  onCropChange: (crop: EditorSettings["crop"]) => void;
  onResolutionChange: (resolution: EditorSettings["resolution"]) => void;
  onAudioChange: (audio: EditorSettings["audio"]) => void;
  onCompressionChange: (compression: EditorSettings["compression"]) => void;
  compressionApplyAll: boolean;
  onCompressionApplyAllChange: (enabled: boolean) => void;
  onToolEnabledChange: (kind: ToolKind, enabled: boolean) => void;
  onToolMove: (kind: ToolKind, destination: number) => void;
  audioStatus?: string;
}

export function SettingsSidebar(props: SettingsSidebarProps) {
  const { settings } = props;
  const panels: Record<ToolKind, ReactNode> = {
    compress: <CompressPanel compression={settings.compression} applyToAll={props.compressionApplyAll} onChange={props.onCompressionChange} onApplyToAllChange={props.onCompressionApplyAllChange} />,
    transform: <TransformPanel rotation={settings.rotation} flipHorizontal={settings.flipHorizontal} flipVertical={settings.flipVertical} onRotationChange={props.onRotationChange} onFlipsChange={props.onFlipsChange} />,
    crop: <CropPanel crop={settings.crop} sourceWidth={props.sourceWidth} sourceHeight={props.sourceHeight} onChange={props.onCropChange} />,
    resolution: <ResolutionPanel resolution={settings.resolution} onChange={props.onResolutionChange} />,
    audio: <AudioPanel audio={settings.audio} onChange={props.onAudioChange} status={props.audioStatus} />,
  };

  return (
    <aside className="settings-sidebar" aria-label="Video settings">
      {settings.pipeline.map((stage, position) => (
        <ToolFrameProvider key={stage.kind} value={{
          kind: stage.kind,
          enabled: stage.enabled,
          position,
          onToggle: (enabled) => props.onToolEnabledChange(stage.kind, enabled),
          onMove: (destination) => props.onToolMove(stage.kind, destination),
          onDrop: props.onToolMove,
        }}>
          {panels[stage.kind]}
        </ToolFrameProvider>
      ))}
    </aside>
  );
}
