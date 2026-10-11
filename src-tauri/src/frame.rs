//! The frame of a window with no OS frame, an app window or the host list: on
//! Windows, DWM draws its corners round, as on pood's window (opendoop
//! `packages/tauri/src-tauri/src/lib.rs`). The page rounds its own root to
//! match and squares it when maximized.

#[cfg(target_os = "windows")]
pub fn round_corners<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>) {
    use std::ffi::c_void;

    // https://learn.microsoft.com/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute
    const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
    const DWMWCP_ROUND: u32 = 2;

    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(
            hwnd: *mut c_void,
            attribute: u32,
            pv_attribute: *const c_void,
            cb_attribute: u32,
        ) -> i32;
    }

    let Ok(hwnd) = window.hwnd() else { return };
    let preference = DWMWCP_ROUND;
    // Windows 10 has no such attribute and answers an error; its corners stay square.
    unsafe {
        DwmSetWindowAttribute(
            hwnd.0,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &preference as *const u32 as *const c_void,
            std::mem::size_of::<u32>() as u32,
        );
    }
}

#[cfg(not(target_os = "windows"))]
pub fn round_corners<R: tauri::Runtime>(_window: &tauri::WebviewWindow<R>) {}
