export interface MediaMetadata {
  durationMs: number;
  width: number;
  height: number;
}

export function readMetadata(file: File): Promise<MediaMetadata> {
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

export function createThumbnail(file: File): Promise<string> {
  return new Promise((resolve) => {
    const video = document.createElement("video");
    const url = URL.createObjectURL(file);
    let settled = false;
    let timeout = 0;
    const finish = (thumbnailUrl: string) => {
      if (settled) return;
      settled = true;
      window.clearTimeout(timeout);
      video.removeAttribute("src");
      video.load();
      URL.revokeObjectURL(url);
      resolve(thumbnailUrl);
    };
    const capture = () => {
      if (!video.videoWidth || !video.videoHeight) {
        finish("");
        return;
      }
      const canvas = document.createElement("canvas");
      canvas.width = 160;
      canvas.height = Math.max(90, Math.round(160 * video.videoHeight / video.videoWidth));
      const context = canvas.getContext("2d");
      if (!context) {
        finish("");
        return;
      }
      context.drawImage(video, 0, 0, canvas.width, canvas.height);
      finish(canvas.toDataURL("image/jpeg", 0.72));
    };
    timeout = window.setTimeout(() => finish(""), 8_000);
    video.preload = "auto";
    video.muted = true;
    video.playsInline = true;
    video.onerror = () => finish("");
    video.onloadeddata = () => {
      const previewTime = Math.min(1, Math.max(0, video.duration * 0.2));
      if (previewTime < 0.05) {
        capture();
        return;
      }
      video.onseeked = capture;
      video.currentTime = previewTime;
    };
    video.src = url;
  });
}
