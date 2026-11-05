#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bridge;
mod diagnostics;

use std::path::Path;
use std::sync::{Arc, Mutex};

use lumina_core::config::Config;
use lumina_core::win::single_instance::Instance;
use lumina_core::{injector, logging, minecraft, paths, pe};
use slint::ComponentHandle;

slint::include_modules!();

fn main() {
    let _ = paths::ensure_dirs();
    let _ = logging::init(&paths::log_path());

    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("--inject") => {
            let Some(path) = args.get(2) else {
                eprintln!("uso: lumina-launcher --inject <caminho-da-dll>");
                std::process::exit(2);
            };

            if let Err(error) = run_inject(path) {
                eprintln!("erro: {error}");
                std::process::exit(1);
            }
        }
        Some("--diagnose") => {
            let config = Config::load();
            let report = diagnostics::report(&config);
            println!("{report}");

            let path = paths::diagnose_path();
            let _ = std::fs::write(&path, &report);

            if cfg!(not(debug_assertions)) {
                let _ = lumina_core::win::shell::open(&path.to_string_lossy());
            }
        }
        _ => {
            if let Err(error) = run_ui() {
                logging::error(&format!("erro fatal: {error}"));
                eprintln!("erro fatal: {error}");
            }
        }
    }
}

fn run_inject(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load();
    let library = pe::inspect(Path::new(path))?;
    let status = minecraft::status(&config.minecraft)?;

    let pid = status
        .pid
        .ok_or_else(|| lumina_core::Error::Message("o Minecraft precisa estar aberto".into()))?;

    injector::inject(pid, &library)?;
    println!("injetado em pid {pid} ({} imports)", library.imports.len());
    Ok(())
}

fn run_ui() -> Result<(), Box<dyn std::error::Error>> {
    let instance = Instance::acquire("Local\\LuminaLauncher")?;
    if instance.is_none() {
        return Ok(());
    }

    let config = Arc::new(Mutex::new(Config::load()));
    let ui = AppWindow::new()?;

    {
        let config = config
            .lock()
            .map_err(|_| lumina_core::Error::Message("configuração envenenada".into()))?;

        ui.set_launcher_version(config.launcher_version.clone().into());
        ui.set_status("Pronto!".into());
        ui.set_performance_mode(config.performance_mode);
        ui.set_automatic_updates(config.automatic_updates);
        ui.set_custom_dll_path(
            config
                .client
                .custom_dll
                .as_ref()
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_default()
                .into(),
        );
        ui.set_dll_valid(
            config
                .client
                .custom_dll
                .as_ref()
                .is_some_and(|path| path.is_file()),
        );
    }

    bridge::install(&ui, config.clone());
    bridge::start_polling(&ui, config.clone());

    ui.run()?;

    if let Ok(config) = config.lock() {
        let _ = config.save();
    }

    drop(instance);
    Ok(())
}
