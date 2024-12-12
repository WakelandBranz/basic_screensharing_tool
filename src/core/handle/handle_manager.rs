
use sysinfo::get_current_pid;
use windows::{
    Win32::{
        System::{
            Memory::{
                VirtualAlloc,
                VirtualFree,
                PAGE_EXECUTE_READWRITE,
                MEM_COMMIT,
                MEM_RELEASE,
            },
            Threading::{
                PROCESS_ALL_ACCESS,
                PROCESS_VM_READ,
                PROCESS_VM_WRITE
            }
        },
        Foundation::{
            HANDLE,
            STATUS_SUCCESS,

        }
    },
    Wdk::{
        System::{
            SystemInformation::{
                NtQuerySystemInformation,
                SYSTEM_INFORMATION_CLASS,
            }
        },
        Foundation::{
            NtQueryObject,
            ObjectTypeInformation,
        }
    }
};
use windows::Win32::Foundation::{CloseHandle, DuplicateHandle, DUPLICATE_SAME_ACCESS};
use windows::Win32::System::Threading::{GetCurrentProcess, GetProcessId, OpenProcess, PROCESS_DUP_HANDLE, PROCESS_QUERY_INFORMATION};
use crate::core::handle::{
    SystemHandleInformation,
    SystemHandleEntry,
    SystemHandleType,
    handle_context::HandleContext,
    HandleError,
    ObjectTypeInformation
};
use crate::core::handle::handle_info::HandleInfo;

const SYSTEM_HANDLE_INFORMATION: i32 = 0x10; // 16

pub struct HandleManager {
    pub handles: Vec<HandleContext>,
}

impl HandleManager {
    pub fn new() -> Result<Self, HandleError> {
        let raw_handles = Self::query_system_handles()?;
        let handles = raw_handles.into_iter()
            .map(|raw| HandleContext {
                raw,
                info: None,
            })
            .collect();

        Ok(Self { handles })
    }

    fn query_system_handles() -> Result<Vec<SystemHandleEntry>, HandleError> {
        let mut entries = Vec::new();

        unsafe {
            // Start with a reasonable initial buffer size (4KB)
            let mut info_length: u32 = 0x1000;

            // Allocate initial memory for handle information
            let mut buffer = VirtualAlloc(
                Some(std::ptr::null_mut()),
                info_length as usize,
                MEM_COMMIT,
                PAGE_EXECUTE_READWRITE
            );

            // First attempt to query system handles
            // This will likely fail with STATUS_INFO_LENGTH_MISMATCH
            // but will update info_length with the required size
            let mut status = NtQuerySystemInformation(
                SYSTEM_INFORMATION_CLASS(SYSTEM_HANDLE_INFORMATION),
                buffer,
                info_length,
                &mut info_length,
            );

            log::debug!("Status: {:x?} | Info length: {:?}| Getting buffer", status, info_length);

            // Keep trying until we get a successful query
            while status != STATUS_SUCCESS {
                // Free the old buffer FIRST
                VirtualFree(buffer, 0, MEM_RELEASE)
                    .map_err(|_| HandleError::FailedToFreeMemory)?;

                // Allocate new buffer with updated size
                buffer = VirtualAlloc(
                    Some(std::ptr::null_mut()),
                    info_length as usize,
                    MEM_COMMIT,
                    PAGE_EXECUTE_READWRITE
                );

                if buffer.is_null() {
                    return Err(HandleError::FailedToAllocateMemory);
                }

                // Try to query with current buffer
                status = NtQuerySystemInformation(
                    SYSTEM_INFORMATION_CLASS(SYSTEM_HANDLE_INFORMATION),
                    buffer,
                    info_length,
                    &mut info_length,
                );

                log::debug!("Status: {:x?} | Info length: {:?}| Getting buffer", status, info_length);
            }

            // We have now successfully retrieved all handles
            log::debug!("Buffer: {:?}", buffer);

            let handle_info = &*(buffer as *const SystemHandleInformation);
            log::debug!("Number of handles: {}", handle_info.number_of_handles);

            // Get handles slice and clone each entry into our vector
            let handles = std::slice::from_raw_parts(
                &handle_info.handles as *const SystemHandleEntry,
                handle_info.number_of_handles as usize
            );

            entries = handles.to_vec();  // Simply clone all entries into our vector

            // Clean up allocated memory
            VirtualFree(buffer, 0, MEM_RELEASE)
                .map_err(|_| HandleError::FailedToFreeMemory)?;
        }

        Ok(entries)
    }

