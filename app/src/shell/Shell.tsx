import { useState } from "react";
import type { AppConfig } from "../lib/api";
import { ClientOverview } from "../pages/ClientOverview";
import { Devices } from "../pages/Devices";
import { HostOverview } from "../pages/HostOverview";
import { Shelves } from "../pages/Shelves";
import { Settings } from "../settings/Settings";
import { Sidebar, type Page } from "./Sidebar";
import "./shell.css";

type Props = { config: AppConfig; onConfig: (config: AppConfig | null) => void };

export function Shell({ config, onConfig }: Props) {
  const [page, setPage] = useState<Page>("overview");
  const overview = config.role === "host" ? <HostOverview config={config} /> : <ClientOverview config={config} onConfig={onConfig} />;
  return (
    <div className="shell">
      <Sidebar config={config} page={page} onPage={setPage} />
      <main className="shell__main">
        {page === "overview" && overview}
        {page === "shelves" && <Shelves />}
        {page === "devices" && <Devices />}
        {page === "settings" && <Settings config={config} onConfig={onConfig} />}
      </main>
    </div>
  );
}
