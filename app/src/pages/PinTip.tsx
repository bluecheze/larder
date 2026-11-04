import { useEffect, useState } from "react";
import { api, type AppConfig } from "../lib/api";

const SEEN = "larder.pinTipSeen";

function seen(): boolean {
  try {
    return localStorage.getItem(SEEN) === "1";
  } catch {
    return false;
  }
}

// Pins the Larder folder the first time a client opens this page. Windows
// and Linux do it silently. macOS doesn't allow it anymore, so there we
// open the folder and show how to drag it into the sidebar.
export function PinTip({ config }: { config: AppConfig }) {
  const [manual, setManual] = useState(false);

  useEffect(() => {
    if (!config.prefs.show_in_sidebar || seen()) return;
    api
      .pinFolder()
      .then((result) => {
        if (result === "manual") setManual(true);
        else dismiss();
      })
      .catch(() => {});
  }, [config.prefs.show_in_sidebar]);

  function dismiss() {
    try {
      localStorage.setItem(SEEN, "1");
    } catch {}
    setManual(false);
  }

  if (!manual) return null;
  return (
    <div className="note note--row">
      <span>
        Finder opened your Larder folder. Drag it into the sidebar under Favorites to keep it one click away.
      </span>
      <button type="button" className="btn btn--small" onClick={dismiss}>
        Done
      </button>
    </div>
  );
}
