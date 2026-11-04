# Roadmap

Larder is a self-hosted drive. One computer is the **host** and keeps the files. Your other computers are **clients** that sync folders with it. Runs on macOS, Linux and Windows.

## How it fits together

```
crates/
  larder-core/     shared types: paths, manifests, the sync planner, invites
  larder-host/     the server that runs on the host (axum + rustls)
  larder-client/   talks to a host and runs the sync loop
app/
  src/             React UI (setup, overview, shelves, devices, activity, settings)
  src-tauri/       Tauri shell, config, wires host/client into the app
.github/workflows/ CI on all three platforms, release builds
```

Words used everywhere:

- **library**: the folder on the host where everything lives
- **shelf**: one synced folder inside the library (Documents, Photos, ...)
- **invite**: a code the host makes so a client can pair. It carries the address, the TLS fingerprint and a one-time token

## Build order

1. Workspace, README, license
2. `larder-core`: safe relative paths, folder scanning with blake3 hashes
3. `larder-core`: the sync planner (local vs host vs last synced state)
4. `larder-core`: invite codes
5. `larder-host`: device store and pairing
6. `larder-host`: shelves, manifest and file routes, with conflict checks on upload
7. `larder-host`: TLS, self-signed with a pinned fingerprint or Let's Encrypt for a domain
8. `larder-client`: API client with fingerprint pinning
9. `larder-client`: sync engine, conflict copies, file watcher
10. End to end tests: one host, two clients
11. Tauri app: config, first launch setup (defaults to host, can switch to client)
12. UI pages: overview, shelves, devices, activity, settings (role can be changed)
13. CI on macOS, Linux, Windows and release bundles

## Not in v1

- LAN auto discovery (you paste an invite instead)
- Relays or NAT punching. You need a forwarded port or a domain pointing at the host
- Selective sync of single files inside a shelf
- Encryption at rest on the host
- Mobile apps
