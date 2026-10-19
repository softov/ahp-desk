# Developer guide

ahp-desk is a Tauri 2 desktop app. The shell is Rust; the windows load the ahpw or ahpapp bundle.

## What you need

| Tool | Version | Why |
| --- | --- | --- |
| Node.js | 22 or later | The UI bundles and the Tauri CLI. |
| pnpm | 11 | The package manager, as in ahpw. |
| Rust | stable, through rustup | The Tauri shell. |
| Tauri system packages | see below, per OS | The webview and the tray. |

## Linux (Debian 13)

Install the system packages:

```sh
sudo apt install build-essential curl wget file pkg-config libssl-dev libxdo-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev
```

Install Rust:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
```

Check the toolchain:

```sh
rustc --version
cargo --version
node --version
pnpm --version
```

"Start with the OS" writes `~/.config/autostart/ahp-desk.desktop`.
A deb removal does not touch a user's home, so that file stays until the box is unticked or the file is deleted.

## Windows

1. Install the [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and select "Desktop development with C++".
2. Install [Rust with rustup](https://rustup.rs/). Keep the default MSVC toolchain (`stable-x86_64-pc-windows-msvc`).
3. Install [Node.js 22](https://nodejs.org/), then run `corepack enable` to get pnpm.
4. WebView2 ships with Windows 10 and 11. On an older system, install the [Evergreen runtime](https://developer.microsoft.com/microsoft-edge/webview2/).

Check the toolchain in a new PowerShell:

```powershell
rustc --version
cargo --version
node --version
pnpm --version
```

## Get the code

```sh
git clone git@github.com:softov/ahp-desk.git
cd ahp-desk
pnpm install
```

ahpw and its embedding contract, `@ahpd/web/embed`, come from the pinned `@ahpd/web` on npm; both pins, in `package.json` and `packages/ahpw-embedder/package.json`, move together.
An ahpw checkout beside this one is needed only to work on ahpw inside the desk (below).

## Run and build

| Command | What it does |
| --- | --- |
| `pnpm tauri dev` | Runs the app with live reload. |
| `pnpm tauri build` | Builds the installer for the OS you are on. |
| `pnpm tauri build --debug` | A faster build with dev tools on, for testing an installer. |

The output is under `src-tauri/target/release/bundle/`:

- Windows: `nsis/ahp-desk_<version>_x64-setup.exe`.
- Linux: `appimage/` and `deb/`.

A Windows installer is built on Windows. Tauri does not cross-compile it from Linux in a supported way.

## Testing ahpw from source

A build ships one UI, chosen by `AHP_DESK_UI`: `ahpw`, the default, or `ahpapp`, which comes in plan 05.
ahpw comes from the published `@ahpd/web` by default, or from an ahpw checkout:

| Variable | Values | What it does |
| --- | --- | --- |
| `AHP_DESK_AHPW` | `npm` (default), `local` | `npm` serves `node_modules/@ahpd/web/dist/app`. `local` loads ahpw's dev server at `http://127.0.0.1:5180` in `pnpm tauri dev`, and stages the checkout's `dist/app` for `pnpm tauri build`. |
| `AHP_DESK_AHPW_DIR` | a path | The ahpw checkout for `local`. Default `../ahpw`. |
| `AHP_DESK_CONFIG_DIR` | a path | Read when the desk runs, not when it builds: the folder of `hosts.json`, in place of the app config folder. A test runs on its own host list with it. |

To work on ahpw inside the desk, run its dev server in the ahpw checkout, then the desk:

```sh
pnpm dev                               # in ../ahpw
AHP_DESK_AHPW=local pnpm tauri dev     # here
```

`pnpm stage:ui` copies the chosen `dist/app` into `src-tauri/resources/ui/`, where a build bundles it from.

`pnpm stage:node` puts the Node 22 runtime the desk ships into `src-tauri/resources/node/`: the official build for the OS and CPU you build on, checked against the release's `SHASUMS256.txt`.
The desk installs ahpd with that Node's npm, and runs it on that Node.
The first run downloads about 30 MB into `.cache/node/`; later runs are offline.
The version is `VERSION` in `scripts/stage-node.mjs`.

