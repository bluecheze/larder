import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { api, type AppConfig, type ClientSettings } from "../lib/api";
import { Field } from "../ui/Field";
import { Switch } from "../ui/Switch";

const INTERVALS: [number, string][] = [
  [15, "Every 15 seconds"],
  [30, "Every 30 seconds"],
  [60, "Every minute"],
  [300, "Every 5 minutes"],
  [900, "Every 15 minutes"],
];

type Props = { draft: AppConfig; onEdit: (change: Partial<AppConfig>) => void };

export function SyncSection({ draft, onEdit }: Props) {
  const client = draft.client!;
  const [resolved, setResolved] = useState("");
  const [ignoreText, setIgnoreText] = useState(draft.prefs.ignore.join("\n"));
  const [pinned, setPinned] = useState<string | null>(null);

  useEffect(() => {
    api.larderFolder().then((f) => setResolved(f ?? ""));
  }, []);
  useEffect(() => setIgnoreText(draft.prefs.ignore.join("\n")), [draft.prefs.ignore]);

  const setClient = (change: Partial<ClientSettings>) => onEdit({ client: { ...client, ...change } });

  async function chooseFolder() {
    const picked = await open({ directory: true, defaultPath: client.folder ?? resolved });
    if (typeof picked === "string") setClient({ folder: picked });
  }

  async function pinNow() {
    const result = await api.pinFolder();
    setPinned(result === "done" ? "Added to the sidebar." : "Finder opened the folder, drag it into the sidebar.");
  }

  function commitIgnore(text: string) {
    setIgnoreText(text);
    const ignore = text.split("\n").map((line) => line.trim()).filter(Boolean);
    onEdit({ prefs: { ...draft.prefs, ignore } });
  }

  return (
    <div className="card panel">
      <h2>Sync</h2>
      <div className="row">
        <Field
          label="Larder folder"
          value={client.folder ?? resolved}
          onChange={(folder) => setClient({ folder: folder || null })}
          hint="Each shelf you keep on this computer becomes a folder in here. Moving it starts those shelves fresh."
        />
        <button type="button" className="btn" onClick={chooseFolder}>
          Choose
        </button>
      </div>
      <div className="setting">
        <span>
          <span className="setting__label">Show in the sidebar</span>
          <small className="field__hint">{pinned ?? "Puts the Larder folder next to Downloads in Finder or Explorer."}</small>
        </span>
        <span className="setting__actions">
          <button type="button" className="btn btn--small" onClick={pinNow}>
            Pin now
          </button>
          <Switch
            on={draft.prefs.show_in_sidebar}
            onChange={(show_in_sidebar) => onEdit({ prefs: { ...draft.prefs, show_in_sidebar } })}
            label="Show the Larder folder in the sidebar"
          />
        </span>
      </div>
      <div className="setting">
        <span>
          <span className="setting__label">Check the host for changes</span>
          <small className="field__hint">Your own edits sync right away, this is for changes from other computers.</small>
        </span>
        <select
          className="select"
          value={client.check_every_secs}
          onChange={(e) => setClient({ check_every_secs: Number(e.target.value) })}
          aria-label="How often to check the host"
        >
          {INTERVALS.map(([secs, label]) => (
            <option key={secs} value={secs}>
              {label}
            </option>
          ))}
        </select>
      </div>
      <div className="field">
        <label className="field__label" htmlFor="ignore">
          Don't sync
        </label>
        <textarea
          id="ignore"
          className="textarea"
          value={ignoreText}
          placeholder={"node_modules\n*.tmp\n~$*"}
          spellCheck={false}
          onChange={(e) => commitIgnore(e.target.value)}
        />
        <span className="field__hint">One name per line, * matches anything. Ignored files are left alone on both sides, never deleted.</span>
      </div>
    </div>
  );
}