    fn query_handle_type_info(&self, handle: HANDLE) -> Result<ObjectTypeInformation, HandleError> {
        unsafe {
            let mut info_length = 0u32;

            // First call to get required buffer size
            let mut status = NtQueryObject(
                handle,
                ObjectTypeInformation,
                None,
                0,
                Some(&mut info_length)
            );

            if info_length == 0 {
                return Err(HandleError::FailedToQueryObject);
            }

            // Now allocate buffer with known size
            let mut buffer = VirtualAlloc(
                Some(std::ptr::null_mut()),
                info_length as usize,
                MEM_COMMIT,
                PAGE_EXECUTE_READWRITE
            );

            if buffer.is_null() {
                return Err(HandleError::FailedToAllocateMemory);
            }

            while status != STATUS_SUCCESS {
                // Free the old buffer FIRST
                VirtualFree(buffer, 0, MEM_RELEASE)
                    .map_err(|_| HandleError::FailedToFreeMemory)?;

                // Allocate new buffer with updated size
                buffer = VirtualAlloc(
                    Some(std::ptr::null_mut()),
                    info_length as usize,
                    MEM_COMMIT,
                    PAGE_EXECUTE_READWRITE
                );

                if buffer.is_null() {
                    log::debug!("Failed to allocate memory!");
                    return Err(HandleError::FailedToAllocateMemory);
                }

                // Try to query with new buffer
                status = NtQueryObject(
                    handle,
                    ObjectTypeInformation,
                    Some(buffer),
                    info_length,
                    Some(&mut info_length)
                );

            }

            // Extract relevant information
            let type_info = ObjectTypeInformation {
                type_index: (*(buffer as *const ObjectTypeInformation)).type_index,
                total_handles: (*(buffer as *const ObjectTypeInformation)).total_handles,
                valid_access: (*(buffer as *const ObjectTypeInformation)).valid_access,
            };

            VirtualFree(buffer, 0, MEM_RELEASE)
                .map_err(|_| HandleError::FailedToFreeMemory)?;
            Ok(type_info)
        }
    }

    /// Filter handles that are attached to our target process (but not owned by it)
    /// Keep handles that:
    /// 1. Point to our target process
    /// 2. Are NOT owned by our target process
    pub fn test_filter_handles_to_target(&mut self, target_pid: u32) -> Result<&mut Self, HandleError> {
        let target_handle = unsafe {
            OpenProcess(
                PROCESS_QUERY_INFORMATION,
                false,
                target_pid
            ).map_err(|_| HandleError::FailedToOpenProcess)?
        };

        let target_ptr = target_handle.0;
        log::debug!("Target process handle ptr: {:?}", target_ptr);

        let initial_count = self.handles.len();
        log::debug!("Initial handle count: {}", initial_count);

        let mut filtered_count = 0;
        // Filter handles
        self.handles.retain(|handle| {
            // Skip if handle belongs to target process
            if handle.raw.process_id as u32 == target_pid {
                return false;
            }

            // Try to duplicate the handle from its owning process
            unsafe {
                let process_handle = OpenProcess(
                    PROCESS_DUP_HANDLE,
                    false,
                    handle.raw.process_id as u32
                );

                if let Ok(process_handle) = process_handle {
                    log::debug!("Successfully opened process {} for handle duplication", handle.raw.process_id);
                    let mut duplicated_handle = HANDLE::default();

                    let dup_result = DuplicateHandle(
                        process_handle,
                        HANDLE(handle.raw.handle_value as _),
                        GetCurrentProcess(),
                        &mut duplicated_handle,
                        0,
                        false,
                        DUPLICATE_SAME_ACCESS,
                    );

                    CloseHandle(process_handle)
                        .map_err(|_| HandleError::FailedToCloseHandle)
                        .unwrap();

                    if dup_result.is_ok() {
                        // Compare raw pointer values
                        let is_target = handle.raw.object == target_ptr;
                        if is_target {
                            filtered_count += 1;
                            log::debug!("Found matching handle from PID {} with access {:x}",
                            handle.raw.process_id, handle.raw.granted_access);
                        }
                        CloseHandle(duplicated_handle)
                            .map_err(|_| HandleError::FailedToCloseHandle)
                            .unwrap();
                        return is_target;
                    }
                }
            }
            false
        });

        log::debug!("After target process filter: found {} valid handles", filtered_count);

        unsafe { CloseHandle(target_handle) }
            .map_err(|_| HandleError::FailedToCloseHandle)?;

        Ok(self)
    }

