import type { AppConfig, HostSettings } from "../lib/api";
import { Field } from "../ui/Field";

type Props = { draft: AppConfig; onEdit: (change: Partial<AppConfig>) => void };

export function HostSection({ draft, onEdit }: Props) {
  const host = draft.host!;
  const setHost = (change: Partial<HostSettings>) => onEdit({ host: { ...host, ...change } });

  return (
    <div className="card panel">
      <h2>Host</h2>
      <dl className="summary">
        <dt>Library</dt>
        <dd>{host.library}</dd>
      </dl>
      <div className="row">
        <Field
          label="Port"
          value={String(host.port)}
          inputMode="numeric"
          onChange={(port) => setHost({ port: Number(port.replace(/\D/g, "")) || 0 })}
          hint="Change it here and on your router together, or clients won't find the host."
        />
        <Field
          label="Domain"
          value={host.domain ?? ""}
          placeholder="Leave empty for port forwarding"
          onChange={(domain) => setHost({ domain: domain.trim() || null })}
        />
      </div>
      {host.domain && (
        <Field label="Email for Let's Encrypt" value={host.email ?? ""} onChange={(email) => setHost({ email: email || null })} />
      )}
    </div>
  );
}
