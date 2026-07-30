import type { ToolKind, ToolStage } from "../types";

export function togglePipelineStage(pipeline: ToolStage[], kind: ToolKind, enabled: boolean): ToolStage[] {
  return pipeline.map((stage) => stage.kind === kind ? { ...stage, enabled } : stage);
}

export function movePipelineStage(pipeline: ToolStage[], kind: ToolKind, destination: number): ToolStage[] {
  const source = pipeline.findIndex((stage) => stage.kind === kind);
  if (source < 0) return pipeline;
  const next = [...pipeline];
  const [stage] = next.splice(source, 1);
  next.splice(Math.max(0, Math.min(next.length, destination)), 0, stage);
  return next;
}

export function updateQueueEntry<T>(entries: T[], index: number, update: (entry: T) => T): T[] {
  if (!entries[index]) return entries;
  const next = [...entries];
  next[index] = update(entries[index]);
  return next;
}
