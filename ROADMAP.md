# Roadmap

Larder is a self-hosted drive. One computer is the **host** and keeps the files. Your other computers are **clients**. By default a client only downloads a file when you open it.

## How it fits together

```
crates/
  larder-core/     shared types: paths, manifests, the sync planner, invites
  larder-host/     the server that runs on the host (axum + rustls)
  larder-client/   talks to a host: the on-demand drive, offline shelves
app/
  src/             React UI (setup, overview, shelves, devices, activity, settings)
  src-tauri/       Tauri shell, config, wires host/client into the app
art/               banner and UI pixel art
.github/workflows/ CI on all three platforms, release builds
```

Words used everywhere:

- **library**: the folder on the host where everything lives
- **shelf**: one folder inside the library (Documents, Photos, ...)
- **invite**: a code the host makes so a client can pair. It carries the address, the TLS fingerprint and a one-time token
- **larder drive**: the mounted drive on a client, files stream in when opened
- **offline shelf**: a shelf the client keeps a full synced copy of

## Why a WebDAV drive for files on demand

Native placeholders (Windows Cloud Files, macOS File Provider) need per-OS code, and on macOS a signed app extension. Linux has nothing equivalent. Every OS can mount a WebDAV share without extra drivers, so the client runs a small WebDAV server on localhost that forwards to the host and caches what you open. Native placeholders on Windows are the first thing on the list after v1.

## Build order

1. Workspace, README, license
2. `larder-core`: safe relative paths, folder scanning with blake3 hashes
3. `larder-core`: the sync planner (local vs host vs last synced state)
4. `larder-core`: invite codes and shared API types
5. `larder-host`: device store and pairing
6. `larder-host`: shelves, listing, file routes (range reads, uploads with conflict checks, rename, delete)
7. `larder-host`: TLS, self-signed with a pinned fingerprint or Let's Encrypt for a domain
8. `larder-client`: API client with fingerprint pinning
9. `larder-client`: on-demand drive (local WebDAV server, size-capped cache, auto mount per OS)
10. `larder-client`: offline shelves (sync engine, conflict copies, file watcher)
11. End to end tests: one host, two clients
12. Tauri app: config, first launch setup (defaults to host, can switch to client)
13. UI pages: overview, shelves, devices, activity, settings (role can be changed)
14. CI on macOS, Linux, Windows and release bundles

## Not in v1

- Native placeholders (Windows Cloud Files, macOS File Provider)
- LAN auto discovery (you paste an invite instead)
- Relays or NAT punching. You need a forwarded port or a domain pointing at the host
- Encryption at rest on the host
- Mobile apps
