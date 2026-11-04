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
