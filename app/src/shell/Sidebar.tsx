import jar from "../assets/icon-shelf.png";
import type { AppConfig } from "../lib/api";
import { Icon, type IconName } from "../ui/Icon";

export type Page = "overview" | "shelves" | "devices" | "settings";

const HOST_PAGES: [Page, string, IconName][] = [
  ["overview", "Overview", "server"],
  ["shelves", "Shelves", "folder"],
  ["devices", "Devices", "laptop"],
  ["settings", "Settings", "sliders"],
];

const CLIENT_PAGES = HOST_PAGES.filter(([page]) => page === "overview" || page === "settings");

type Props = { config: AppConfig; page: Page; onPage: (page: Page) => void };

export function Sidebar({ config, page, onPage }: Props) {
  const pages = config.role === "host" ? HOST_PAGES : CLIENT_PAGES;
  return (
    <nav className="sidebar" aria-label="Main">
      <div className="mark">
        <img src={jar} alt="" />
        larder
      </div>
      <ul className="sidebar__links">
        {pages.map(([id, label, icon]) => (
          <li key={id}>
            <button
              type="button"
              className="sidebar__link"
              aria-current={page === id ? "page" : undefined}
              onClick={() => onPage(id)}
            >
              <Icon name={icon} size={18} /> {label}
            </button>
          </li>
        ))}
      </ul>
      <div className="sidebar__me">
        <span className="sidebar__dot" data-role={config.role} />
        <span>
          <strong>{config.device_name}</strong>
          <small>{config.role === "host" ? "Host" : "Client"}</small>
        </span>
      </div>
    </nav>
  );
}
