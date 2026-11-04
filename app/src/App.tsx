import { useEffect, useState } from "react";
import { api, type AppConfig, type Defaults } from "./lib/api";
import { applyTheme } from "./lib/theme";
import { Setup } from "./setup/Setup";
import { Shell } from "./shell/Shell";

type Boot = { config: AppConfig | null; defaults: Defaults };

export function App() {
  const [boot, setBoot] = useState<Boot | null>(null);

  useEffect(() => {
    Promise.all([api.loadConfig(), api.defaults()]).then(([config, defaults]) => setBoot({ config, defaults }));
  }, []);

  useEffect(() => applyTheme(boot?.config?.prefs), [boot?.config?.prefs]);

  if (!boot) return null;
  const setConfig = (config: AppConfig | null) => setBoot({ ...boot, config });
  if (!boot.config) {
    return <Setup defaults={boot.defaults} onDone={setConfig} />;
  }
  return <Shell config={boot.config} onConfig={setConfig} />;
}
