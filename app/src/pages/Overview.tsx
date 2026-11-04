import { useEffect, useState } from "react";
import { api, type AppConfig, type ClientStatus, type HostStatus } from "../lib/api";
import { bytes, files } from "../lib/format";
import { Icon } from "../ui/Icon";
import "./pages.css";

export function Overview({ config }: { config: AppConfig }) {
  return config.role === "host" ? <HostOverview config={config} /> : <ClientOverview />;
}

function HostOverview({ config }: { config: AppConfig }) {
  const [status, setStatus] = useState<HostStatus | null>(null);

  useEffect(() => {
    const load = () => api.hostStatus().then(setStatus);
    load();
    const timer = setInterval(load, 3000);
    return () => clearInterval(timer);
  }, []);

  const state = !status ? "off" : status.error ? "error" : status.running ? "on" : "off";
  return (
    <section className="page">
      <header className="page__head">
        <h1>Overview</h1>
        <p>This computer keeps the library. Other computers reach it through an invite.</p>
      </header>
      <div className="card panel">
        <p className="status" data-state={state}>
          {state === "on" ? "Host is running" : state === "error" ? "Host couldn't start" : "Starting up..."}
        </p>
        {status?.error && <p className="field__error">{status.error}</p>}
        {status?.running && (
          <dl className="summary">
            <dt>Library</dt>
            <dd>{config.host?.library}</dd>
            <dt>Port</dt>
            <dd>{status.port}</dd>
            <dt>On this network</dt>
            <dd className="mono">{status.addresses.map((a) => `${a}:${status.port}`).join("  ") || "no network found"}</dd>
            {status.domain && (
              <>
                <dt>Domain</dt>
                <dd className="mono">{status.domain}</dd>
              </>
            )}
          </dl>
        )}
      </div>
    </section>
  );
}

function ClientOverview() {
  const [status, setStatus] = useState<ClientStatus | null>(null);

  useEffect(() => {
    const load = () => api.clientStatus().then(setStatus);
    load();
    const timer = setInterval(load, 5000);
    return () => clearInterval(timer);
  }, []);

  const state = !status ? "off" : status.connected ? "on" : status.error ? "error" : "off";
  return (
    <section className="page">
      <header className="page__head">
        <h1>Overview</h1>
        <p>This computer uses files from {status?.host_name ?? "your host"}.</p>
      </header>
      <div className="card panel">
        <p className="status" data-state={state}>
          {state === "on" ? `Connected to ${status?.host_name}` : state === "error" ? "Can't reach the host" : "Connecting..."}
        </p>
        {status?.error && <p className="field__error">{status.error}</p>}
        {status?.address && <p className="mono field__hint">{status.address}</p>}
      </div>
      <div className="card panel">
        <h2>Shelves on the host</h2>
        {status?.shelves.length ? (
          <ul className="list">
            {status.shelves.map((shelf) => (
              <li key={shelf.name}>
                <Icon name="folder" size={18} />
                <span className="list__main">
                  <strong>{shelf.name}</strong>
                  <small>
                    {files(shelf.files)} · {bytes(shelf.bytes)}
                  </small>
                </span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="empty">The host has no shelves yet. Add one on the host under Shelves.</p>
        )}
        <p className="field__hint">The Larder drive that mounts these in Finder or Explorer is coming next.</p>
      </div>
    </section>
  );
}