Each app window gets ahpw's embedder: `pnpm build:client` builds `packages/ahpw-embedder` into one script, and Rust injects it with `initialization_script` so `globalThis.ahpwEmbedder` is set before ahpw runs.

`pnpm stage:all` runs all three; `pnpm tauri dev`, `pnpm tauri build`, `pnpm test` and `pnpm lint` run it first.
A bare `cargo build` needs the three in place, and says which is missing.

## Testing on Windows

1. Build on Windows with `pnpm tauri build`, or take the installer from a CI run.
2. Run the `-setup.exe`. It installs for the current user, so it needs no admin rights.
3. To test against a host on this Linux box, run ahpd here and add it as a URL host from Windows.
4. To remove it, use "Apps" in Windows Settings.

"Start with the OS" is the value `ahp-desk` under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`; check it with `reg query HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
It holds the path of the exe that ticked the box, so a debug build can start itself at login.
Untick the box in Settings before you remove that build.
The uninstaller removes the value, and an update keeps it.

On Windows, the dev tools in a webview open with `Ctrl+Shift+I` in a debug build.

## Signing keys

The updater checks each release with a key pair. The private key never goes in git; `.gitignore` covers `*.key` and `.tauri/`.

```sh
pnpm tauri signer generate -w .tauri/updater.key
```

The public key goes in `src-tauri/tauri.conf.json` under `plugins.updater.pubkey`. To sign a build, set these in the shell:

```sh
export TAURI_SIGNING_PRIVATE_KEY_PATH=.tauri/updater.key
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=...
```

A build makes the signed updater bundles only with `--config src-tauri/tauri.release.conf.json`, which turns on `createUpdaterArtifacts`; without it, a clone builds installers with no key.

CI signs with two repository secrets, in GitHub, Settings, Secrets and variables, Actions: `TAURI_SIGNING_PRIVATE_KEY`, the content of `.tauri/updater.key`, and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, its password.

Keep a copy of the key and its password off the machine.
A lost key leaves every installed desk unable to update: a release signed with a new key is refused by them, and each one has to be installed again by hand.

Windows code signing certificates (`*.pfx`, `*.p12`) are also kept out of git.

## Releasing

The desk never fetches its UI at run time: ahpw is built into each desk release.
A fix in ahpw reaches people only through these steps, so make a desk release for every ahpw release the desk needs.

1. When ahpw is released as a new `@ahpd/web`, move both pins to it: `package.json` and `packages/ahpw-embedder/package.json`.
   Then run `pnpm install`.
2. Run `pnpm test` and `pnpm lint`.
   Then run `pnpm tauri dev` and open a host to see the new ahpw in an app window.
3. Raise the desk's version in `package.json`, `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`, the same in all three.
   The updater compares the version in `tauri.conf.json`.
4. Commit, tag `v<version>`, and push the tag.
   `.github/workflows/release.yml` checks that the tag matches the three versions, builds on Windows and on Ubuntu 22.04, and publishes one release with the NSIS installer, the AppImage, the deb, their `.sig` files and `latest.json`.
   The release stays a draft until both builds pass.
5. An installed desk finds the release at its next start, downloads it, and asks before it restarts.

The Windows installer has no code signature, so SmartScreen warns before it runs.

On Linux, `latest.json` has an entry for each bundle: a desk installed from the AppImage updates from the next AppImage, and one installed from the deb from the next deb (`linux-x86_64-deb`).
The workflow's last job warns if the deb entry is ever missing, since a deb install then cannot update itself.

ahpw's own release steps live in the ahpw repository.

## Project records

What is built (specs), the plans, the decisions and the research live in `.project/`. The format is the `do-spec` skill in `.agents/skills/do-spec/`. Start at [`.project/plans/index.md`](.project/plans/index.md).
