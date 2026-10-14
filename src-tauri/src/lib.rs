mod app_window;
mod frame;
mod hosts;
mod local_ahpd;
mod statuses;
mod tokens;
mod ui_source;

use tauri::{Manager, RunEvent, WebviewUrl, WebviewWindowBuilder, WindowEvent};

use crate::app_window::AppWindows;
use crate::hosts::HostStore;
use crate::local_ahpd::LocalAhpd;
use crate::statuses::HostStatuses;
use crate::tokens::Keyring;

/// The label of the window with the host list.
pub const SHELL_WINDOW: &str = "shell";

pub fn run() {
    tauri::Builder::default()
        // The host list's footer opens ahpio.com in the browser; its capability allows that one site.
        .plugin(tauri_plugin_opener::init())
        // Change, in Add host "This computer", picks the ahpd to use.
        .plugin(tauri_plugin_dialog::init())
        .register_asynchronous_uri_scheme_protocol(ui_source::UI, |ctx, request, responder| {
            let source = ctx.app_handle().state::<ui_source::UiSource>().inner().clone();
            ui_source::handle(source, ctx, request, responder);
        })
        .setup(|app| {
            // AHP_DESK_CONFIG_DIR runs the desk on another host list, as a test
            // does, so it never reads or writes the user's own hosts.
            let config = match std::env::var_os("AHP_DESK_CONFIG_DIR") {
                Some(dir) => std::path::PathBuf::from(dir),
                None => app.path().app_config_dir()?,
            };
            app.manage(HostStore::load(config.join("hosts.json"), Box::new(Keyring))?);
            app.manage(AppWindows::default());
            app.manage(HostStatuses::default());
            app.manage(LocalAhpd::new(app.handle())?);
            app.manage(ui_source::resolve(app.handle())?);
            // Frameless and rounded, as an app window (decision: the app window
            // draws its own rounded frame); the host list draws its title bar.
            let shell = WebviewWindowBuilder::new(app, SHELL_WINDOW, WebviewUrl::App("/".into()))
                .title("ahp-desk")
                .decorations(false)
                .transparent(true)
                .shadow(false)
                .inner_size(640.0, 400.0)
                .min_inner_size(480.0, 300.0)
                .build()?;
            frame::round_corners(&shell);
            let handle = app.handle().clone();
            shell.on_window_event(move |event| {
                // The host list holds every host's status, so while an app window
                // shows a host it hides instead of closing (decision: open hides the host list).
                if let WindowEvent::CloseRequested { api, .. } = event {
                    if handle.state::<AppWindows>().any_open() {
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            hosts::hosts,
            hosts::save_host,
            hosts::remove_host,
            hosts::host_token,
            local_ahpd::find_local_ahpd,
            local_ahpd::install_local_ahpd,
            local_ahpd::local_ahpd_status,
            local_ahpd::start_local_ahpd,
            local_ahpd::stop_local_ahpd,
            app_window::open_app_window,
            app_window::window_host,
            app_window::show_host_list,
            statuses::publish_host_statuses,
            statuses::host_statuses,
        ])
        .build(tauri::generate_context!())
        .expect("ahp-desk failed to start")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                local_ahpd::stop_on_quit(app);
            }
        });
}
