<p align="center">
  <img src="assets/banner.png" alt="Pixel art of a boy with an orange backpack putting a box on a pantry shelf full of jars and crates" width="100%">
</p>

<p align="center">
  Your own drive, on your own computer.<br>
  macOS · Linux · Windows
</p>

<img src="assets/divider.png" alt="" width="100%">

## What it is

Larder turns one of your computers into a private cloud drive for the rest of them.

The **host** keeps every file. Your other machines are **clients**: they see everything on the host as a normal drive in Finder, Explorer or your Linux file manager, but a file only comes down when you open it. Nothing sits on a client's disk unless you asked for it.

No account, no subscription, no storage cap other than the disk in your host.

> Larder is being built in the open and isn't ready for files you can't lose yet. [ROADMAP.md](ROADMAP.md) shows where it's at.

<img src="assets/divider.png" alt="" width="100%">

## How it works

| | |
|---|---|
| **Library** | The folder on the host where everything lives. |
| **Shelves** | Folders inside the library: Documents, Photos, Projects, whatever you like. |
| **Larder drive** | On a client, every shelf shows up as a mounted drive. Opening a file streams it from the host and keeps a copy in a small cache, which empties itself when it gets full. |
| **Keep on this device** | Mark a shelf as offline and the client keeps a full copy that syncs both ways, so it works with no connection. |
| **Invites** | The host makes a one-time code. Paste it into a client to pair it. |

If two machines change the same file while offline, neither edit is thrown away. The second one is saved next to the original as `name (conflict, laptop, 2026-11-04).ext`.

<img src="assets/divider.png" alt="" width="100%">

## Reaching your host

**Same network.** Works straight away.

**Port forwarding.** Forward one TCP port on your router to the host. The host uses its own certificate and the invite carries its fingerprint, so a client will refuse to talk to anything that isn't your host.

**Your own domain.** Point a domain at your home IP, forward port 443, and Larder gets a free Let's Encrypt certificate for it.

All traffic between a client and the host goes over TLS either way.

<img src="assets/divider.png" alt="" width="100%">

## Building from source

You need Rust (stable) and Node 20 or newer.

```sh
git clone https://github.com/bluecheze/larder
cd larder
cargo test --workspace
```

The desktop app lands in `app/` later in the roadmap.

<img src="assets/divider.png" alt="" width="100%">

<sub>MIT licensed. Banner painted locally with FLUX.2 klein 4B, turned into pixel art by <code>art/banner.py</code>.</sub>
