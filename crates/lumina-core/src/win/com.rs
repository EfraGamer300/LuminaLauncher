use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx};

use crate::Result;

pub fn initialize_mta() -> Result<()> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
    Ok(())
}
