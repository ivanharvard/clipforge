export function audioNeedsResync(videoTime: number, audioTime: number, threshold = 0.12): boolean {
  return Math.abs(audioTime - videoTime) > threshold;
}
