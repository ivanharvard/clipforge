import { afterEach } from "vitest";
import { cleanup } from "@testing-library/react";

// @testing-library/react's own auto-cleanup only registers itself against a
// *global* afterEach (globalThis.afterEach). This project imports afterEach
// explicitly per test file instead of enabling Vitest's `globals: true`, so
// that auto-registration silently never fires — component trees rendered in
// one test stay mounted in document.body for the next test in the same
// file, and DOM queries like screen.queryByText can match leftover elements
// from a previous test instead of the current one.
afterEach(() => {
  cleanup();
});
