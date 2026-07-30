export function formatDuration(milliseconds: number): string {
  const totalSeconds = Math.max(0, Math.floor(milliseconds / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  const centiseconds = Math.floor((milliseconds % 1000) / 10);

  const whole = [hours, minutes, seconds]
    .map((part) => String(part).padStart(2, "0"))
    .join(":");
  return centiseconds === 0 ? whole : `${whole}.${String(centiseconds).padStart(2, "0")}`;
}

export function parseTimestamp(value: string): number | null {
  const match = /^(\d+):([0-5]\d):([0-5]\d)(?:\.(\d{1,2}))?$/.exec(value.trim());
  if (!match) return null;
  const [, hours, minutes, seconds, fraction = ""] = match;
  const centiseconds = fraction.length === 1 ? Number(fraction) * 10 : Number(fraction || 0);
  return ((Number(hours) * 3600 + Number(minutes) * 60 + Number(seconds)) * 1000) + centiseconds * 10;
}

export function fileSize(bytes: number): string {
  if (bytes < 1024 * 1024) {
    return `${Math.max(1, Math.round(bytes / 1024))} KB`;
  }
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export function outputName(inputName: string): string {
  const base = inputName.replace(/\.[^.]+$/, "") || "video";
  return `${base} (clipforge).mp4`;
}

export function virtualInputName(inputName: string): string {
  const extension = inputName.match(/\.[a-z0-9]+$/i)?.[0].toLowerCase() ?? ".mp4";
  return `input${extension}`;
}
