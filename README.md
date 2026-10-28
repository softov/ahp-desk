# ahp-desk

[![release](https://img.shields.io/github/v/release/softov/ahp-desk)](https://github.com/softov/ahp-desk/releases/latest)
![license MIT](https://img.shields.io/badge/license-MIT-blue)
![Agent Host Protocol 1.0.0](https://img.shields.io/badge/AHP-1.0.0-0b7285)

A desktop app for [Agent Host Protocol](https://microsoft.github.io/agent-host-protocol/) hosts, on Windows and Linux.

Keep a list of your hosts, see which ones are up, and open each one in a window of its own with [ahpw](https://github.com/softov/ahpw).

![The host list with two hosts connected on ahpd 0.14.0 and one down](docs/media/host-list-dark.png)

What you get:

- **Hosts**, one list with a live dot for each: connected, connecting or down, and the server's name and version.
- **Three ways to add a host:**
  - **Remote host**, an AHP server at an address you know, with its token.
  - **This computer**, an ahpd that runs here; the desk installs one for you with its own Node, so nothing else is needed.
  - **Connect via SSH**, a Linux server you can sign in to; the desk uses the ahpd it finds there, or installs one, and reaches it through an SSH tunnel or an open port.
- **A window per host**, with ahpw inside: sessions, chats, terminals and the rest of the host.
- **Updates**, for the desk itself and for ahpd: the host list says when a newer one is ready, and nothing restarts until you click.
- **The tray**, where the desk stays when you close the list, and **Launch at startup**, so your hosts are there after you sign in.
- **English, Português and Español.**

---

## Install

Download the latest release from the [releases page](https://github.com/softov/ahp-desk/releases/latest).

| System | File | How |
| --- | --- | --- |
| Windows 10 and 11 | `ahp-desk_<version>_x64-setup.exe` | Run it. It installs for your user only, so it asks for no admin rights. |
| Linux | `ahp-desk_<version>_amd64.AppImage` | `chmod +x` it and run it. |
| Debian and Ubuntu | `ahp-desk_<version>_amd64.deb` | `sudo apt install ./ahp-desk_<version>_amd64.deb` |

> **Note:** The Windows installer has no code signature yet, so SmartScreen warns before it runs. Click **More info**, then **Run anyway**.

On Linux the tray needs `libayatana-appindicator3`, which the AppImage carries and the deb installs.
GNOME shows the tray only with its AppIndicator extension (`gnome-shell-extension-appindicator` on Debian and Ubuntu).
With no tray, closing the host list quits the desk.

The first start welcomes you and asks where your first host is:

![The welcome screen, with Launch at startup and Get started](docs/media/welcome-dark.png)

![Where is your host? Remote host, This computer, or Connect via SSH](docs/media/where-dark.png)

**This computer** uses the ahpd it finds here, or installs one with the desk's own Node:

![This computer, installing ahpd with the Claude backend](docs/media/add-local-dark.png)

**Connect via SSH** checks the server first, then uses the ahpd it finds there or installs one as a user service:

![Connect via SSH, installing ahpd on an Ubuntu server, with Sign in, Check the server and Node done](docs/media/add-ssh-dark.png)

**Remote host** takes the address and token of an AHP server you already run:

![Remote host, with a name, an address and a token](docs/media/add-remote-dark.png)

Once installed, the desk updates itself: it checks each release's signature before it installs it.

---

## Where it keeps things

- The host list and the settings: `%APPDATA%\dev.softov.ahpdesk\` on Windows, `~/.config/dev.softov.ahpdesk/` on Linux.
- Tokens and SSH passwords: the OS keyring (Windows Credential Manager, or the Secret Service on Linux), never in the host list.
- SSH server keys you trusted: `known_keys.json` beside the host list, and your own `~/.ssh/known_hosts` is read, never written.

---

## Build it yourself

See [DEVELOPER.md](DEVELOPER.md): the tools, building on Windows and Debian, testing ahpw from source, and how a release is made.

## Related

- [ahpd](https://github.com/softov/ahpd), an AHP server and daemon.
- [ahpw](https://github.com/softov/ahpw), the web UI the desk shows in each host's window.

## License

[MIT](LICENSE)
