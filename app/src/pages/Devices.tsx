import { useCallback, useEffect, useState } from "react";
import { api, type Device } from "../lib/api";
import { ago } from "../lib/format";
import { InviteMaker } from "./InviteMaker";
import "./pages.css";

export function Devices() {
  const [devices, setDevices] = useState<Device[]>([]);
  const refresh = useCallback(() => api.listDevices().then(setDevices), []);

  useEffect(() => {
    refresh();
    const timer = setInterval(refresh, 4000);
    return () => clearInterval(timer);
  }, [refresh]);

  async function revoke(device: Device) {
    if (!confirm(`Remove ${device.name}? It won't be able to reach this host until you invite it again.`)) return;
    await api.revokeDevice(device.id);
    refresh();
  }

  return (
    <section className="page">
      <header className="page__head">
        <h1>Devices</h1>
        <p>Computers that are allowed to use this host.</p>
      </header>
      <InviteMaker />
      <div className="card panel">
        <h2>Paired</h2>
        {devices.length === 0 ? (
          <p className="empty">Nothing paired yet. Make an invite above and paste it into Larder on the other computer.</p>
        ) : (
          <ul className="list">
            {devices.map((device) => (
              <li key={device.id}>
                <span className="list__main">
                  <strong>{device.name}</strong>
                  <small>Last seen {ago(device.last_seen)}</small>
                </span>
                <button type="button" className="btn btn--danger" onClick={() => revoke(device)}>
                  Remove
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </section>
  );
}
