import { describe, expect, it } from "vitest";

import { audioNeedsResync } from "./audioSync";

describe("audio preview synchronization", () => {
  it("corrects meaningful drift but ignores tiny timeupdate jitter", () => {
    expect(audioNeedsResync(10, 10.2)).toBe(true);
    expect(audioNeedsResync(10, 10.08)).toBe(false);
  });
});
