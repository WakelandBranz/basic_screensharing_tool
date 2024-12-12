// Converts SystemHandles to a more easily usable and readable format

use windows::{
    Win32::{
        Foundation::{
            HANDLE,
            CloseHandle,
        },
        System::{
            Threading::{
                OpenProcess,
                QueryFullProcessImageNameA,
                PROCESS_NAME_FORMAT,
                PROCESS_QUERY_INFORMATION,
                PROCESS_VM_READ,
            },
            ProcessStatus::{
                K32GetProcessImageFileNameA,
            }
        }
    },
    core::{
        PSTR,
    }
};

use std::fmt;
use crate::core::handle::{HandleError, SystemHandleEntry};

#[derive(Clone)]
#[derive(Default)]
pub struct HandleInfo {
    handle: HANDLE,
    pub nt_path: String,
    pub win32_path: String,
    pub access_rights: Vec<String>,
}

impl HandleInfo {
    pub fn from_handle_entry(entry: SystemHandleEntry) -> Result<Self, HandleError> {
        let mut info = HandleInfo {
            handle: HANDLE(entry.handle_value as _),
            nt_path: String::new(),
            win32_path: String::new(),
            access_rights: Vec::new(),

            ..Default::default()
        };

        // Only try to get paths if this is a process or file handle

        unsafe {
            // Open the process that owns the handle
            let process_handle: HANDLE = OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                false,
                entry.process_id as u32,
            ).map_err(|_| HandleError::FailedToOpenProcess)?;

            if let Err(e) = info.get_process_nt_path(process_handle) {
                log::debug!("Failed to get NT path: {:?}", e);
            }

            if let Err(e) = info.get_process_win32_path(process_handle) {
                log::debug!("Failed to get Win32 path: {:?}", e);
            }

            CloseHandle(process_handle).expect("Failed to close handle!");
        }


        // Decode the access rights
        info.access_rights = decode_access_mask(entry.granted_access);

        Ok(info)
    }

    /// Gets a process' nt path from a pid
    fn get_process_nt_path(&mut self, handle: HANDLE) -> Result<(), HandleError> {
        unsafe {
            let mut buffer = [0u8; 260];
            let length = K32GetProcessImageFileNameA(
                handle,
                &mut buffer
            );

            if length == 0 {
                return Err(HandleError::FailedToGetProcessNtPath)
            }

            self.nt_path = String::from_utf8_lossy(&buffer[..length as usize]).to_string();
        }

        Ok(())
    }

    /// Gets a process' Win32 path from a pid
    fn get_process_win32_path(&mut self, handle: HANDLE) -> Result<(), HandleError> {
        unsafe {
            let mut buffer = [0u8; 260];
            let mut size = buffer.len() as u32;

            match QueryFullProcessImageNameA(
                handle,
                PROCESS_NAME_FORMAT(0),
                PSTR(buffer.as_mut_ptr()),
                &mut size
            ) {
                Ok(()) => {
                    self.win32_path = String::from_utf8_lossy(&buffer[..size as usize]).to_string();
                    Ok(())
                },
                Err(_) => Err(HandleError::FailedToGetProcessWin32Path)
            }
        }
    }
}

/// Gets readable permissions for a handle
fn decode_access_mask(access_mask: u32) -> Vec<String> {
    let mut rights = Vec::new();

    // Generic rights
    if access_mask & 0x80000000 != 0 { rights.push("GENERIC_READ".to_string()); }
    if access_mask & 0x40000000 != 0 { rights.push("GENERIC_WRITE".to_string()); }
    if access_mask & 0x20000000 != 0 { rights.push("GENERIC_EXECUTE".to_string()); }
    if access_mask & 0x10000000 != 0 { rights.push("GENERIC_ALL".to_string()); }

    // Specific rights
    if access_mask & 0x0001 != 0 { rights.push("PROCESS_TERMINATE".to_string()); }
    if access_mask & 0x0002 != 0 { rights.push("PROCESS_CREATE_THREAD".to_string()); }
    if access_mask & 0x0008 != 0 { rights.push("PROCESS_VM_OPERATION".to_string()); }
    if access_mask & 0x0010 != 0 { rights.push("PROCESS_VM_READ".to_string()); }
    if access_mask & 0x0020 != 0 { rights.push("PROCESS_VM_WRITE".to_string()); }
    if access_mask & 0x0040 != 0 { rights.push("PROCESS_DUP_HANDLE".to_string()); }
    if access_mask & 0x0080 != 0 { rights.push("PROCESS_CREATE_PROCESS".to_string()); }
    if access_mask & 0x0100 != 0 { rights.push("PROCESS_SET_QUOTA".to_string()); }
    if access_mask & 0x0200 != 0 { rights.push("PROCESS_SET_INFORMATION".to_string()); }
    if access_mask & 0x0400 != 0 { rights.push("PROCESS_QUERY_INFORMATION".to_string()); }
    if access_mask & 0x1000 != 0 { rights.push("PROCESS_SUSPEND_RESUME".to_string()); }

    rights
}

impl fmt::Debug for HandleInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HandleInfo")
            .field("handle", &self.handle)
            .field("nt_path", &self.nt_path)
            .field("win32_path", &self.win32_path)
            .field("access_rights", &self.access_rights)
            .finish()
    }
}
