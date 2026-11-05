use std::path::PathBuf;

fn env_path(name: &str) -> PathBuf {
    std::env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_default()
}

pub fn local_app_data() -> PathBuf {
    let path = env_path("LOCALAPPDATA");
    if path.as_os_str().is_empty() {
        std::env::current_dir().unwrap_or_default()
    } else {
        path
    }
}

pub fn roaming_app_data() -> PathBuf {
    let path = env_path("APPDATA");
    if path.as_os_str().is_empty() {
        std::env::current_dir().unwrap_or_default()
    } else {
        path
    }
}

pub fn base_dir() -> PathBuf {
    local_app_data().join("Lumina")
}

pub fn launcher_dir() -> PathBuf {
    base_dir().join("Launcher")
}

pub fn client_dir() -> PathBuf {
    base_dir().join("Client")
}

pub fn config_path() -> PathBuf {
    launcher_dir().join("lumina.json")
}

pub fn log_path() -> PathBuf {
    launcher_dir().join("lumina.log")
}

pub fn diagnose_path() -> PathBuf {
    launcher_dir().join("diagnose.txt")
}

pub fn minecraft_users_dir() -> PathBuf {
    roaming_app_data().join("Minecraft Bedrock").join("Users")
}

pub fn ensure_dirs() -> std::io::Result<()> {
    std::fs::create_dir_all(launcher_dir())?;
    std::fs::create_dir_all(client_dir())?;
    Ok(())
}