    pub fn filter_handles_to_target(&mut self, target_pid: u32) -> Result<&mut Self, HandleError> {
        let target_handle = unsafe {
            OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,  // Add VM_READ to get more info
                false,
                target_pid
            ).map_err(|_| HandleError::FailedToOpenProcess)?
        };

        log::debug!("Target process handle: {:?}", target_handle);

        let initial_count = self.handles.len();
        log::debug!("Initial handle count: {}", initial_count);

        let mut filtered_count = 0;
        self.handles.retain(|handle| {
            // Skip if handle belongs to target process
            if handle.raw.process_id as u32 == target_pid {
                return false;
            }

            unsafe {
                let process_handle = OpenProcess(
                    PROCESS_DUP_HANDLE,
                    false,
                    handle.raw.process_id as u32
                );

                if let Ok(process_handle) = process_handle {
                    let mut duplicated_handle = HANDLE::default();

                    let dup_result = DuplicateHandle(
                        process_handle,
                        HANDLE(handle.raw.handle_value as _),
                        GetCurrentProcess(),
                        &mut duplicated_handle,
                        0,
                        false,
                        DUPLICATE_SAME_ACCESS,
                    );

                    CloseHandle(process_handle)
                        .map_err(|_| HandleError::FailedToCloseHandle)
                        .unwrap();

                    if dup_result.is_ok() {
                        // Get Process ID of the duplicated handle
                        let duplicated_pid = GetProcessId(duplicated_handle);

                        let is_target = duplicated_pid == target_pid;
                        if is_target {
                            filtered_count += 1;
                            log::debug!("Found matching handle from PID {} with access {:x}",
                            handle.raw.process_id, handle.raw.granted_access);
                        }
                        CloseHandle(duplicated_handle)
                            .map_err(|_| HandleError::FailedToCloseHandle)
                            .unwrap();
                        return is_target;
                    }
                }
            }
            false
        });

        log::debug!("After target process filter: found {} valid handles", filtered_count);

        unsafe { CloseHandle(target_handle) }
            .map_err(|_| HandleError::FailedToCloseHandle)?;

        Ok(self)
    }

    /// Filter's handle type
    pub fn filter_by_handle_type(&mut self, system_handle_type: SystemHandleType) -> &mut Self {
        self.handles.retain(|handle| handle.raw.object_type_index == system_handle_type as u8);
        log::debug!("After handle type filter: {} handles", self.handles.len());
        self
    }

    /// Filter handles by parent's process ID
    pub fn filter_by_parent_pid(&mut self, pid: u32) -> &mut Self {
        self.handles.retain(|handle| handle.raw.process_id as u32 == pid);
        self
    }

    /// Enrich filtered handles with additional information
    pub fn collect_handle_info(&mut self) -> Result<&mut Self, HandleError> {
        for handle in &mut self.handles {
            handle.info = Some(HandleInfo::from_handle_entry(handle.raw.clone())?);
        }
        Ok(self)
    }

    /// Filter handles by access rights
    pub fn filter_by_access(&mut self, required_access: u32) -> &mut Self {
        self.handles.retain(|handle| {
            handle.raw.granted_access & required_access == required_access
        });
        self
    }

    /// Filters suspicious handles based on predefined attributes
    pub fn filter_suspicious_handles(&mut self) -> &mut Self {
        let read_write_mask = PROCESS_VM_READ.0 | PROCESS_VM_WRITE.0;

        self.handles = self.handles
            .iter()
            .filter(|entry| {
                entry.raw.granted_access == PROCESS_ALL_ACCESS.0 ||
                    (entry.raw.granted_access & read_write_mask) == read_write_mask
            })
            .cloned()
            .collect();

        self
    }

    /// Filters out handles belonging to the anticheat
    pub fn filter_anticheat_handles(&mut self) -> &mut Self {
        let anticheat_pid = get_current_pid()
            .unwrap()
            .as_u32()
            as u16;

        self.handles = self.handles
            .iter()
            .filter(|entry| {
                entry.raw.process_id != anticheat_pid
            })
            .cloned()
            .collect();

        self
    }

    /// Get the final filtered and enriched handles
    pub fn get_handles(&self) -> Vec<HandleContext> {
        self.handles.to_vec()
    }
}