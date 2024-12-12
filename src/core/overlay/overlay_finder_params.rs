use windows::Win32::Foundation::{HWND, POINT, RECT};
use crate::core::overlay::window_info::WindowInfo;

// Structure to hold search criteria for finding overlay windows
#[derive(Clone, Default)]
pub struct OverlayFinderParams {
    pub pid_owner: Option<u32>,     // Optional specific process ID to search for
    pub wnd_class_name: String,     // Target window class name
    pub wnd_name: String,           // Target window title
    pub pos: RECT,                  // Target window position
    pub res: POINT,                 // Target window size
    pub percent_all_screens: f32,   // Minimum percentage of total screen space
    pub percent_main_screen: f32,   // Minimum percentage of main screen
    pub style: u32,                 // Required window styles
    pub style_ex: u32,              // Required extended window styles
    pub satisfy_all_criteria: bool, // Must match all criteria if true
    pub hwnds: Vec<HWND>,           // Collection of matching window handles
}

impl OverlayFinderParams {
    /// Verifies if the current window matches our search criteria
    pub(crate) fn matches_criteria(&self, info: &WindowInfo) -> (u8, u8) {
        let mut satisfied = 0u8;     // Count of matched criteria
        let mut unsatisfied = 0u8;   // Count of unmatched criteria

        // If we're looking for a specific PID, check it
        if let Some(target_pid) = self.pid_owner {
            if target_pid == info.pid {
                satisfied += 1;
            }
            else {
                unsatisfied += 1;
            }
        }

        // Check if class name matches (if we're looking for one)
        if !self.wnd_class_name.is_empty() {
            if self.wnd_class_name == info.class_name {
                satisfied += 1;
            }
            else {
                unsatisfied += 1;
            }
        }

        // Check if window title matches (if we're looking for one)
        if !self.wnd_name.is_empty() {
            if self.wnd_name == info.title {
                satisfied += 1;
            } else {
                unsatisfied += 1;
            }
        }

        // Check if position matches (if we specified one)
        if self.pos.left != 0 || self.pos.top != 0 || self.pos.right != 0 || self.pos.bottom != 0 {
            if self.pos == info.position {
                satisfied += 1;
            }
            else {
                unsatisfied += 1;
            }
        }

        // Check if size matches (if we specified one)
        if self.res.x != 0 || self.res.y != 0 {
            if self.res == info.size {
                satisfied += 1;
            }
            else {
                unsatisfied += 1;
            }
        }

        // Check screen space percentages
        let (percent_all, percent_main) = info.get_screen_percentages();

        // Check if window takes up enough of all screens
        if self.percent_all_screens != 0.0 {
            if percent_all >= self.percent_all_screens {
                satisfied += 1;
            }
            else {
                unsatisfied += 1;
            }
        }

        // Check if window takes up enough of main screen
        if self.percent_main_screen != 0.0 {
            if percent_main >= self.percent_main_screen {
                satisfied += 1;
            }
            else {
                unsatisfied += 1;
            }
        }

        // Check if window has required style bits
        if self.style != 0 {
            if self.style & info.style as u32 != 0 {
                satisfied += 1;
            }
            else {
                unsatisfied += 1;
            }
        }

        // Check if window has required extended style bits
        if self.style_ex != 0 {
            if self.style_ex & info.style_ex as u32 != 0 {
                satisfied += 1;
            }
            else {
                unsatisfied += 1;
            }
        }

        (satisfied, unsatisfied)
    }
}