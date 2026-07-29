import { useCallback, useEffect, useRef, useState } from "react";

import { ClipForgeProject } from "../lib/wasm";
import {
  defaultSettings,
  type ClipSource,
  type EditorSettings,
} from "../types";

interface MediaMetadata {
  durationMs: number;
  width: number;
  height: number;
}

export function useClipEditor() {
  const projectRef = useRef<ClipForgeProject | null>(null);
  const urlRef = useRef<string | null>(null);
  const pendingFileRef = useRef<File | null>(null);
  const [clip, setClip] = useState<ClipSource | null>(null);
  const [pendingUrl, setPendingUrl] = useState<string | null>(null);
  const [settings, setSettings] = useState<EditorSettings | null>(null);

  const releaseCurrent = useCallback(() => {
    projectRef.current?.free();
    projectRef.current = null;
    if (urlRef.current) {
      URL.revokeObjectURL(urlRef.current);
      urlRef.current = null;
    }
  }, []);

  useEffect(() => releaseCurrent, [releaseCurrent]);

  const chooseFile = useCallback(
    (file: File) => {
      releaseCurrent();
      const url = URL.createObjectURL(file);
      urlRef.current = url;
      pendingFileRef.current = file;
      setClip(null);
      setSettings(null);
      setPendingUrl(url);
    },
    [releaseCurrent],
  );

  const loadMetadata = useCallback(({ durationMs, width, height }: MediaMetadata) => {
    const file = pendingFileRef.current;
    const url = urlRef.current;
    if (!file || !url || durationMs <= 0 || width <= 0 || height <= 0) {
      return;
    }

    const project = new ClipForgeProject(file.name, width, height, durationMs, 30);
    projectRef.current = project;
    setClip({ file, url, durationMs, width, height });
    setSettings(defaultSettings(width, height, durationMs));
    setPendingUrl(null);
  }, []);

  const closeClip = useCallback(() => {
    releaseCurrent();
    pendingFileRef.current = null;
    setPendingUrl(null);
    setClip(null);
    setSettings(null);
  }, [releaseCurrent]);

  const setTrim = useCallback((inMs: number, outMs: number) => {
    const project = projectRef.current;
    if (!project) return;
    project.setTrim(inMs, outMs);
    setSettings((current) => current ? { ...current, trim: { inMs, outMs } } : current);
  }, []);

  const setRotation = useCallback((rotation: number) => {
    const project = projectRef.current;
    if (!project) return;
    const normalized = ((rotation % 360) + 360) % 360;
    while (project.rotationDegrees !== normalized) {
      project.rotateClockwise();
    }
    setSettings((current) => current ? { ...current, rotation: normalized } : current);
  }, []);

  const setFlips = useCallback((horizontal: boolean, vertical: boolean) => {
    projectRef.current?.setFlips(horizontal, vertical);
    setSettings((current) => current ? {
      ...current,
      flipHorizontal: horizontal,
      flipVertical: vertical,
    } : current);
  }, []);

  const setCrop = useCallback((crop: EditorSettings["crop"]) => {
    const project = projectRef.current;
    if (!project) return;
    project.setCrop(crop.x, crop.y, crop.width, crop.height, crop.aspectLocked);
    setSettings((current) => current ? { ...current, crop } : current);
  }, []);

  const setResolution = useCallback((resolution: EditorSettings["resolution"]) => {
    const project = projectRef.current;
    if (!project) return;
    project.setResolution(
      resolution.preset,
      resolution.customWidth,
      resolution.customHeight,
      resolution.aspectLocked,
    );
    setSettings((current) => current ? { ...current, resolution } : current);
  }, []);

  const setAudio = useCallback((audio: EditorSettings["audio"]) => {
    projectRef.current?.setAudio(audio.volume, audio.muted, -1, audio.normalize);
    setSettings((current) => current ? { ...current, audio } : current);
  }, []);

  const setCompression = useCallback((compression: EditorSettings["compression"]) => {
    projectRef.current?.setCompression(
      compression.mode,
      compression.value,
      compression.frameRateLimit,
      compression.codec,
      compression.extraQuality,
      compression.tolerancePercent,
    );
    setSettings((current) => current ? { ...current, compression } : current);
  }, []);

  return {
    clip,
    pendingUrl,
    settings,
    projectRef,
    chooseFile,
    loadMetadata,
    closeClip,
    setTrim,
    setRotation,
    setFlips,
    setCrop,
    setResolution,
    setAudio,
    setCompression,
  };
}
