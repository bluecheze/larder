import { invoke } from "@tauri-apps/api/core";

export type Role = "host" | "client";

export type HostSettings = {
  library: string;
  port: number;
  domain: string | null;
  email: string | null;
};

export type ClientSettings = {
  invite: string;
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

// Thin wrappers so components never deal with command name strings.
export const api = {
  loadConfig: () => invoke<AppConfig | null>("load_config"),
  saveConfig: (config: AppConfig) => invoke<void>("save_config", { config }),
  resetConfig: () => invoke<void>("reset_config"),
  defaults: () => invoke<Defaults>("defaults"),
};
