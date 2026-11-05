use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();

pub fn init(path: &Path) -> std::io::Result<()> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    let _ = LOG_FILE.set(Mutex::new(file));
    info("log inicializado");
    Ok(())
}

fn write(level: &str, message: &str) {
    let line = format!("[{level}] {message}\n");
    if let Some(lock) = LOG_FILE.get()
        && let Ok(mut file) = lock.lock()
    {
        let _ = file.write_all(line.as_bytes());
    }
    if cfg!(debug_assertions) {
        eprint!("{line}");
    }
}

pub fn info(message: &str) {
    write("INFO", message);
}

pub fn warn(message: &str) {
    write("WARN", message);
}

pub fn error(message: &str) {
    write("ERROR", message);
}
