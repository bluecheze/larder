import { useState } from "react";
import type { AppConfig } from "../lib/api";
import { Devices } from "../pages/Devices";
import { Overview } from "../pages/Overview";
import { Settings } from "../pages/Settings";
import { Shelves } from "../pages/Shelves";
import { Sidebar, type Page } from "./Sidebar";
import "./shell.css";

type Props = { config: AppConfig; onReset: () => void };

export function Shell({ config, onReset }: Props) {
  const [page, setPage] = useState<Page>("overview");
  return (
    <div className="shell">
      <Sidebar config={config} page={page} onPage={setPage} />
      <main className="shell__main">
        {page === "overview" && <Overview config={config} />}
        {page === "shelves" && <Shelves />}
        {page === "devices" && <Devices />}
        {page === "settings" && <Settings config={config} onReset={onReset} />}
      </main>
    </div>
  );
}
