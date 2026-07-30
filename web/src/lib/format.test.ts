import { describe, expect, it } from "vitest";

import { formatDuration, parseTimestamp } from "./format";

describe("timestamp editing", () => {
  it("formats whole seconds without a forced fraction", () => {
    expect(formatDuration(3_000)).toBe("00:00:03");
  });

  it("preserves centisecond precision when present", () => {
    expect(formatDuration(3_420)).toBe("00:00:03.42");
    expect(parseTimestamp("01:02:03.4")).toBe(3_723_400);
    expect(parseTimestamp("01:02:03.42")).toBe(3_723_420);
  });

  it("rejects malformed and out-of-range fields", () => {
    expect(parseTimestamp("12:99:00")).toBeNull();
    expect(parseTimestamp("not a time")).toBeNull();
  });
});
