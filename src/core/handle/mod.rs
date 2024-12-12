use windows::Win32::Foundation::HANDLE;

pub mod handle_manager;
pub(crate) mod handle_info;

pub mod handle_context;

#[derive(Debug)]
pub enum HandleError {
    FailedToOpenProcess,
    FailedToGetProcessNtPath,
    FailedToGetProcessWin32Path,
    FailedToQueryObject,
    FailedToFreeMemory,
    FailedToAllocateMemory,
    FailedToCloseHandle,
}

// Minimal Display implementation - just show the variant name
impl std::fmt::Display for HandleError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for HandleError {}

#[derive(Debug)]
pub enum SystemHandleError {
    MemoryAllocationFailed,
}

#[repr(C)]
pub struct SystemHandleInformation {
    number_of_handles: u32,
    #[cfg(target_arch = "x86")]
    _padding: u32,  // Padding for alignment on x86
    handles: [SystemHandleEntry; 1], // Variable length array
}

#[repr(C)]
#[derive(Clone)]
pub struct SystemHandleEntry {
    pub(crate) process_id: u16,                   // UniqueProcessId
    pub(crate) creator_back_trace_index: u16,     // CreatorBackTraceIndex
    pub(crate) object_type_index: u8,             // ObjectTypeIndex
    pub(crate) handle_attributes: u8,             // HandleAttributes
    pub(crate) handle_value: u16,                 // HandleValue
    pub(crate) object: *mut std::ffi::c_void,     // Object
    pub(crate) granted_access: u32,               // GrantedAccess
}

impl SystemHandleEntry {
    fn to_handle(&self) -> HANDLE {
        HANDLE(self.handle_value as _)
    }
}

impl std::fmt::Debug for SystemHandleEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemHandleEntry")
            .field("process_id", &self.process_id)
            .field("handle_value", &format!("0x{:x}", self.handle_value))
            .field("object_type_index", &self.object_type_index)
            .field("granted_access", &format!("0x{:x}", self.granted_access))
            .finish()
    }
}

#[derive(Copy, Clone)]
pub enum SystemHandleType {
    Process = 7, // Really the only useful one, but the rest are just in case... it can't hurt
    Thread = 8,
    Event = 10,
    Mutex = 11,
    Sempahore = 12,
    File = 25,
}

#[derive(Debug)]
#[repr(C)]
struct ObjectTypeInformation {
    type_index: u8,
    total_handles: u32,
    valid_access: u32,
}
