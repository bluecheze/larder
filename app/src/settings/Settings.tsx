import { useEffect, useState } from "react";
import { api, type AppConfig } from "../lib/api";
import { applyTheme } from "../lib/theme";
import { Appearance } from "./Appearance";
import { Computer } from "./Computer";
import { HostSection } from "./HostSection";
import { RoleSection } from "./RoleSection";
import { SyncSection } from "./SyncSection";
import "../pages/pages.css";
import "./settings.css";

type Props = { config: AppConfig; onConfig: (config: AppConfig | null) => void };

export function Settings({ config, onConfig }: Props) {
  const [draft, setDraft] = useState<AppConfig>(config);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => setDraft(config), [config]);
  // Theme and accent preview live, the rest waits for Save.
  useEffect(() => applyTheme(draft.prefs), [draft.prefs]);
  useEffect(() => () => applyTheme(config.prefs), [config.prefs]);

  const dirty = JSON.stringify(draft) !== JSON.stringify(config);
  const edit = (change: Partial<AppConfig>) => {
    setDraft((d) => ({ ...d, ...change }));
    setSaved(false);
  };

  async function save() {
    setError(null);
    try {
      onConfig(await api.saveSettings(draft));
      setSaved(true);
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <section className="page">
      <header className="page__head">
        <h1>Settings</h1>
      </header>
      <Appearance prefs={draft.prefs} onPrefs={(prefs) => edit({ prefs })} />
      <Computer draft={draft} onEdit={edit} />
      {draft.client && <SyncSection draft={draft} onEdit={edit} />}
      {draft.host && <HostSection draft={draft} onEdit={edit} />}
      <RoleSection config={config} onReset={() => onConfig(null)} />
      <footer className="save-bar" data-visible={dirty || saved || undefined}>
        {error && <p className="field__error">{error}</p>}
        {saved && !dirty && <span className="field__hint">Saved</span>}
        <button type="button" className="btn" onClick={() => setDraft(config)} disabled={!dirty}>
          Discard
        </button>
        <button type="button" className="btn btn--primary" onClick={save} disabled={!dirty}>
          Save changes
        </button>
      </footer>
    </section>
  );
}
