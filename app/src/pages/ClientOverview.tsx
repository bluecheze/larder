import { useEffect, useState } from "react";
import { api, type Activity, type AppConfig, type ClientStatus, type ShelfSync } from "../lib/api";
import { ActivityList } from "./ActivityList";
import { PinTip } from "./PinTip";
import { ShelfRow } from "./ShelfRow";
import "./pages.css";

type Props = { config: AppConfig; onConfig: (config: AppConfig) => void };

export function ClientOverview({ config, onConfig }: Props) {
  const [status, setStatus] = useState<ClientStatus | null>(null);
  const [sync, setSync] = useState<Record<string, ShelfSync>>({});
  const [activity, setActivity] = useState<Activity[]>([]);

  useEffect(() => {
    const load = () => {
      api.clientStatus().then(setStatus);
      api.syncStatus().then(setSync);
      api.syncActivity().then(setActivity);
    };
    load();
    const timer = setInterval(load, 2500);
    return () => clearInterval(timer);
  }, []);

  const synced = new Set(config.client?.synced ?? []);
  const state = !status ? "off" : status.connected ? "on" : status.error ? "error" : "off";

  async function toggle(shelf: string, on: boolean) {
    onConfig(await api.setSynced(shelf, on));
  }

  return (
    <section className="page">
      <header className="page__head">
        <h1>Overview</h1>
        <p>Pick which shelves live on this computer. Those stay in sync both ways, on their own.</p>
      </header>
      <PinTip config={config} />
      <div className="card panel">
        <div className="panel__row">
          <p className="status" data-state={state}>
            {state === "on" ? `Connected to ${status?.host_name}` : state === "error" ? "Can't reach the host" : "Connecting..."}
          </p>
          <button type="button" className="btn" onClick={() => api.openFolder()}>
            Open Larder folder
          </button>
        </div>
        {status?.error && <p className="field__error">{status.error}</p>}
      </div>
      <div className="card panel">
        <h2>Shelves</h2>
        {status?.shelves.length ? (
          <ul className="list">
            {status.shelves.map((shelf) => (
              <ShelfRow key={shelf.name} shelf={shelf} synced={synced.has(shelf.name)} sync={sync[shelf.name]} onToggle={toggle} />
            ))}
          </ul>
        ) : (
          <p className="empty">The host has no shelves yet. Add one on the host under Shelves.</p>
        )}
      </div>
      <ActivityList items={activity} />
    </section>
  );
}
