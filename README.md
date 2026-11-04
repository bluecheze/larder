# Larder

Your own drive, on your own computer.

Pick one machine to be the host. It keeps the files. Every other machine you own becomes a client and keeps its folders in sync with the host. No cloud account, no monthly bill, no file size limits other than your disk.

Works on macOS, Linux and Windows.

> Larder is being built in the open. It is not ready for real data yet. See [ROADMAP.md](ROADMAP.md) for where things are.

## How it works

- The host stores a **library**, split into **shelves** (Documents, Photos, whatever you want).
- A client pairs with the host using an **invite** code and picks which shelves to keep on that machine.
- Changes go both ways. If two machines edit the same file before syncing, nothing is lost: the second one is kept as a conflict copy next to the original.
- Everything between client and host goes over TLS.

## Reaching your host

- **Same network**: works out of the box.
- **Port forwarding**: forward one TCP port on your router to the host. Larder uses a self-signed certificate and the invite carries its fingerprint, so clients know they are talking to your host and not something in between.
- **Your own domain**: point a domain at your home IP, forward port 443, and Larder gets a Let's Encrypt certificate for it.

## License

MIT
