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
};

export type AppConfig = {
  role: Role;
  device_name: string;
  host: HostSettings | null;
  client: ClientSettings | null;
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
};
