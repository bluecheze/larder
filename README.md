<p align="center">
  <img src="assets/banner.png" alt="Pixel art pantry: the word larder chalked on a blackboard, and a boy with an orange backpack putting a box on a shelf full of jars and crates" width="100%">
</p>

<p align="center">
  Your own drive, on your own computer.<br>
  <sub>macOS · Linux · Windows</sub>
</p>

<img src="assets/divider-1.png" alt="" width="100%">

<h2><img src="assets/h-what-it-is.png" alt="What it is" height="34"></h2>

Larder turns one of your computers into a private cloud drive for the rest of them.

The **host** keeps every file. Your other machines are **clients**: they see everything on the host as a normal drive in Finder, Explorer or your Linux file manager, but a file only comes down when you open it. Nothing sits on a client's disk unless you asked for it.

No account, no subscription, no storage cap other than the disk in your host.

> [!NOTE]
> Larder is being built in the open and isn't ready for files you can't lose yet. [ROADMAP.md](ROADMAP.md) shows where it's at.

<img src="assets/divider-2.png" alt="" width="100%">

<h2><img src="assets/h-how-it-works.png" alt="How it works" height="34"></h2>

<table>
  <tr>
    <td width="64" align="center"><img src="assets/icon-library.png" width="48" alt=""></td>
    <td><b>Library</b><br>The folder on the host where everything lives.</td>
  </tr>
  <tr>
    <td align="center"><img src="assets/icon-shelf.png" width="48" alt=""></td>
    <td><b>Shelves</b><br>Folders inside the library: Documents, Photos, Projects, whatever you like.</td>
  </tr>
  <tr>
    <td align="center"><img src="assets/icon-drive.png" width="48" alt=""></td>
    <td><b>Larder drive</b><br>On a client, every shelf shows up as a mounted drive. Opening a file streams it from the host and keeps a copy in a small cache that empties itself when it fills up.</td>
  </tr>
  <tr>
    <td align="center"><img src="assets/icon-offline.png" width="48" alt=""></td>
    <td><b>Keep on this device</b><br>Mark a shelf as offline and the client keeps a full copy that syncs both ways, so it still works with no connection.</td>
  </tr>
  <tr>
    <td align="center"><img src="assets/icon-invite.png" width="48" alt=""></td>
    <td><b>Invites</b><br>The host makes a one-time code. Paste it into a client to pair it.</td>
  </tr>
</table>

If two machines change the same file while offline, neither edit is thrown away. The second one is saved next to the original as `name (conflict, laptop, 2026-11-04).ext`.

<img src="assets/divider-3.png" alt="" width="100%">

<h2><img src="assets/h-reaching-your-host.png" alt="Reaching your host" height="34"></h2>

**Same network.** Works straight away.

**Port forwarding.** Forward one TCP port on your router to the host. The host uses its own certificate and the invite carries its fingerprint, so a client refuses to talk to anything that isn't your host.

**Your own domain.** Point a domain at your home IP, forward port 443, and Larder gets a free Let's Encrypt certificate for it.

All traffic between a client and the host goes over TLS either way.

<img src="assets/divider-4.png" alt="" width="100%">

<h2><img src="assets/h-building-from-source.png" alt="Building from source" height="34"></h2>

You need Rust (stable) and Node 20 or newer.

```sh
git clone https://github.com/bluecheze/larder
cd larder
cargo test --workspace
```

The desktop app lands in `app/` later in the roadmap.

<br>

<p align="center"><sub>MIT licensed. Art painted locally with FLUX.2 klein 4B and turned into pixel art by the scripts in <code>art/</code>.</sub></p>
