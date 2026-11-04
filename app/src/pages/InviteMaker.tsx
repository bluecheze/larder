import { useEffect, useState } from "react";
import { api, type HostStatus } from "../lib/api";
import { Field } from "../ui/Field";
import { Icon } from "../ui/Icon";

export function InviteMaker() {
  const [status, setStatus] = useState<HostStatus | null>(null);
  const [address, setAddress] = useState("");
  const [invite, setInvite] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    api.hostStatus().then((s) => {
      setStatus(s);
      setAddress(s.domain ?? s.addresses[0] ?? "");
    });
  }, []);

  async function make() {
    setError(null);
    setCopied(false);
    try {
      setInvite(await api.createInvite(address));
    } catch (e) {
      setError(String(e));
    }
  }

  async function copy() {
    if (!invite) return;
    await navigator.clipboard.writeText(invite);
    setCopied(true);
  }

  const options = [...(status?.domain ? [status.domain] : []), ...(status?.addresses ?? [])];
  return (
    <div className="card panel">
      <h2>Invite a computer</h2>
      <p>Pick the address the other computer should use. Same network: use a local one. Elsewhere: type your public IP.</p>
      {options.length > 0 && (
        <div className="chips" role="group" aria-label="Suggested addresses">
          {options.map((option) => (
            <button
              key={option}
              type="button"
              className="chip mono"
              aria-pressed={address === option}
              onClick={() => setAddress(option)}
            >
              {option}
            </button>
          ))}
        </div>
      )}
      <div className="inline">
        <Field label="Address" value={address} onChange={setAddress} placeholder="192.168.1.20 or files.example.com" />
        <button type="button" className="btn btn--primary" onClick={make} disabled={!status?.running}>
          <Icon name="link" size={18} /> Make invite
        </button>
      </div>
      {error && <p className="field__error">{error}</p>}
      {invite && (
        <>
          <p className="invite-code mono">{invite}</p>
          <div className="inline">
            <button type="button" className="btn" onClick={copy}>
              <Icon name={copied ? "check" : "copy"} size={18} /> {copied ? "Copied" : "Copy invite"}
            </button>
            <span className="field__hint">Works for one computer, for the next 15 minutes.</span>
          </div>
        </>
      )}
    </div>
  );
}
