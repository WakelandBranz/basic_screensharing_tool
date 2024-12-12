use std::fmt;
use crate::core::handle::{
    SystemHandleEntry,
    handle_info::HandleInfo,
};

#[derive(Clone)]
pub struct HandleContext {
    pub(crate) raw: SystemHandleEntry,
    pub(crate) info: Option<HandleInfo>,
}

impl HandleContext {
    pub fn access_rights(&self) -> &[String] {
        match &self.info {
            Some(info) => &info.access_rights,
            None => &[],
        }
    }

    pub fn paths(&self) -> Option<(&String, &String)> {
        self.info.as_ref().map(|info| {
            (&info.nt_path, &info.win32_path)
        })
    }
}

impl fmt::Display for HandleContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "   Process ID: {}", self.raw.process_id)?;
        writeln!(f, "  Access Rights: {:#x}", self.raw.granted_access)?;

        // Show access rights if info is available
        if !self.access_rights().is_empty() {
            writeln!(f, "  Decoded Access Rights:")?;
            for right in self.access_rights() {
                writeln!(f, "    - {}", right)?;
            }
        }

        // Show paths if available
        if let Some((nt_path, win32_path)) = self.paths() {
            writeln!(f, "  NT Path: {}", nt_path)?;
            writeln!(f, "  Win32 Path: {}", win32_path)?;
        }

        Ok(())
    }
}