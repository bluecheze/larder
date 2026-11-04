import { useEffect, useState } from "react";
import { api, type AppConfig, type Defaults } from "./lib/api";
import { Setup } from "./setup/Setup";
import { Shell } from "./shell/Shell";

type Boot = { config: AppConfig | null; defaults: Defaults };

export function App() {
  const [boot, setBoot] = useState<Boot | null>(null);

  useEffect(() => {
    Promise.all([api.loadConfig(), api.defaults()]).then(([config, defaults]) => setBoot({ config, defaults }));
  }, []);

  if (!boot) return null;
  if (!boot.config) {
    return <Setup defaults={boot.defaults} onDone={(config) => setBoot({ ...boot, config })} />;
  }
  return <Shell config={boot.config} onReset={() => setBoot({ ...boot, config: null })} />;
}
