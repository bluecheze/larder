import type { AppConfig, Defaults, Role } from "../lib/api";

export type StepId = "role" | "name" | "library" | "network" | "invite" | "done";

export type Draft = {
  role: Role;
  deviceName: string;
  library: string;
  port: string;
  useDomain: boolean;
  domain: string;
  email: string;
  invite: string;
};

const HOST_STEPS: StepId[] = ["role", "name", "library", "network", "done"];
const CLIENT_STEPS: StepId[] = ["role", "name", "invite", "done"];

export function stepsFor(role: Role): StepId[] {
  return role === "host" ? HOST_STEPS : CLIENT_STEPS;
}

// Setup assumes you're on the host. You'd set that up first anyway, since
// a client needs an invite from it.
export function initialDraft(defaults: Defaults): Draft {
  return {
    role: "host",
    deviceName: defaults.device_name,
    library: defaults.library,
    port: String(defaults.port),
    useDomain: false,
    domain: "",
    email: "",
    invite: "",
  };
}

const DOMAIN = /^(?!-)[a-z0-9-]{1,63}(?<!-)(\.(?!-)[a-z0-9-]{1,63}(?<!-))+$/i;

export function problem(step: StepId, draft: Draft): string | null {
  switch (step) {
    case "name":
      return draft.deviceName.trim() ? null : "Give this computer a name so you can tell it apart later.";
    case "library":
      return draft.library.trim() ? null : "Pick a folder for the library.";
    case "network": {
      const port = Number(draft.port);
      if (!Number.isInteger(port) || port < 1024 || port > 65535) {
        return "Use a port between 1024 and 65535.";
      }
      if (draft.useDomain && !DOMAIN.test(draft.domain.trim())) {
        return "That doesn't look like a domain. Something like files.example.com.";
      }
      if (draft.useDomain && !draft.email.includes("@")) {
        return "Let's Encrypt wants an email for expiry warnings.";
      }
      return null;
    }
    case "invite":
      return draft.invite.trim().startsWith("larder://") ? null : "Invites start with larder:// and come from your host.";
    default:
      return null;
  }
}

// Clients don't build their config here, pairing with the host does that
// on the Rust side and hands back the result.
export function toHostConfig(draft: Draft): AppConfig {
  return {
    role: "host",
    device_name: draft.deviceName.trim(),
    client: null,
    host: {
      library: draft.library.trim(),
      port: Number(draft.port),
      domain: draft.useDomain ? draft.domain.trim().toLowerCase() : null,
      email: draft.useDomain ? draft.email.trim() : null,
    },
  };
}
