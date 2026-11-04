import { useEffect, useState } from "react";
import { api, type AppConfig, type HostStatus } from "../lib/api";
import "./pages.css";

export function HostOverview({ config }: { config: AppConfig }) {
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
