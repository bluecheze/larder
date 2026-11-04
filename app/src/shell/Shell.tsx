import { api, type AppConfig } from "../lib/api";
import { Icon } from "../ui/Icon";
import "./shell.css";

type Props = { config: AppConfig; onReset: () => void };

// Placeholder until the real pages land. Enough to see setup worked and
// to switch roles while testing.
export function Shell({ config, onReset }: Props) {
  async function changeRole() {
    await api.resetConfig();
    onReset();
  }

  return (
    <main className="shell">
      <section className="card shell__card">
        <h1>{config.device_name}</h1>
        <p className="shell__role">
          <Icon name={config.role === "host" ? "server" : "laptop"} /> This computer is a {config.role}
        </p>
        {config.host && <p>Library: {config.host.library}</p>}
        <button type="button" className="btn" onClick={changeRole}>
          Change role
        </button>
      </section>
    </main>
  );
}
