use windows::Win32::{
    Foundation::{
        POINT,
        RECT,
    },
};
use crate::core::overlay::{
    find_overlays,
    overlay_finder_params::OverlayFinderParams,
    window_info::WindowInfo,
};

// TODO! Make this into a builder style

pub struct OverlayFinder {
    params: OverlayFinderParams,
    pub overlays: Vec<WindowInfo>,
}

impl Default for OverlayFinder {
    fn default() -> Self {
        Self {
            params: OverlayFinderParams::default(),
            overlays: Vec::new(),
        }
    }
}

impl OverlayFinder {
    pub fn new() -> Self {
        Self::default()
    }

    // BUILDER METHODS -----------------------------------------------------------------------------
    pub fn with_pid_owner(&mut self, pid: u32) -> &mut Self {
        self.params.pid_owner = Some(pid);
        self
    }

    pub fn with_window_class(&mut self, class_name: impl Into<String>) -> &mut Self {
        self.params.wnd_class_name = class_name.into();
        self
    }

    pub fn with_window_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.params.wnd_name = name.into();
        self
    }

    pub fn with_position(&mut self, rect: RECT) -> &mut Self {
        self.params.pos = rect;
        self
    }

    pub fn with_size(&mut self, size: POINT) -> &mut Self {
        self.params.res = size;
        self
    }

    pub fn with_style(&mut self, style: u32) -> &mut Self {
        self.params.style = style;
        self
    }

    pub fn with_style_ex(&mut self, style_ex: u32) -> &mut Self {
        self.params.style_ex = style_ex;
        self
    }

    pub fn with_percent_all_screens(&mut self, percent: f32) -> &mut Self {
        self.params.percent_all_screens = percent;
        self
    }

    pub fn with_percent_main_screen(&mut self, percent: f32) -> &mut Self {
        self.params.percent_main_screen = percent;
        self
    }

    pub fn satisfy_all_criteria(&mut self, satisfy_all: bool) -> &mut Self {
        self.params.satisfy_all_criteria = satisfy_all;
        self
    }

    /// Updates the overlays field
    pub(crate) fn find(&mut self) -> Vec<WindowInfo> {
        let hwnds = find_overlays(self.params.clone());
        let overlays: Vec<WindowInfo> = hwnds.into_iter()
            .filter_map(|handle| {
                let window_info = unsafe { WindowInfo::from_hwnd(handle) };
                Some(window_info)
            })
            .collect();

        self.overlays = overlays;
        self.overlays.clone()
    }
}