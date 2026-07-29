export function formatDuration(milliseconds: number): string {
  const totalSeconds = Math.max(0, Math.floor(milliseconds / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  const centiseconds = Math.floor((milliseconds % 1000) / 10);

  return [hours, minutes, seconds]
    .map((part) => String(part).padStart(2, "0"))
    .join(":") + `.${String(centiseconds).padStart(2, "0")}`;
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
