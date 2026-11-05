use std::mem::size_of;

use windows::Win32::Foundation::{CloseHandle, ERROR_SUCCESS, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::Packaging::Appx::GetPackageFamilyName;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, MODULEENTRY32W, Module32FirstW, Module32NextW, PROCESSENTRY32W,
    Process32FirstW, Process32NextW, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::core::PWSTR;

use crate::Result;
use crate::win::wide::from_wide;

fn snapshot(
    flags: windows::Win32::System::Diagnostics::ToolHelp::CREATE_TOOLHELP_SNAPSHOT_FLAGS,
    pid: u32,
) -> Result<HANDLE> {
    let handle = unsafe { CreateToolhelp32Snapshot(flags, pid)? };
    if handle == INVALID_HANDLE_VALUE {
        return Err(crate::Error::Message("falha ao enumerar processos".into()));
    }
    Ok(handle)
}

fn package_family_of(pid: u32) -> Option<String> {
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut length = 260u32;
        let mut buffer = vec![0u16; length as usize];
        let status = GetPackageFamilyName(process, &mut length, Some(PWSTR(buffer.as_mut_ptr())));
        let _ = CloseHandle(process);

        if status != ERROR_SUCCESS {
            return None;
        }

        let family = from_wide(&buffer);
        if family.is_empty() {
            None
        } else {
            Some(family)
        }
    }
}

pub fn find_pid_by_package(process_name: &str, package_family: &str) -> Result<Option<u32>> {
    unsafe {
        let handle = snapshot(TH32CS_SNAPPROCESS, 0)?;
        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        let mut found = None;
        if Process32FirstW(handle, &mut entry).is_ok() {
            loop {
                let name = from_wide(&entry.szExeFile);
                if name.eq_ignore_ascii_case(process_name)
                    && let Some(family) = package_family_of(entry.th32ProcessID)
                    && family.eq_ignore_ascii_case(package_family)
                {
                    found = Some(entry.th32ProcessID);
                    break;
                }

                if Process32NextW(handle, &mut entry).is_err() {
                    break;
                }
            }
        }

        let _ = CloseHandle(handle);
        Ok(found)
    }
}

pub fn is_module_loaded(pid: u32, module_name: &str) -> Result<bool> {
    unsafe {
        let handle = snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid)?;
        let mut entry = MODULEENTRY32W {
            dwSize: size_of::<MODULEENTRY32W>() as u32,
            ..Default::default()
        };

        let mut found = false;
        if Module32FirstW(handle, &mut entry).is_ok() {
            loop {
                let name = from_wide(&entry.szModule);
                if name.eq_ignore_ascii_case(module_name) {
                    found = true;
                    break;
                }

                if Module32NextW(handle, &mut entry).is_err() {
                    break;
                }
            }
        }

        let _ = CloseHandle(handle);
        Ok(found)
    }
}
