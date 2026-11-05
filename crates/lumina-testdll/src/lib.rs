use std::ffi::c_void;
use std::io::Write;

const DLL_PROCESS_ATTACH: u32 = 1;

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(_instance: *mut c_void, reason: u32, _reserved: *mut c_void) -> i32 {
    if reason == DLL_PROCESS_ATTACH {
        std::thread::spawn(|| {
            let path = std::env::temp_dir().join("lumina_injected.txt");
            if let Ok(mut file) = std::fs::File::create(path) {
                let _ = writeln!(
                    file,
                    "injected pid={} at {:?}",
                    std::process::id(),
                    std::time::SystemTime::now()
                );
            }
        });
    }

    1
}
