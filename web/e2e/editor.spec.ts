import { execFileSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { expect, test } from "@playwright/test";

const fixture = join(tmpdir(), "clipforge-two-track.mp4");

test.beforeAll(() => {
  execFileSync("ffmpeg", [
    "-hide_banner", "-loglevel", "error",
    "-f", "lavfi", "-i", "testsrc2=size=640x360:rate=30",
    "-f", "lavfi", "-i", "sine=frequency=440",
    "-f", "lavfi", "-i", "sine=frequency=880",
    "-t", "2", "-map", "0:v", "-map", "1:a", "-map", "2:a",
    "-c:v", "libx264", "-pix_fmt", "yuv420p", "-c:a", "aac",
    "-metadata:s:a:0", "language=eng", "-metadata:s:a:1", "language=spa",
    "-disposition:a:0", "default", "-disposition:a:1", "0", fixture, "-y",
  ]);
});

test("supports ordered tools, two-track preview, crop handles, and responsive transport", async ({ page }) => {
  await page.goto("/");
  await page.getByLabel("Choose videos").setInputFiles(fixture);
  const tracks = page.getByLabel("Audio track");
  await expect(tracks.locator("option")).toHaveCount(2, { timeout: 60_000 });
  await tracks.selectOption("1");
  await expect(page.getByRole("button", { name: "Play" })).toBeEnabled({ timeout: 60_000 });

  const compression = page.getByLabel("Enable Compress");
  await compression.uncheck();
  await expect(page.getByRole("button", { name: "Undo" })).toBeEnabled();
  await page.getByRole("button", { name: "Undo" }).click();
  await expect(compression).toBeChecked();

  await page.getByRole("button", { name: /Reorder Audio/ }).press("ArrowUp");
  const toolNames = await page.locator(".panel-disclosure > span:nth-child(2)").allTextContents();
  expect(toolNames.indexOf("Audio")).toBeLessThan(toolNames.indexOf("Resolution"));
  await expect(page.getByRole("button", { name: /Resize crop from/ })).toHaveCount(4);

  const start = page.getByRole("textbox", { name: "Start" });
  await start.fill("bad value");
  await start.press("Enter");
  await expect(start).toHaveValue("00:00:00");

  await page.setViewportSize({ width: 960, height: 600 });
  await expect(page.getByRole("button", { name: "Export video" })).toBeVisible();
  await page.setViewportSize({ width: 1920, height: 1080 });
  await expect(page.getByRole("region", { name: "Clip timeline" })).toBeVisible();
});
