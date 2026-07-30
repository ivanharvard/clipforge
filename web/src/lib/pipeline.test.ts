import { describe, expect, it } from "vitest";

import { defaultPipeline } from "../types";
import { movePipelineStage, togglePipelineStage, updateQueueEntry } from "./pipeline";

describe("per-video pipeline state", () => {
  it("starts with compression first and retains disabled settings", () => {
    const initial = defaultPipeline();
    const disabled = togglePipelineStage(initial, "compress", false);
    expect(initial[0]).toEqual({ kind: "compress", enabled: true });
    expect(disabled[0]).toEqual({ kind: "compress", enabled: false });
  });

  it("reorders stages without mutating the previous pipeline", () => {
    const initial = defaultPipeline();
    const moved = movePipelineStage(initial, "audio", 1);
    expect(moved.map((stage) => stage.kind)).toEqual(["compress", "audio", "transform", "crop", "resolution"]);
    expect(initial.map((stage) => stage.kind)).toEqual(["compress", "transform", "crop", "resolution", "audio"]);
  });

  it("isolates queue updates to the active video", () => {
    const queue = [{ pipeline: defaultPipeline() }, { pipeline: defaultPipeline() }];
    const next = updateQueueEntry(queue, 1, (entry) => ({ ...entry, pipeline: togglePipelineStage(entry.pipeline, "crop", false) }));
    expect(next[0]).toBe(queue[0]);
    expect(next[0].pipeline[2].enabled).toBe(true);
    expect(next[1].pipeline[2].enabled).toBe(false);
  });
});
