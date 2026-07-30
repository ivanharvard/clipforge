import { useCallback, useEffect, useRef, useState } from "react";

import { ClipForgeProject } from "../lib/wasm";
import { movePipelineStage, togglePipelineStage, updateQueueEntry } from "../lib/pipeline";
import { revokeAudioPreviews } from "../media/exportVideo";
import { defaultSettings, type AudioTrack, type ClipSource, type EditorSettings, type ToolKind } from "../types";

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
  undo: EditorSnapshot[];
  redo: EditorSnapshot[];
  lastHistoryKey: string;
  lastHistoryAt: number;
}

interface EditorSnapshot {
  projectJson: string;
  settings: EditorSettings;
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
  }, [releaseUrl]);

  const addFiles = useCallback((files: File[]) => {
    if (files.length === 0) return;
    const additions = files.map((file) => ({
      id: nextQueueId++,
      file,
      metadata: null,
      project: null,
      settings: null,
      undo: [],
      redo: [],
      lastHistoryKey: "",
      lastHistoryAt: 0,
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
    revokeAudioPreviews(entry.id);
    entry.project?.free();
    const next = entriesRef.current.filter((_, itemIndex) => itemIndex !== index);
    commitEntries(next);
    activateClip(next.length === 0 ? -1 : Math.min(index, next.length - 1), next);
  }, [activateClip, commitEntries]);

  const updateActive = useCallback((update: (entry: QueueEntry) => QueueEntry) => {
    const index = activeIndexRef.current;
    const entry = entriesRef.current[index];
    if (!entry) return;
    const next = updateQueueEntry(entriesRef.current, index, update);
    commitEntries(next);
  }, [commitEntries]);

  const pushHistory = useCallback((key: string) => {
    const index = activeIndexRef.current;
    const entry = entriesRef.current[index];
    if (!entry?.project || !entry.settings) return;
    const now = performance.now();
    const next = [...entriesRef.current];
    if (entry.lastHistoryKey === key && now - entry.lastHistoryAt < 450) {
      next[index] = { ...entry, lastHistoryAt: now };
    } else {
      const snapshot = { projectJson: entry.project.toJson(), settings: structuredClone(entry.settings) };
      next[index] = {
        ...entry,
        undo: [...entry.undo.slice(-99), snapshot],
        redo: [],
        lastHistoryKey: key,
        lastHistoryAt: now,
      };
    }
    commitEntries(next);
  }, [commitEntries]);

  const setTrim = useCallback((inMs: number, outMs: number) => {
    pushHistory("trim");
    projectRef.current?.setTrim(inMs, outMs);
    updateActive((entry) => entry.settings ? { ...entry, settings: { ...entry.settings, trim: { inMs, outMs } } } : entry);
  }, [pushHistory, updateActive]);

  const setRotation = useCallback((rotation: number) => {
    const project = projectRef.current;
    if (!project) return;
    pushHistory("rotation");
    const normalized = ((rotation % 360) + 360) % 360;
    while (project.rotationDegrees !== normalized) project.rotateClockwise();
    updateActive((entry) => entry.settings ? { ...entry, settings: { ...entry.settings, rotation: normalized } } : entry);
  }, [pushHistory, updateActive]);

  const setFlips = useCallback((horizontal: boolean, vertical: boolean) => {
    pushHistory("flips");
    projectRef.current?.setFlips(horizontal, vertical);
    updateActive((entry) => entry.settings ? { ...entry, settings: { ...entry.settings, flipHorizontal: horizontal, flipVertical: vertical } } : entry);
  }, [pushHistory, updateActive]);

  const setCrop = useCallback((crop: EditorSettings["crop"]) => {
    pushHistory("crop");
    projectRef.current?.setCrop(crop.x, crop.y, crop.width, crop.height, crop.aspectLocked);
    updateActive((entry) => entry.settings ? { ...entry, settings: { ...entry.settings, crop } } : entry);
  }, [pushHistory, updateActive]);

  const setResolution = useCallback((resolution: EditorSettings["resolution"]) => {
    pushHistory("resolution");
    projectRef.current?.setResolution(resolution.preset, resolution.customWidth, resolution.customHeight, resolution.aspectLocked);
    updateActive((entry) => entry.settings ? { ...entry, settings: { ...entry.settings, resolution } } : entry);
  }, [pushHistory, updateActive]);

  const setAudio = useCallback((audio: EditorSettings["audio"]) => {
    pushHistory("audio");
    projectRef.current?.setAudio(audio.volume, audio.muted, audio.trackIndex, audio.normalize);
    updateActive((entry) => entry.settings ? { ...entry, settings: { ...entry.settings, audio } } : entry);
  }, [pushHistory, updateActive]);

  const setToolEnabled = useCallback((kind: ToolKind, enabled: boolean) => {
    pushHistory(`tool-toggle-${kind}`);
    projectRef.current?.setToolEnabled(kind, enabled);
    updateActive((entry) => entry.settings ? {
      ...entry,
      settings: {
        ...entry.settings,
        pipeline: togglePipelineStage(entry.settings.pipeline, kind, enabled),
      },
    } : entry);
  }, [pushHistory, updateActive]);

  const moveTool = useCallback((kind: ToolKind, destination: number) => {
    const entry = entriesRef.current[activeIndexRef.current];
    if (!entry?.settings) return;
    const source = entry.settings.pipeline.findIndex((stage) => stage.kind === kind);
    if (source < 0) return;
    pushHistory(`tool-move-${kind}`);
    const target = Math.max(0, Math.min(entry.settings.pipeline.length - 1, destination));
    const pipeline = movePipelineStage(entry.settings.pipeline, kind, destination);
    projectRef.current?.moveTool(kind, target);
    updateActive((current) => current.settings ? { ...current, settings: { ...current.settings, pipeline } } : current);
  }, [pushHistory, updateActive]);

  const setAudioTracks = useCallback((tracks: AudioTrack[], normalizedJson: string) => {
    const project = projectRef.current;
    if (!project) return;
    project.setMediaInfoJson(normalizedJson);
    updateActive((entry) => {
      if (!entry.settings) return entry;
      const defaultIndex = Math.max(0, tracks.findIndex((track) => track.isDefault));
      const audio = { ...entry.settings.audio, tracks, trackIndex: defaultIndex };
      project.setAudio(audio.volume, audio.muted, audio.trackIndex, audio.normalize);
      return { ...entry, settings: { ...entry.settings, audio } };
    });
  }, [updateActive]);

  const setCompression = useCallback((compression: EditorSettings["compression"]) => {
    if (applyToAllRef.current) {
      sharedCompressionRef.current = { ...compression };
      const next = entriesRef.current.map((entry) => {
        if (!entry.project || !entry.settings) return entry;
        const snapshot = { projectJson: entry.project.toJson(), settings: structuredClone(entry.settings) };
        applyCompression(entry.project, compression);
        return {
          ...entry,
          settings: { ...entry.settings, compression: { ...compression } },
          undo: [...entry.undo.slice(-99), snapshot],
          redo: [],
          lastHistoryKey: "compression",
          lastHistoryAt: performance.now(),
        };
      });
      commitEntries(next);
    } else {
      const project = projectRef.current;
      if (!project) return;
      pushHistory("compression");
      applyCompression(project, compression);
      updateActive((entry) => entry.settings ? { ...entry, settings: { ...entry.settings, compression } } : entry);
    }
  }, [commitEntries, pushHistory, updateActive]);

  const restoreHistory = useCallback((direction: "undo" | "redo") => {
    const index = activeIndexRef.current;
    const entry = entriesRef.current[index];
    if (!entry?.project || !entry.settings) return;
    const source = direction === "undo" ? entry.undo : entry.redo;
    const snapshot = source.at(-1);
    if (!snapshot) return;
    const current = { projectJson: entry.project.toJson(), settings: structuredClone(entry.settings) };
    const restoredProject = ClipForgeProject.fromJson(snapshot.projectJson);
    entry.project.free();
    const restored: QueueEntry = {
      ...entry,
      project: restoredProject,
      settings: structuredClone(snapshot.settings),
      undo: direction === "undo" ? entry.undo.slice(0, -1) : [...entry.undo, current],
      redo: direction === "redo" ? entry.redo.slice(0, -1) : [...entry.redo, current],
      lastHistoryKey: "",
      lastHistoryAt: 0,
    };
    const next = [...entriesRef.current];
    next[index] = restored;
    projectRef.current = restoredProject;
    commitEntries(next);
  }, [commitEntries]);
  const undo = useCallback(() => restoreHistory("undo"), [restoreHistory]);
  const redo = useCallback(() => restoreHistory("redo"), [restoreHistory]);

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
    canUndo: Boolean(active?.undo.length),
    canRedo: Boolean(active?.redo.length),
    activeId: active?.id ?? null,
    activeFile: active?.file ?? null,
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
    setAudioTracks,
    setToolEnabled,
    moveTool,
    setCompression,
    setCompressionApplyAll,
    prepareQueue,
    undo,
    redo,
  };
}
