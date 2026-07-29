import type { EditorSettings } from "../types";
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
}

export function SettingsSidebar(props: SettingsSidebarProps) {
  const { settings } = props;
  return (
    <aside className="settings-sidebar" aria-label="Video settings">
      <TransformPanel rotation={settings.rotation} flipHorizontal={settings.flipHorizontal} flipVertical={settings.flipVertical} onRotationChange={props.onRotationChange} onFlipsChange={props.onFlipsChange} />
      <CropPanel crop={settings.crop} sourceWidth={props.sourceWidth} sourceHeight={props.sourceHeight} onChange={props.onCropChange} />
      <ResolutionPanel resolution={settings.resolution} onChange={props.onResolutionChange} />
      <AudioPanel audio={settings.audio} onChange={props.onAudioChange} />
      <CompressPanel compression={settings.compression} onChange={props.onCompressionChange} />
    </aside>
  );
}
