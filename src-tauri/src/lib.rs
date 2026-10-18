mod app_window;
mod frame;
mod hosts;
mod local_ahpd;
mod settings;
mod ssh;
mod ssh_install;
mod statuses;
mod tokens;
mod tray;
mod ui_source;
mod updates;

use tauri::{Manager, RunEvent, WebviewUrl, WebviewWindowBuilder, WindowEvent};

use crate::app_window::AppWindows;
use crate::hosts::HostStore;
use crate::local_ahpd::LocalAhpd;
use crate::settings::{CloseBehavior, SettingsStore};
use crate::ssh::known_keys::KnownKeys;
use crate::statuses::HostStatuses;
use crate::tokens::Keyring;

/// The label of the window with the host list.
pub const SHELL_WINDOW: &str = "shell";

/// The flag the login entry starts the desk with.
const AT_LOGIN: &str = "--autostart";

pub fn run() {
    tauri::Builder::default()
        // First, as its docs ask: a second start shows this desk's host list
        // rather than making a second desk on the same hosts.json and ahpd.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Err(err) = app_window::show_host_list(app.clone(), None, None) {
                eprintln!("ahp-desk: a second start could not show the host list: {err}");
            }
        }))
        // The OS starts the desk at login with --autostart.
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![AT_LOGIN]),
        ))
        // The host list's footer opens ahpio.com in the browser; its capability allows that one site.
        .plugin(tauri_plugin_opener::init())
        // Change, in Add host "This computer", picks the ahpd to use.
        .plugin(tauri_plugin_dialog::init())
        .register_asynchronous_uri_scheme_protocol(ui_source::UI, |ctx, request, responder| {
            let source = ctx
                .app_handle()
                .state::<ui_source::UiSource>()
                .inner()
                .clone();
            ui_source::handle(source, ctx, request, responder);
        })
        .setup(|app| {
            // AHP_DESK_CONFIG_DIR runs the desk on another host list, as a test
            // does, so it never reads or writes the user's own hosts.
            let config = match std::env::var_os("AHP_DESK_CONFIG_DIR") {
                Some(dir) => std::path::PathBuf::from(dir),
                None => app.path().app_config_dir()?,
            };
            app.manage(HostStore::load(
                config.join("hosts.json"),
                Box::new(Keyring),
            )?);
            app.manage(SettingsStore::load(config.join("settings.json"))?);
            app.manage(updates::Updates::load(config.join("updates.json")));
            app.manage(KnownKeys::load(
                config.join("known_keys.json"),
                app.path().home_dir()?.join(".ssh").join("known_hosts"),
            )?);
            app.manage(AppWindows::default());
            app.manage(HostStatuses::default());
            app.manage(ssh_install::InstallControl::default());
            ssh::tunnel::manage(app.handle());
            app.manage(LocalAhpd::new(app.handle())?);
            app.manage(ui_source::resolve(app.handle())?);
            // Frameless and rounded, as an app window; the host list draws its title bar.
            let shell = WebviewWindowBuilder::new(app, SHELL_WINDOW, WebviewUrl::App("/".into()))
                .title("ahp-desk")
                .decorations(false)
                .transparent(true)
                .shadow(false)
                .inner_size(640.0, 400.0)
                .min_inner_size(480.0, 300.0)
                // A start at login shows only the tray icon; a start by hand shows the list.
                .visible(!std::env::args().any(|arg| arg == AT_LOGIN))
                .build()?;
            frame::round_corners(&shell);
            tray::install(app.handle())?;
            let handle = app.handle().clone();
            shell.on_window_event(move |event| {
                // The host list holds every host's status, so while an app window
                // shows a host it hides instead of closing.
                // With none, it hides to the tray unless Settings says quit.
                if let WindowEvent::CloseRequested { api, .. } = event {
                    let hide = handle.state::<AppWindows>().any_open()
                        || handle.state::<SettingsStore>().get().close_behavior
                            == CloseBehavior::MinimizeToTray;
                    if hide {
                        api.prevent_close();
                        if let Some(shell) = handle.get_webview_window(SHELL_WINDOW) {
                            // A host list that will not hide stays on screen; nothing is lost.
                            let _ = shell.hide();
                        }
                    }
                }
            });
            // After the host list window, so its rows show each host as it comes up.
            local_ahpd::start_on_launch(app.handle());
            updates::watch(app.handle());
            ssh::tunnel::open_on_launch(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            hosts::hosts,
            hosts::save_host,
            hosts::remove_host,
            hosts::host_token,
            local_ahpd::find_local_ahpd,
            local_ahpd::install_local_ahpd,
            local_ahpd::add_local_backends,
            local_ahpd::serve_local_folder,
            local_ahpd::local_home_folder,
            local_ahpd::local_ahpd_status,
            local_ahpd::start_local_ahpd,
            local_ahpd::stop_local_ahpd,
            app_window::open_app_window,
            app_window::window_host,
            app_window::show_host_list,
            statuses::publish_host_statuses,
            statuses::host_statuses,
            tray::set_tray_words,
            settings::settings,
            settings::save_settings,
            settings::start_with_os,
            settings::set_start_with_os,
            ssh::password::ssh_keyring_works,
            ssh::known_keys::confirm_ssh_key,
            ssh::known_keys::forget_ssh_key,
            ssh_install::probe_ssh_server,
            ssh_install::install_ssh_ahpd,
            ssh_install::cancel_ssh_install,
            ssh_install::uninstall_ssh_ahpd,
            updates::updates,
            ssh::tunnel::ssh_tunnel_states,
            ssh::tunnel::open_ssh_tunnel,
            ssh::tunnel::ssh_sign_in,
            ssh::tunnel::pick_ssh_local_port,
        ])
        .build(tauri::generate_context!())
        .expect("ahp-desk failed to start")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                app.state::<ssh::tunnel::Tunnels>().close_all();
                local_ahpd::stop_on_quit(app);
            }
        });
}
