use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::PCWSTR;

use crate::win::wide::to_wide;
use crate::{Error, Result};

pub fn open(target: &str) -> Result<()> {
    let operation = to_wide("open");
    let target = to_wide(target);

    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(operation.as_ptr()),
            PCWSTR(target.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        )
    };

    if result.0 as isize <= 32 {
        return Err(Error::Message(format!("não foi possível abrir {target:?}")));
    }

    Ok(())
}
