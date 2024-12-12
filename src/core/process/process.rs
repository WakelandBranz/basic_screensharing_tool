// Thank you https://github.com/zofiaclient/memwar/tree/main/memwar/src (adapted for my use)

use std::ffi::{c_void, CStr, CString};
use sysinfo::System;
use windows::Win32::{
    Foundation::{
        HANDLE,
        CloseHandle
    },
    System::{
        Threading::{
            OpenProcess,
            PROCESS_ACCESS_RIGHTS,
            PROCESS_ALL_ACCESS,
        },
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot,
            TH32CS_SNAPMODULE,
            TH32CS_SNAPMODULE32,
            Module32First,
            Module32Next,
            MODULEENTRY32,
        }
    }
};

pub type DWORD = u32;

/// Gets pid of process by name (case-insensitive!)
pub(super) fn get_pid_by_name(process_name: &str) -> Option<u32> {
    let s = System::new_all();
    let lower_name = process_name.to_ascii_lowercase();

    for (pid, process) in s.processes() {
        let process_name = process.name().to_string_lossy().to_ascii_lowercase();
        if process_name == lower_name {
            return Some(pid.as_u32());
        }
    }

    None
}

/// Opens a handle to the requested process
pub(super) unsafe fn open_process_handle(pid: u32) -> Result<HANDLE, DWORD> {
    let handle = OpenProcess(
        PROCESS_ACCESS_RIGHTS(PROCESS_ALL_ACCESS.0),
        false,
        pid,
    );

    match handle {
        Ok(handle) => Ok(handle),
        Err(error) => {
            // Get the error code from Windows
            let error_code = error.code().0 as u32;
            Err(error_code)
        }
    }
}

/// Returns a list of modules in the given process
pub(super) unsafe fn get_modules(pid: u32) -> Result<Vec<MODULEENTRY32>, DWORD> {
    let mut modules = Vec::new();

    // Store the unwrapped snapshot handle
    let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) {
        Ok(handle) => handle,
        Err(error) => return Err(error.code().0 as u32),
    };

    let mut mod_entry: MODULEENTRY32 = std::mem::zeroed();
    mod_entry.dwSize = size_of_val(&mod_entry) as _;

    // Get first module
    if let Ok(_) = Module32First(snapshot, &mut mod_entry) {
        // Add first module to vector
        modules.push(mod_entry);

        // Get remaining modules
        while let Ok(_) = Module32Next(snapshot, &mut mod_entry) {
            modules.push(mod_entry);
        }
    }

    CloseHandle(snapshot);
    Ok(modules)
}

/// Returns the base address of the module with the given name in the process.
///
/// This function will return Ok([null_mut]) if the module with the name provided could not be
/// found.
/// # Returns
/// * `Ok(*mut c_void)` - Pointer to module base address, or null if not found
/// * `Err(DWORD)` - Windows error code if operation fails
pub(super) unsafe fn get_mod_base(pid: u32, mod_name: &str) -> Result<*mut c_void, DWORD> {
    // Convert Rust string to null-terminated C string for Windows API compatibility
    // Will panic if string contains null bytes
    let c_mod_name = CString::new(mod_name)
        .expect("Could not create CString in get_mod_base!");

    // Iterate through all modules in the process
    for module in get_modules(pid)? {
        // Convert module name (Windows char array) to C string and compare with target name
        if CStr::from_ptr(module.szModule.as_ptr()) == c_mod_name.as_c_str() {
            // Found matching module - return its base address
            return Ok(module.modBaseAddr as _)
        }
    }

    // Module not found - return null pointer
    // This is not an error condition, just indicates module wasn't found
    Ok(std::ptr::null_mut())
}