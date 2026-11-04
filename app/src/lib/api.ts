import { invoke } from "@tauri-apps/api/core";

export type Role = "host" | "client";

export type HostSettings = {
  library: string;
  port: number;
  domain: string | null;
  email: string | null;
};

export type Connection = {
  address: string;
  fingerprint: string | null;
  host_name: string;
  device_id: string;
};

export type ClientSettings = {
  connection: Connection;
  folder: string | null;
  synced: string[];
  check_every_secs: number;
};

export type Theme = "system" | "light" | "dark";
export type Accent = "cobalt" | "forest" | "rust" | "graphite";

export type Prefs = {
  theme: Theme;
  accent: Accent;
  show_in_sidebar: boolean;
  ignore: string[];
  start_at_login: boolean;
};

export type AppConfig = {
  role: Role;
  device_name: string;
  host: HostSettings | null;
  client: ClientSettings | null;
  prefs: Prefs;
};

export type Defaults = {
  device_name: string;
  library: string;
  port: number;
};

export type HostStatus = {
  running: boolean;
  error: string | null;
  port: number;
  addresses: string[];
  domain: string | null;
};

export type Shelf = {
  name: string;
  files: number;
  bytes: number;
};

export type ClientStatus = {
  connected: boolean;
  host_name: string | null;
  address: string | null;
  error: string | null;
  shelves: Shelf[];
};

export type ShelfSync = {
  state: "idle" | "syncing" | "error";
  last_sync: number | null;
  error: string | null;
};

export type ChangeKind = "uploaded" | "downloaded" | "deleted" | "removed" | "conflict";

export type Activity = {
  at: number;
  shelf: string;
  path: string;
  kind: ChangeKind;
};

export type Device = {
  id: string;
  name: string;
  paired_at: number;
  last_seen: number;
};

// Thin wrappers so components never deal with command name strings.
export const api = {
  loadConfig: () => invoke<AppConfig | null>("load_config"),
  saveConfig: (config: AppConfig) => invoke<void>("save_config", { config }),
  joinHost: (invite: string, deviceName: string) => invoke<AppConfig>("join_host", { invite, deviceName }),
  resetConfig: () => invoke<void>("reset_config"),
  defaults: () => invoke<Defaults>("defaults"),
  hostStatus: () => invoke<HostStatus>("host_status"),
  createInvite: (address: string) => invoke<string>("create_invite", { address }),
  listDevices: () => invoke<Device[]>("list_devices"),
  revokeDevice: (id: string) => invoke<boolean>("revoke_device", { id }),
  hostShelves: () => invoke<string[]>("host_shelves"),
  createShelf: (name: string) => invoke<void>("create_shelf", { name }),
  clientStatus: () => invoke<ClientStatus>("client_status"),
  saveSettings: (edit: AppConfig) => invoke<AppConfig>("save_settings", { edit }),
  setSynced: (shelf: string, on: boolean) => invoke<AppConfig>("set_synced", { shelf, on }),
  syncNow: (shelf: string) => invoke<void>("sync_now", { shelf }),
  syncStatus: () => invoke<Record<string, ShelfSync>>("sync_status"),
  syncActivity: () => invoke<Activity[]>("sync_activity"),
  larderFolder: () => invoke<string | null>("larder_folder"),
  openFolder: (shelf?: string) => invoke<void>("open_folder", { shelf: shelf ?? null }),
  pinFolder: () => invoke<"done" | "manual">("pin_folder"),
};
