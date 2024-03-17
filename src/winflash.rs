#[cfg(target_os = "windows")]
use winapi::um::winuser::{FlashWindowEx, FLASHW_ALL, FLASHW_TIMERNOFG, FLASHWINFO};

#[cfg(target_os = "windows")]
pub fn flash_window(window_handle: winapi::shared::windef::HWND) {
    let mut flash_info = FLASHWINFO {
        cbSize: std::mem::size_of::<FLASHWINFO>() as u32,
        hwnd: window_handle,
        dwFlags: FLASHW_ALL | FLASHW_TIMERNOFG,
        uCount: std::u32::MAX,
        dwTimeout: 0,
    };

    unsafe {
        FlashWindowEx(&mut flash_info);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn flash_window(_window_handle: usize) {
    // No-op or alternative implementation for non-Windows platforms
}