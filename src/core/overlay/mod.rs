// Adapted from https://www.unknowncheats.me/forum/anti-cheat-bypass/263403-window-hijacking-dont-overlay-betray.html

use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::EnumWindows;
use crate::core::overlay::overlay_finder_params::OverlayFinderParams;
use crate::core::overlay::window_info::WindowInfo;

pub(crate) mod window_info;
pub mod overlay_finder;
pub mod overlay_finder_params;

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // Convert the LPARAM back to our params structure
    let params = &mut *(lparam.0 as *mut OverlayFinderParams);

    let window_info = WindowInfo::from_hwnd(hwnd);

    // Check if window matches our criteria
    let (satisfied, unsatisfied) = params.matches_criteria(&window_info);

    // If no criteria are satisfied, skip this window
    if satisfied == 0 {
        return BOOL(1);
    }

    // If we need all criteria to match and some don't, skip this window
    if params.satisfy_all_criteria && unsatisfied > 0 {
        return BOOL(1);
    }

    // Window matches criteria - add it to our list
    params.hwnds.push(hwnd);
    BOOL(1) // Continue enumeration
}

pub fn find_overlays(mut params: OverlayFinderParams) -> Vec<HWND> {
    unsafe {
        EnumWindows(Some(enum_windows_callback), LPARAM(&mut params as *mut _ as isize))
            .expect("Failed to enumerate windows for window finder!");
    }
    params.hwnds
}

pub fn debug_print_overlays(handles: Vec<HWND>) {
    for (i, handle) in handles.iter().enumerate() {
        // Get window details
        let window_info = unsafe { WindowInfo::from_hwnd(*handle) };

        log::debug!("Window #{} found:", i + 1);
        log::debug!("\n{}", window_info);
    }
}

