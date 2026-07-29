import { useCallback, useEffect, useRef, useState } from "react";

import { ClipForgeProject } from "../lib/wasm";
import { defaultSettings, type ClipSource, type EditorSettings } from "../types";

interface MediaMetadata {
  durationMs: number;
  width: number;
  height: number;
}

interface QueueEntry {
  id: number;
  file: File;
  metadata: MediaMetadata | null;
  project: ClipForgeProject | null;
  settings: EditorSettings | null;
}

export interface PreparedClip {
  file: File;
  project: ClipForgeProject;
}

let nextQueueId = 1;

function applyCompression(project: ClipForgeProject, compression: EditorSettings["compression"]) {
  project.setCompression(
    compression.mode,
    compression.value,
    compression.frameRateLimit,
    compression.codec,
    compression.extraQuality,
    compression.tolerancePercent,
  );
}

function readMetadata(file: File): Promise<MediaMetadata> {
  return new Promise((resolve, reject) => {
    const video = document.createElement("video");
    const url = URL.createObjectURL(file);
    const release = () => {
      video.removeAttribute("src");
      URL.revokeObjectURL(url);
    };
    video.preload = "metadata";
    video.onloadedmetadata = () => {
      const metadata = {
        durationMs: Math.round(video.duration * 1000),
        width: video.videoWidth,
        height: video.videoHeight,
      };
      release();
      if (metadata.durationMs > 0 && metadata.width > 0 && metadata.height > 0) {
        resolve(metadata);
      } else {
        reject(new Error(`Could not read video metadata for ${file.name}`));
      }
    };
    video.onerror = () => {
      release();
      reject(new Error(`This browser cannot read ${file.name}`));
    };
    video.src = url;
  });
}

