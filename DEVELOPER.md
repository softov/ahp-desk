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

Clone [ahpw](https://github.com/softov/ahpw) beside it and run `pnpm install && pnpm build` there once.
`packages/ahpw-embedder` takes ahpw's embedding contract, `@ahpd/web/embed`, from that checkout until a version of `@ahpd/web` with it is on npm.

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

A build ships one UI, chosen by `AHP_DESK_UI`: `ahpw`, the default, or `ahpapp`, which comes in plan p5.
ahpw comes from the published `@ahpd/web` by default, or from an ahpw checkout:

| Variable | Values | What it does |
| --- | --- | --- |
| `AHP_DESK_AHPW` | `npm` (default), `local` | `npm` serves `node_modules/@ahpd/web/dist/app`. `local` loads ahpw's dev server at `http://127.0.0.1:5180` in `pnpm tauri dev`, and stages the checkout's `dist/app` for `pnpm tauri build`. |
| `AHP_DESK_AHPW_DIR` | a path | The ahpw checkout for `local`. Default `../ahpw`. |

To work on ahpw inside the desk, run its dev server in the ahpw checkout, then the desk:

```sh
pnpm dev                               # in ../ahpw
AHP_DESK_AHPW=local pnpm tauri dev     # here
```

`pnpm stage:ui` copies the chosen `dist/app` into `src-tauri/resources/ui/`, where a release build bundles it from; `pnpm tauri build` runs it first.

Each app window gets ahpw's embedder: `pnpm build:client` builds `packages/ahpw-embedder` into one script, and Rust injects it with `initialization_script` so `globalThis.ahpwEmbedder` is set before ahpw runs.
`pnpm tauri dev`, `pnpm tauri build`, `pnpm test` and `pnpm lint` run `pnpm build:client` first; a bare `cargo build` stops and asks for it.

## Testing on Windows

1. Build on Windows with `pnpm tauri build`, or take the installer from a CI run.
2. Run the `-setup.exe`. It installs for the current user, so it needs no admin rights.
3. To test against a host on this Linux box, run ahpd here and add it as a URL host from Windows.
4. To remove it, use "Apps" in Windows Settings.

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

Windows code signing certificates (`*.pfx`, `*.p12`) are also kept out of git.

## Project records

Plans, decisions and research live in `.project/`. The format is the `do-spec` skill in `.agents/skills/do-spec/`. Start at [`.project/plans/index.md`](.project/plans/index.md).
