// @vitest-environment jsdom

import { act, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { ExportStatus } from "../types";
import { ExportDialog } from "./ExportDialog";

const runningStatus: ExportStatus = {
  phase: "loading",
  progress: 0.25,
  message: "Preparing your clip…",
};

describe("slow export download prompt", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it("appears after ten seconds and remains through the running phase", () => {
    vi.useFakeTimers();
    const { rerender } = render(
      <ExportDialog status={runningStatus} onClose={() => {}} onCancel={() => {}} />,
    );

    act(() => vi.advanceTimersByTime(9_999));
    expect(screen.queryByText("Taking too long?")).toBeNull();

    act(() => vi.advanceTimersByTime(1));
    expect(screen.getByText("Taking too long?")).toBeTruthy();

    rerender(
      <ExportDialog
        status={{ ...runningStatus, phase: "running" }}
        onClose={() => {}}
        onCancel={() => {}}
      />,
    );
    expect(screen.getByText("Taking too long?")).toBeTruthy();
  });

  it("hides the prompt when exporting finishes", () => {
    vi.useFakeTimers();
    const { rerender } = render(
      <ExportDialog status={runningStatus} onClose={() => {}} onCancel={() => {}} />,
    );
    act(() => vi.advanceTimersByTime(10_000));

    rerender(
      <ExportDialog
        status={{ phase: "success", progress: 1, message: "Saved" }}
        onClose={() => {}}
        onCancel={() => {}}
      />,
    );
    expect(screen.queryByText("Taking too long?")).toBeNull();
  });
});
