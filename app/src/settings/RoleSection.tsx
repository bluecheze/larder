import { api, type AppConfig } from "../lib/api";

type Props = { config: AppConfig; onReset: () => void };

export function RoleSection({ config, onReset }: Props) {
  async function changeRole() {
    const other = config.role === "host" ? "a client" : "the host";
    if (!confirm(`Set this computer up as ${other}? Your files stay where they are.`)) return;
    await api.resetConfig();
    onReset();
  }

  return (
    <div className="card panel">
      <h2>Role</h2>
      <div className="setting">
        <span>
          <span className="setting__label">This computer is {config.role === "host" ? "the host" : "a client"}</span>
          <small className="field__hint">Runs setup again. The library and anything already synced stay on disk.</small>
        </span>
        <button type="button" className="btn" onClick={changeRole}>
          Change role
        </button>
      </div>
    </div>
  );
}
