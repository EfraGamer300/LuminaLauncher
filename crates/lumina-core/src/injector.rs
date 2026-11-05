use std::ffi::c_void;
use std::mem::transmute;

use windows::Win32::Foundation::{CloseHandle, GetLastError, HANDLE};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::System::Memory::{
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, VirtualAllocEx, VirtualFreeEx,
};
use windows::Win32::System::Threading::{
    CREATE_SUSPENDED, CreateRemoteThread, INFINITE, OpenProcess, PROCESS_ALL_ACCESS, QueueUserAPC,
    ResumeThread, SetThreadDescription, TerminateThread, WaitForSingleObject,
};
use windows::core::PCWSTR;

use crate::pe::LibraryInfo;
use crate::{Error, Result};

type ThreadStart = unsafe extern "system" fn(*mut c_void) -> u32;
type ApcFunc = unsafe extern "system" fn(usize);

pub fn inject(pid: u32, library: &LibraryInfo) -> Result<()> {
    let mut paths = library.imports.clone();
    paths.push(library.path.to_string_lossy().to_string());

    unsafe {
        let process = OpenProcess(PROCESS_ALL_ACCESS, false, pid)?;
        let result = inject_impl(process, pid, &paths);
        let _ = CloseHandle(process);
        result
    }
}

unsafe fn inject_impl(process: HANDLE, _pid: u32, paths: &[String]) -> Result<()> {
    unsafe {
        let kernel32 = GetModuleHandleW(windows::core::w!("kernel32.dll"))?;
        let address = GetProcAddress(kernel32, windows::core::s!("LoadLibraryW"))
            .ok_or(Error::InjectionFailed)?;

        let start: ThreadStart = transmute(address);
        let apc: ApcFunc = transmute(address);

        let thread = CreateRemoteThread(
            process,
            None,
            0,
            Some(start),
            None,
            CREATE_SUSPENDED.0,
            None,
        )?;

        let mut allocations: Vec<*mut c_void> = Vec::new();
        let mut outcome: Result<()> = Ok(());

        for path in paths {
            let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
            let size = wide.len() * std::mem::size_of::<u16>();

            let remote = VirtualAllocEx(
                process,
                None,
                size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            );
            if remote.is_null() {
                outcome = Err(Error::InjectionFailed);
                break;
            }
            allocations.push(remote);

            if let Err(error) = windows::Win32::System::Diagnostics::Debug::WriteProcessMemory(
                process,
                remote,
                wide.as_ptr() as *const c_void,
                size,
                None,
            ) {
                outcome = Err(error.into());
                break;
            }

            if QueueUserAPC(Some(apc), thread, remote as usize) == 0 {
                let code = GetLastError().0;
                outcome = Err(Error::Message(format!(
                    "QueueUserAPC falhou (código {code})"
                )));
                break;
            }
        }

        if outcome.is_ok() {
            let metadata = crate::win::wide::to_wide("{}");
            let _ = SetThreadDescription(thread, PCWSTR(metadata.as_ptr()));
            let _ = ResumeThread(thread);
            WaitForSingleObject(thread, INFINITE);
        }

        let _ = TerminateThread(thread, 0);
        let _ = CloseHandle(thread);

        for pointer in allocations {
            let _ = VirtualFreeEx(process, pointer, 0, MEM_RELEASE);
        }

        outcome
    }
}
