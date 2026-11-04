import { api, type Shelf, type ShelfSync } from "../lib/api";
import { ago, bytes, files } from "../lib/format";
import { Icon } from "../ui/Icon";
import { Switch } from "../ui/Switch";

type Props = {
  shelf: Shelf;
  synced: boolean;
  sync: ShelfSync | undefined;
  onToggle: (shelf: string, on: boolean) => void;
};

function syncLabel(synced: boolean, sync: ShelfSync | undefined): string {
  if (!synced) return "Only on the host";
  if (!sync) return "Starting...";
  if (sync.state === "syncing") return "Syncing...";
  if (sync.state === "error") return `Sync paused: ${sync.error}`;
  return sync.last_sync ? `Synced ${ago(sync.last_sync)}` : "Synced";
}

export function ShelfRow({ shelf, synced, sync, onToggle }: Props) {
  return (
    <li>
      <Icon name="folder" size={18} />
      <span className="list__main">
        <strong>{shelf.name}</strong>
        <small data-error={sync?.state === "error" || undefined}>
          {files(shelf.files)} · {bytes(shelf.bytes)} · {syncLabel(synced, sync)}
        </small>
      </span>
      {synced && (
        <>
          <button type="button" className="btn btn--small" onClick={() => api.syncNow(shelf.name)}>
            <Icon name="reload" size={16} /> Sync now
          </button>
          <button type="button" className="btn btn--small" onClick={() => api.openFolder(shelf.name)}>
            Open
          </button>
        </>
      )}
      <Switch on={synced} onChange={(on) => onToggle(shelf.name, on)} label={`Keep ${shelf.name} on this computer`} />
    </li>
  );
}
