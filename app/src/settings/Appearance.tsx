import type { Accent, Prefs, Theme } from "../lib/api";

const THEMES: [Theme, string][] = [
  ["system", "System"],
  ["light", "Light"],
  ["dark", "Dark"],
];

const ACCENTS: [Accent, string][] = [
  ["cobalt", "Cobalt"],
  ["forest", "Forest"],
  ["rust", "Rust"],
  ["graphite", "Graphite"],
];

type Props = { prefs: Prefs; onPrefs: (prefs: Prefs) => void };

export function Appearance({ prefs, onPrefs }: Props) {
  return (
    <div className="card panel">
      <h2>Appearance</h2>
      <div className="setting">
        <span className="setting__label">Theme</span>
        <div className="segmented" role="group" aria-label="Theme">
          {THEMES.map(([id, label]) => (
            <button key={id} type="button" aria-pressed={prefs.theme === id} onClick={() => onPrefs({ ...prefs, theme: id })}>
              {label}
            </button>
          ))}
        </div>
      </div>
      <div className="setting">
        <span className="setting__label">Accent</span>
        <div className="swatches" role="group" aria-label="Accent color">
          {ACCENTS.map(([id, label]) => (
            <button
              key={id}
              type="button"
              className="swatch"
              data-accent={id}
              aria-pressed={prefs.accent === id}
              aria-label={label}
              title={label}
              onClick={() => onPrefs({ ...prefs, accent: id })}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
