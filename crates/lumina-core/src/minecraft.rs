use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use windows::Win32::System::Threading::CREATE_NO_WINDOW;

use notify::Watcher;

use crate::config::{ClientConfig, MinecraftConfig};
use crate::win::{pkg, process, window};
use crate::{Error, Result, logging};

const GAMING_SERVICES_FAMILY: &str = "Microsoft.GamingServices_8wekyb3d8bbwe";
const LAUNCH_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone)]
pub struct MinecraftStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub installed_path: Option<PathBuf>,
    pub is_store: bool,
    pub running: bool,
    pub pid: Option<u32>,
    pub window_visible: bool,
}

pub fn gaming_services_installed() -> bool {
    pkg::is_installed(GAMING_SERVICES_FAMILY)
}

pub fn status(config: &MinecraftConfig) -> Result<MinecraftStatus> {
    let info = pkg::find_info(&config.package_family)?;
    let pid = process::find_pid_by_package(&config.process_name, &config.package_family)?;

    let window_visible = window::find_window(&config.window_class, pid)
        .is_some_and(|(handle, _)| window::is_visible(handle));

    Ok(MinecraftStatus {
        installed: info.is_some(),
        version: info.as_ref().map(|info| info.version.clone()),
        installed_path: info.as_ref().map(|info| info.installed_path.clone()),
        is_store: info.as_ref().map(|info| info.is_store).unwrap_or(false),
        running: pid.is_some(),
        pid,
        window_visible,
    })
}

pub fn launch(config: &MinecraftConfig) -> Result<u32> {
    if let Some((handle, pid)) = window::find_window(&config.window_class, None)
        && window::is_visible(handle)
    {
        window::activate(handle);
        return Ok(pid);
    }

    if let Some(pid) = process::find_pid_by_package(&config.process_name, &config.package_family)? {
        wait_ready(config, pid)?;
        return Ok(pid);
    }

    let info = pkg::find_info(&config.package_family)?.ok_or(Error::MinecraftNotInstalled)?;

    if !info.is_store {
        return Err(Error::SideloadedUnsupported);
    }

    let executable = info.installed_path.join(&config.process_name);
    if !executable.is_file() {
        return Err(Error::Message(format!(
            "executável do Minecraft não encontrado em {}",
            executable.display()
        )));
    }

    let command = format!(
        "Invoke-CommandInDesktopPackage -PackageFamilyName '{}' -AppId '{}' -Command '{}'",
        config.package_family,
        config.app_id,
        executable.display()
    );

    let powershell = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
        .join("System32")
        .join("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe");

    logging::info(&format!("iniciando Minecraft via {}", powershell.display()));

    std::process::Command::new(&powershell)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &command,
        ])
        .creation_flags(CREATE_NO_WINDOW.0)
        .spawn()?
        .wait()?;

    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Some(pid) =
            process::find_pid_by_package(&config.process_name, &config.package_family)?
        {
            logging::info(&format!("Minecraft iniciado (pid {pid})"));
            wait_ready(config, pid)?;
            return Ok(pid);
        }

        if Instant::now() >= deadline {
            return Err(Error::LaunchTimeout);
        }

        std::thread::sleep(Duration::from_millis(200));
    }
}

fn wait_ready(config: &MinecraftConfig, pid: u32) -> Result<()> {
    let users = crate::paths::minecraft_users_dir();
    let _ = std::fs::create_dir_all(&users);

    let (sender, receiver) = std::sync::mpsc::channel();
    let watcher = notify::recommended_watcher(move |event| {
        let _ = sender.send(event);
    })
    .ok()
    .and_then(|mut watcher| {
        watcher
            .watch(&users, notify::RecursiveMode::Recursive)
            .ok()?;
        Some(watcher)
    });

    if watcher.is_none() {
        logging::warn("watcher do menu_load_lock indisponível; usando polling de janela");
    }

    let deadline = Instant::now() + LAUNCH_TIMEOUT;

    loop {
        if let Some((handle, _)) = window::find_window(&config.window_class, Some(pid))
            && window::is_visible(handle)
        {
            logging::info("Minecraft pronto (janela visível)");
            return Ok(());
        }

        if Instant::now() >= deadline {
            return Err(Error::LaunchTimeout);
        }

        match receiver.recv_timeout(Duration::from_millis(500)) {
            Ok(Ok(event)) => {
                if event
                    .paths
                    .iter()
                    .any(|path| path.to_string_lossy().contains("menu_load_lock"))
                {
                    logging::info("Minecraft pronto (menu_load_lock removido)");
                    return Ok(());
                }
            }
            Ok(Err(error)) => logging::warn(&format!("watcher: {error}")),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                std::thread::sleep(Duration::from_millis(500));
            }
        }
    }
}

pub fn wait_until_injected(pid: u32, client: &ClientConfig, library_name: &str) -> bool {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if process::is_module_loaded(pid, library_name).unwrap_or(false)
            || crate::client::is_injected(client)
        {
            return true;
        }

        if Instant::now() >= deadline {
            return false;
        }

        std::thread::sleep(Duration::from_millis(250));
    }
}
