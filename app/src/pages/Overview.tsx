import { useEffect, useState } from "react";
import { api, type AppConfig, type HostStatus } from "../lib/api";
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
  return (
    <section className="page">
      <header className="page__head">
        <h1>Overview</h1>
        <p>This computer is a client.</p>
      </header>
      <div className="card panel">
        <p className="status">Not connected yet</p>
        <p>
          Pairing with the host and the Larder drive are the next pieces being built. Your invite is saved and will be used
          as soon as they land.
        </p>
      </div>
    </section>
  );
}
