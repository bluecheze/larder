import type { Activity, ChangeKind } from "../lib/api";
import { ago } from "../lib/format";

const VERBS: Record<ChangeKind, string> = {
  uploaded: "Sent",
  downloaded: "Got",
  deleted: "Deleted on host",
  removed: "Removed here",
  conflict: "Kept both versions of",
};

export function ActivityList({ items }: { items: Activity[] }) {
  return (
    <div className="card panel">
      <h2>Recent activity</h2>
      {items.length === 0 ? (
        <p className="empty">Nothing yet. Changes show up here as they sync.</p>
      ) : (
        <ul className="list list--tight">
          {items.slice(0, 15).map((item, i) => (
            <li key={`${item.at}-${i}`}>
              <span className="list__main">
                <span>
                  {VERBS[item.kind]} <span className="mono">{item.path}</span>
                </span>
                <small>
                  {item.shelf} · {ago(item.at)}
                </small>
              </span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
