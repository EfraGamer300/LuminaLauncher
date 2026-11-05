use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, GetForegroundWindow, GetWindowThreadProcessId, HTCAPTION, IsWindowVisible,
    SW_MINIMIZE, SW_RESTORE, SendMessageW, SetForegroundWindow, ShowWindow, WM_NCLBUTTONDOWN,
};
use windows::core::PCWSTR;

use crate::win::wide::to_wide;

pub fn find_window(class_name: &str, pid_filter: Option<u32>) -> Option<(HWND, u32)> {
    let class = to_wide(class_name);

    unsafe {
        let mut previous: Option<HWND> = None;

        loop {
            let Ok(handle) = FindWindowExW(None, previous, PCWSTR(class.as_ptr()), None) else {
                break;
            };
            if handle.0.is_null() {
                break;
            }

            let mut pid = 0u32;
            GetWindowThreadProcessId(handle, Some(&mut pid));
            previous = Some(handle);

            if pid_filter.is_some_and(|filter| filter != pid) {
                continue;
            }

            return Some((handle, pid));
        }

        None
    }
}

pub fn is_visible(handle: HWND) -> bool {
    unsafe { IsWindowVisible(handle).as_bool() }
}

pub fn activate(handle: HWND) {
    unsafe {
        let _ = ShowWindow(handle, SW_RESTORE);
        let _ = SetForegroundWindow(handle);
    }
}

pub fn minimize_foreground() {
    unsafe {
        let handle = GetForegroundWindow();
        if !handle.0.is_null() {
            let _ = ShowWindow(handle, SW_MINIMIZE);
        }
    }
}

pub fn start_drag() {
    unsafe {
        let handle = GetForegroundWindow();
        if handle.0.is_null() {
            return;
        }
        let _ = ReleaseCapture();
        SendMessageW(
            handle,
            WM_NCLBUTTONDOWN,
            Some(WPARAM(HTCAPTION as usize)),
            Some(LPARAM(0)),
        );
    }
}
