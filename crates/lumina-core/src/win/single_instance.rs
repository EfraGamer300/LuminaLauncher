use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;
use windows::core::PCWSTR;

use crate::Result;
use crate::win::wide::to_wide;

pub struct Instance {
    handle: HANDLE,
}

impl Instance {
    pub fn acquire(name: &str) -> Result<Option<Self>> {
        let name = to_wide(name);
        let handle = unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr()))? };
        let already_exists = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;

        if already_exists {
            unsafe {
                let _ = CloseHandle(handle);
            }
            return Ok(None);
        }

        Ok(Some(Self { handle }))
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

pub fn exists(name: &str) -> bool {
    let name = to_wide(name);
    match unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr())) } {
        Ok(handle) => {
            let already_exists = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
            unsafe {
                let _ = CloseHandle(handle);
            }
            already_exists
        }
        Err(_) => false,
    }
}
