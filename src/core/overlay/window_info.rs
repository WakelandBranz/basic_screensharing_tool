use std::fmt;
use windows::Win32::{
    Foundation::{
        HWND,
        POINT,
        RECT,
    },
    UI::WindowsAndMessaging::{
        GetClassNameW,
        GetDesktopWindow,
        GetSystemMetrics,
        GetWindowLongPtrW,
        GetWindowRect,
        GetWindowTextW,
        GetWindowThreadProcessId,
        GWL_EXSTYLE,
        GWL_STYLE,
        SM_CXSCREEN,
        SM_CYSCREEN,
    }
};

const MAX_CLASS_NAME: usize = 255;
const MAX_WND_NAME: usize = MAX_CLASS_NAME;

// Structure to hold all information about a window
#[derive(Clone, Default)]
pub(crate) struct WindowInfo {
    pub hwnd: HWND,         // Window handle
    pub pid: u32,           // Process ID that owns the window
    pub tid: u32,           // Thread ID that created the window
    pub class_name: String, // Window class name
    pub title: String,      // Window title text
    pub position: RECT,     // Window position (left, top, right, bottom)
    pub size: POINT,        // Window size (width, height)
    pub style: isize,       // Window style flags
    pub style_ex: isize,    // Extended window style flags
}

impl WindowInfo {
    /// Creates a WindowInfo struct from a window handle
    pub unsafe fn from_hwnd(hwnd: HWND) -> Self {
        let mut info = WindowInfo::default();

        // Store handle
        info.hwnd = hwnd;

        // Get process ID and thread ID
        GetWindowThreadProcessId(hwnd, Some(&mut info.pid));
        info.tid = GetWindowThreadProcessId(hwnd, None);

        // Get window class name
        let mut class_name = [0u16; MAX_CLASS_NAME];
        let len = GetClassNameW(hwnd, &mut class_name);
        info.class_name = String::from_utf16_lossy(&class_name[..len as usize]);

        // Get Window title
        let mut title = [0u16; MAX_WND_NAME]; // Create a buffer array filled with zeros
        let len = GetWindowTextW(hwnd, &mut title);
        info.title = String::from_utf16_lossy(&title[..len as usize]);

        // Get window position and calculate size
        GetWindowRect(hwnd, &mut info.position)
            .expect("Failed to get window rect!");
        info.size = POINT {
            x: info.position.right - info.position.left,  // Width
            y: info.position.bottom - info.position.top,  // Height
        };

        // Get window styles
        info.style = GetWindowLongPtrW(hwnd, GWL_STYLE);      // Basic Styles
        info.style_ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE); // Extended styles

        info
    }

    // Calculates what percentage of screen space the window occupies
    pub fn get_screen_percentages(&self) -> (f32, f32) {
        // Get total screen dimensions
        let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };

        // Calculate percentage of all screens
        let ratio_all_screens_x = self.size.x as f32 / screen_width as f32;
        let ratio_all_screens_y = self.size.y as f32 / screen_height as f32;
        let percent_all_screens = ratio_all_screens_x * ratio_all_screens_y * 100.0;

        // Get main desktop window dimensions
        let desktop_hwnd = unsafe { GetDesktopWindow() };
        let mut desktop_rect = RECT::default();
        unsafe { GetWindowRect(desktop_hwnd, &mut desktop_rect) }
            .expect("Failed to get window rect!");

        let desktop_width = desktop_rect.right - desktop_rect.left;
        let desktop_height = desktop_rect.bottom - desktop_rect.top;

        // Calculate percentage of main screen
        let ratio_main_screen_x = self.size.x as f32 / desktop_width as f32;
        let ratio_main_screen_y = self.size.y as f32 / desktop_height as f32;
        let percent_main_screen = ratio_main_screen_x * ratio_main_screen_y * 100.0;

        (percent_all_screens, percent_main_screen)
    }

}

impl fmt::Display for WindowInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Window Details:")?;
        writeln!(f, "Handle: {:?}", self.hwnd)?;
        writeln!(f, "  Title: {}", self.title)?;
        writeln!(f, "  Class Name: {}", self.class_name)?;
        writeln!(f, "  Process ID: {}", self.pid)?;
        writeln!(f, "  Thread ID: {}", self.tid)?;
        writeln!(f, "  Position: Left={}, Top={}, Right={}, Bottom={}",
                 self.position.left, self.position.top,
                 self.position.right, self.position.bottom)?;
        writeln!(f, "  Size: {}x{}", self.size.x, self.size.y)?;
        writeln!(f, "  Style: {:#x}", self.style)?;
        writeln!(f, "  Extended Style: {:#x}", self.style_ex)
    }
}