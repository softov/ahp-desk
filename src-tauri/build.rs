//! Chooses the build's one UI and where a debug build reads ahpw from.
//!
//! - `AHP_DESK_UI`: `ahpw` (default) or `ahpapp`. A build ships one UI.
//! - `AHP_DESK_AHPW`: `npm` (default), the published `@ahpd/web`, or `local`,
//!   ahpw's dev server, for a debug build.
//!
//! A release build serves the bundle `scripts/stage-ui.mjs` copies into
//! `resources/ui/`, from the same settings and `AHP_DESK_AHPW_DIR`.
//!
//! Each app window also gets ahpw's embedder, built by `pnpm build:client`.

use std::env;
use std::path::PathBuf;

fn setting(name: &str, default: &str, allowed: &[&str]) -> String {
    println!("cargo:rerun-if-env-changed={name}");
    let value = env::var(name).unwrap_or_else(|_| default.to_string());
    if !allowed.contains(&value.as_str()) {
        panic!("{name} is {value:?}; use one of {allowed:?}");
    }
    value
}

fn main() {
    let ui = setting("AHP_DESK_UI", "ahpw", &["ahpw", "ahpapp"]);
    if ui == "ahpapp" {
        panic!("AHP_DESK_UI=ahpapp: ahpapp support comes in plan p5");
    }
    let source = setting("AHP_DESK_AHPW", "npm", &["npm", "local"]);

    let workspace = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .expect("src-tauri sits in the workspace")
        .to_path_buf();
    let npm_dir = workspace.join("node_modules/@ahpd/web/dist/app");

    println!("cargo:rustc-env=AHP_DESK_UI={ui}");
    println!("cargo:rustc-env=AHP_DESK_AHPW={source}");
    println!("cargo:rustc-env=AHP_DESK_AHPW_NPM_DIR={}", npm_dir.display());

    // The embedder script each app window gets; `pnpm build:client` makes it.
    let embedder = workspace.join("packages/ahpw-embedder/dist/ahpw-embedder.js");
    println!("cargo:rerun-if-changed={}", embedder.display());
    if !embedder.is_file() {
        panic!("{} is missing; run `pnpm build:client` first", embedder.display());
    }
    println!("cargo:rustc-env=AHP_DESK_EMBEDDER={}", embedder.display());

    // Declaring the commands makes each one a permission, so a capability
    // grants a window only the commands it lists.
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "hosts",
            "save_host",
            "remove_host",
            "host_token",
            "open_app_window",
            "window_host",
            "show_host_list",
            "publish_host_statuses",
            "host_statuses",
            "find_local_ahpd",
            "install_local_ahpd",
            "local_ahpd_status",
            "start_local_ahpd",
            "stop_local_ahpd",
        ]),
    ))
    .expect("the Tauri build failed");
}
