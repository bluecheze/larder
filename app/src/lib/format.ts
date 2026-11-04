const UNITS = ["B", "KB", "MB", "GB", "TB"];

export function bytes(n: number): string {
  let value = n;
  let unit = 0;
  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024;
    unit++;
  }
  const digits = value >= 10 || unit === 0 ? 0 : 1;
  return `${value.toFixed(digits)} ${UNITS[unit]}`;
}

export function files(n: number): string {
  return n === 1 ? "1 file" : `${n} files`;
}

export function ago(seconds: number, now = Date.now() / 1000): string {
  const diff = Math.max(0, now - seconds);
  if (diff < 45) return "just now";
  if (diff < 3600) return `${Math.max(1, Math.round(diff / 60))} min ago`;
  if (diff < 86400) return `${Math.round(diff / 3600)} h ago`;
  return `${Math.round(diff / 86400)} days ago`;
}
