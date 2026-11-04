import { api, type AppConfig } from "../lib/api";
import "./pages.css";

type Props = { config: AppConfig; onReset: () => void };

export function Settings({ config, onReset }: Props) {
  async function changeRole() {
    const other = config.role === "host" ? "a client" : "the host";
    if (!confirm(`Set this computer up as ${other}? Your files stay where they are.`)) return;
    await api.resetConfig();
    onReset();
  }

  return (
    <section className="page">
      <header className="page__head">
        <h1>Settings</h1>
      </header>
      <div className="card panel">
        <h2>This computer</h2>
        <dl className="summary">
          <dt>Name</dt>
          <dd>{config.device_name}</dd>
          <dt>Role</dt>
          <dd>{config.role === "host" ? "Host" : "Client"}</dd>
          {config.host && (
            <>
              <dt>Library</dt>
              <dd>{config.host.library}</dd>
              <dt>Port</dt>
              <dd>{config.host.port}</dd>
            </>
          )}
        </dl>
        <div>
          <button type="button" className="btn" onClick={changeRole}>
            Change role
          </button>
        </div>
        <p className="field__hint">Runs setup again. The library and anything already synced stay on disk.</p>
      </div>
    </section>
  );
}
