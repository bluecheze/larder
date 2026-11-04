import type { Prefs } from "./api";

export function applyTheme(prefs: Prefs | undefined) {
  const root = document.documentElement;
  root.dataset.theme = prefs?.theme ?? "system";
  root.dataset.accent = prefs?.accent ?? "cobalt";
}