export function useClipEditor() {
  const projectRef = useRef<ClipForgeProject | null>(null);
  const entriesRef = useRef<QueueEntry[]>([]);
  const activeIndexRef = useRef(-1);
  const urlRef = useRef<string | null>(null);
  const applyToAllRef = useRef(true);
  const sharedCompressionRef = useRef(defaultSettings(2, 2, 1).compression);
  const [entries, setEntries] = useState<QueueEntry[]>([]);
  const [activeIndex, setActiveIndex] = useState(-1);
  const [sourceUrl, setSourceUrl] = useState<string | null>(null);
  const [compressionApplyAll, setCompressionApplyAllState] = useState(true);

  const commitEntries = useCallback((next: QueueEntry[]) => {
    entriesRef.current = next;
    setEntries(next);
  }, []);

  const releaseUrl = useCallback(() => {
    if (urlRef.current) URL.revokeObjectURL(urlRef.current);
    urlRef.current = null;
  }, []);

  const activateClip = useCallback((index: number, queue = entriesRef.current) => {
    releaseUrl();
    const entry = queue[index];
    if (!entry) {
      activeIndexRef.current = -1;
      projectRef.current = null;
      setActiveIndex(-1);
      setSourceUrl(null);
      return;
    }
    const url = URL.createObjectURL(entry.file);
    urlRef.current = url;
    activeIndexRef.current = index;
    projectRef.current = entry.project;
    setActiveIndex(index);
    setSourceUrl(url);
  }, [releaseUrl]);

  useEffect(() => () => {
    releaseUrl();
    for (const entry of entriesRef.current) entry.project?.free();
  }, [releaseUrl]);

  const addFiles = useCallback((files: File[]) => {
    if (files.length === 0) return;
    const additions = files.map((file) => ({
      id: nextQueueId++,
      file,
      metadata: null,
      project: null,
      settings: null,
    }));
    const wasEmpty = entriesRef.current.length === 0;
    const next = [...entriesRef.current, ...additions];
    commitEntries(next);
    if (wasEmpty) activateClip(0, next);
  }, [activateClip, commitEntries]);

  const loadMetadata = useCallback((metadata: MediaMetadata) => {
    const index = activeIndexRef.current;
    const entry = entriesRef.current[index];
    if (!entry || entry.project || metadata.durationMs <= 0 || metadata.width <= 0 || metadata.height <= 0) return;

    const project = new ClipForgeProject(entry.file.name, metadata.width, metadata.height, metadata.durationMs, 30);
    const settings = defaultSettings(metadata.width, metadata.height, metadata.durationMs);
    if (applyToAllRef.current) settings.compression = { ...sharedCompressionRef.current };
    applyCompression(project, settings.compression);
    const next = [...entriesRef.current];
    next[index] = { ...entry, metadata, project, settings };
    projectRef.current = project;
    commitEntries(next);
  }, [commitEntries]);

  const removeActiveClip = useCallback(() => {
    const index = activeIndexRef.current;
    const entry = entriesRef.current[index];
    if (!entry) return;
    entry.project?.free();
    const next = entriesRef.current.filter((_, itemIndex) => itemIndex !== index);
    commitEntries(next);
    activateClip(next.length === 0 ? -1 : Math.min(index, next.length - 1), next);
  }, [activateClip, commitEntries]);

  const updateActive = useCallback((update: (entry: QueueEntry) => QueueEntry) => {
    const index = activeIndexRef.current;
    const entry = entriesRef.current[index];
    if (!entry) return;
    const next = [...entriesRef.current];
    next[index] = update(entry);
    commitEntries(next);
  }, [commitEntries]);

  const setTrim = useCallback((inMs: number, outMs: number) => {
    projectRef.current?.setTrim(inMs, outMs);
    updateActive((entry) => entry.settings ? { ...entry, settings: { ...entry.settings, trim: { inMs, outMs } } } : entry);
  }, [updateActive]);

  const setRotation = useCallback((rotation: number) => {
    const project = projectRef.current;
    if (!project) return;
    const normalized = ((rotation % 360) + 360) % 360;
    while (project.rotationDegrees !== normalized) project.rotateClockwise();
    updateActive((entry) => entry.settings ? { ...entry, settings: { ...entry.settings, rotation: normalized } } : entry);
  }, [updateActive]);

  const setFlips = useCallback((horizontal: boolean, vertical: boolean) => {
    projectRef.current?.setFlips(horizontal, vertical);
    updateActive((entry) => entry.settings ? { ...entry, settings: { ...entry.settings, flipHorizontal: horizontal, flipVertical: vertical } } : entry);
  }, [updateActive]);

  const setCrop = useCallback((crop: EditorSettings["crop"]) => {
    projectRef.current?.setCrop(crop.x, crop.y, crop.width, crop.height, crop.aspectLocked);
    updateActive((entry) => entry.settings ? { ...entry, settings: { ...entry.settings, crop } } : entry);
  }, [updateActive]);

  const setResolution = useCallback((resolution: EditorSettings["resolution"]) => {
    projectRef.current?.setResolution(resolution.preset, resolution.customWidth, resolution.customHeight, resolution.aspectLocked);
    updateActive((entry) => entry.settings ? { ...entry, settings: { ...entry.settings, resolution } } : entry);
  }, [updateActive]);

  const setAudio = useCallback((audio: EditorSettings["audio"]) => {
    projectRef.current?.setAudio(audio.volume, audio.muted, -1, audio.normalize);
    updateActive((entry) => entry.settings ? { ...entry, settings: { ...entry.settings, audio } } : entry);
  }, [updateActive]);

  const setCompression = useCallback((compression: EditorSettings["compression"]) => {
    if (applyToAllRef.current) {
      sharedCompressionRef.current = { ...compression };
      const next = entriesRef.current.map((entry) => {
        if (entry.project) applyCompression(entry.project, compression);
        return entry.settings ? { ...entry, settings: { ...entry.settings, compression: { ...compression } } } : entry;
      });
      commitEntries(next);
    } else {
      const project = projectRef.current;
      if (!project) return;
      applyCompression(project, compression);
      updateActive((entry) => entry.settings ? { ...entry, settings: { ...entry.settings, compression } } : entry);
    }
  }, [commitEntries, updateActive]);

  const setCompressionApplyAll = useCallback((enabled: boolean) => {
    applyToAllRef.current = enabled;
    setCompressionApplyAllState(enabled);
    const active = entriesRef.current[activeIndexRef.current];
    if (!enabled || !active?.settings) return;
    sharedCompressionRef.current = { ...active.settings.compression };
    const next = entriesRef.current.map((entry) => {
      if (entry.project) applyCompression(entry.project, active.settings!.compression);
      return entry.settings ? { ...entry, settings: { ...entry.settings, compression: { ...active.settings!.compression } } } : entry;
    });
    commitEntries(next);
  }, [commitEntries]);

  const prepareQueue = useCallback(async (): Promise<PreparedClip[]> => {
    const next = [...entriesRef.current];
    for (let index = 0; index < next.length; index += 1) {
      const entry = next[index];
      if (entry.project) continue;
      const metadata = await readMetadata(entry.file);
      const project = new ClipForgeProject(entry.file.name, metadata.width, metadata.height, metadata.durationMs, 30);
      const settings = defaultSettings(metadata.width, metadata.height, metadata.durationMs);
      if (applyToAllRef.current) settings.compression = { ...sharedCompressionRef.current };
      applyCompression(project, settings.compression);
      next[index] = { ...entry, metadata, project, settings };
    }
    commitEntries(next);
    projectRef.current = next[activeIndexRef.current]?.project ?? null;
    return next.map((entry) => ({ file: entry.file, project: entry.project! }));
  }, [commitEntries]);

  const active = entries[activeIndex];
  const clip: ClipSource | null = active?.metadata && sourceUrl
    ? { file: active.file, url: sourceUrl, ...active.metadata }
    : null;

  return {
    clip,
    pendingUrl: active && !active.metadata ? sourceUrl : null,
    settings: active?.settings ?? null,
    queue: entries.map((entry) => ({ id: entry.id, name: entry.file.name, ready: entry.project !== null })),
    activeIndex,
    compressionApplyAll,
    projectRef,
    addFiles,
    activateClip,
    removeActiveClip,
    loadMetadata,
    setTrim,
    setRotation,
    setFlips,
    setCrop,
    setResolution,
    setAudio,
    setCompression,
    setCompressionApplyAll,
    prepareQueue,
  };
}
