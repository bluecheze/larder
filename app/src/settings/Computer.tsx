import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { useEffect, useState } from "react";
import type { AppConfig } from "../lib/api";
import { Field } from "../ui/Field";
import { Switch } from "../ui/Switch";

type Props = { draft: AppConfig; onEdit: (change: Partial<AppConfig>) => void };

export function Computer({ draft, onEdit }: Props) {
  // Start at login is applied right away instead of waiting for Save. It's
  // an OS setting, not ours, so the switch shows what the OS really has.
  const [atLogin, setAtLogin] = useState(draft.prefs.start_at_login);

  useEffect(() => {
    isEnabled().then(setAtLogin).catch(() => {});
  }, []);

  async function toggleLogin(on: boolean) {
    await (on ? enable() : disable());
    setAtLogin(on);
    onEdit({ prefs: { ...draft.prefs, start_at_login: on } });
  }

  return (
    <div className="card panel">
      <h2>This computer</h2>
      <Field label="Name" value={draft.device_name} onChange={(device_name) => onEdit({ device_name })} />
      <div className="setting">
        <span>
          <span className="setting__label">Start at login</span>
          <small className="field__hint">Keeps your shelves syncing without opening Larder by hand.</small>
        </span>
        <Switch on={atLogin} onChange={toggleLogin} label="Start Larder at login" />
      </div>
    </div>
  );
}
