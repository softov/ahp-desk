// Keeps a console window from opening next to the app in a Windows release build.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ahp_desk_lib::run();
}
