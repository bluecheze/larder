import { Choice } from "../ui/Choice";
import { Field } from "../ui/Field";
import { Icon } from "../ui/Icon";
import type { Draft } from "./steps";

type Props = { draft: Draft; onChange: (patch: Partial<Draft>) => void };

export function NetworkStep({ draft, onChange }: Props) {
  return (
    <>
      <h2>How will clients reach it?</h2>
      <div className="choices" role="group" aria-label="Connection">
        <Choice
          selected={!draft.useDomain}
          onSelect={() => onChange({ useDomain: false })}
          title="Port forwarding"
          art={<Icon name="wifi" size={40} />}
        >
          Same network works right away. For outside, forward the port below on your router.
        </Choice>
        <Choice
          selected={draft.useDomain}
          onSelect={() => onChange({ useDomain: true })}
          title="My own domain"
          art={<Icon name="globe" size={40} />}
        >
          Point a domain at your home IP and forward port 443. Larder gets the certificate for you.
        </Choice>
      </div>
      <Field
        label="Port"
        value={draft.port}
        inputMode="numeric"
        onChange={(port) => onChange({ port: port.replace(/\D/g, "") })}
      />
      {draft.useDomain && (
        <div className="row">
          <Field label="Domain" value={draft.domain} placeholder="files.example.com" onChange={(domain) => onChange({ domain })} />
          <Field label="Email for Let's Encrypt" value={draft.email} inputMode="email" onChange={(email) => onChange({ email })} />
        </div>
      )}
    </>
  );
}
