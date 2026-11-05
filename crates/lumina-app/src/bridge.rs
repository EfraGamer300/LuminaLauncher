use std::sync::{Arc, Mutex};
use std::time::Duration;

use slint::ComponentHandle;

use lumina_core::config::{ClientMode, Config};
use lumina_core::win::{shell, window};
use lumina_core::{client, injector, minecraft, paths, pe};

use crate::AppWindow;

#[derive(Clone)]
struct Ui {
    weak: slint::Weak<AppWindow>,
    action: Arc<Mutex<i32>>,
}

impl Ui {
    fn new(weak: slint::Weak<AppWindow>, action: Arc<Mutex<i32>>) -> Self {
        Self { weak, action }
    }

    fn update(&self, callback: impl FnOnce(&AppWindow) + Send + 'static) {
        let weak = self.weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = weak.upgrade() {
                callback(&ui);
            }
        });
    }

    fn status(&self, value: impl Into<String>) {
        let value = value.into();
        self.update(move |ui| ui.set_status(value.into()));
    }

    fn busy(&self, value: bool) {
        self.update(move |ui| ui.set_busy(value));
    }

    fn dialog(&self, title: &str, message: &str, primary: &str, secondary: &str, action: i32) {
        if let Ok(mut current) = self.action.lock() {
            *current = action;
        }

        let title = title.to_string();
        let message = message.to_string();
        let primary = primary.to_string();
        let secondary = secondary.to_string();

        self.update(move |ui| {
            ui.set_dialog_title(title.into());
            ui.set_dialog_message(message.into());
            ui.set_dialog_primary(primary.into());
            ui.set_dialog_secondary(secondary.into());
            ui.set_dialog_visible(true);
        });
    }
}

pub fn install(ui: &AppWindow, config: Arc<Mutex<Config>>) {
    let action = Arc::new(Mutex::new(0i32));

    let weak = ui.as_weak();
    let config_launch = config.clone();
    let action_launch = action.clone();
    ui.on_launch(move || {
        let worker = Ui::new(weak.clone(), action_launch.clone());
        let config = config_launch.clone();
        std::thread::spawn(move || launch_flow(worker, config));
    });

    let weak = ui.as_weak();
    ui.on_open_settings(move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_settings_open(true);
        }
    });

    let weak = ui.as_weak();
    ui.on_close_settings(move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_settings_open(false);
        }
    });

    let weak = ui.as_weak();
    let config_pick = config.clone();
    ui.on_pick_dll(move || {
        let Some(file) = rfd::FileDialog::new()
            .add_filter("DLL", &["dll"])
            .pick_file()
        else {
            return;
        };

        let valid = match pe::inspect(&file) {
            Ok(_) => true,
            Err(error) => {
                if let Some(ui) = weak.upgrade() {
                    ui.set_dialog_title("DLL inválida".into());
                    ui.set_dialog_message(error.to_string().into());
                    ui.set_dialog_primary("OK".into());
                    ui.set_dialog_secondary("".into());
                    ui.set_dialog_visible(true);
                }
                false
            }
        };

        if let Ok(mut config) = config_pick.lock() {
            config.client.mode = ClientMode::Custom;
            config.client.custom_dll = Some(file.clone());
            let _ = config.save();
        }

        if let Some(ui) = weak.upgrade() {
            ui.set_custom_dll_path(file.to_string_lossy().to_string().into());
            ui.set_dll_valid(valid);
        }
    });

    ui.on_open_client_folder(|| {
        let _ = shell::open(&paths::client_dir().to_string_lossy());
    });

    ui.on_open_launcher_folder(|| {
        let _ = shell::open(&paths::launcher_dir().to_string_lossy());
    });

    ui.on_minimize(window::minimize_foreground);
    ui.on_start_drag(window::start_drag);

    let weak = ui.as_weak();
    ui.on_close_window(move || {
        if let Some(ui) = weak.upgrade() {
            let _ = ui.hide();
        }
        let _ = slint::quit_event_loop();
    });

    let weak = ui.as_weak();
    let config_performance = config.clone();
    ui.on_toggle_performance(move |value| {
        if let Ok(mut config) = config_performance.lock() {
            config.performance_mode = value;
            let _ = config.save();
        }

        if let Some(ui) = weak.upgrade() {
            ui.set_performance_mode(value);
        }
    });

    let weak = ui.as_weak();
    let config_updates = config.clone();
    ui.on_toggle_auto_updates(move |value| {
        if let Ok(mut config) = config_updates.lock() {
            config.automatic_updates = value;
            let _ = config.save();
        }

        if let Some(ui) = weak.upgrade() {
            ui.set_automatic_updates(value);
        }
    });

    let weak = ui.as_weak();
    let action_primary = action.clone();
    ui.on_dialog_primary_clicked(move || {
        let code = action_primary.lock().map(|value| *value).unwrap_or(0);

        if let Some(ui) = weak.upgrade() {
            ui.set_dialog_visible(false);
        }

        match code {
            1 => {
                let _ = shell::open("ms-windows-store://pdp/?ProductId=9NBLGGH2JHXJ");
            }
            2 => {
                let _ = shell::open("ms-windows-store://pdp/?ProductId=9MWPM2CQNLHN");
            }
            3 => {
                if let Some(ui) = weak.upgrade() {
                    ui.set_settings_open(true);
                }
            }
            _ => {}
        }
    });

    let weak = ui.as_weak();
    ui.on_dialog_secondary_clicked(move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_dialog_visible(false);
        }
    });
}

fn launch_flow(ui: Ui, config: Arc<Mutex<Config>>) {
    ui.busy(true);
    ui.status("Verificando...");
    ui.update(|ui| ui.set_dialog_visible(false));

    let config_snapshot = match config.lock() {
        Ok(config) => config.clone(),
        Err(_) => {
            ui.busy(false);
            ui.status("Erro de configuração");
            return;
        }
    };

    if !minecraft::gaming_services_installed() {
        ui.busy(false);
        ui.status("Gaming Services ausente");
        ui.dialog(
            "Gaming Services necessário",
            "O Minecraft precisa do Gaming Services para abrir. Instale pela Microsoft Store.",
            "Abrir Store",
            "Fechar",
            2,
        );
        return;
    }

    match minecraft::status(&config_snapshot.minecraft) {
        Ok(status) if !status.installed => {
            ui.busy(false);
            ui.status("Minecraft não encontrado");
            ui.dialog(
                "Minecraft não encontrado",
                "Instale o Minecraft Bedrock pela Microsoft Store para continuar.",
                "Abrir Store",
                "Fechar",
                1,
            );
            return;
        }
        Ok(_) => {}
        Err(error) => {
            ui.busy(false);
            ui.status("Falha ao verificar o Minecraft");
            ui.dialog(
                "Falha ao verificar o Minecraft",
                &error.to_string(),
                "OK",
                "",
                0,
            );
            return;
        }
    }

    let library = match client::resolve(&config_snapshot.client) {
        Ok(library) => library,
        Err(error) => {
            ui.busy(false);
            ui.status("DLL inválida");
            ui.dialog(
                "DLL do cliente inválida",
                &error.to_string(),
                "Abrir Settings",
                "",
                3,
            );
            return;
        }
    };

    if client::is_injected(&config_snapshot.client) {
        ui.busy(false);
        ui.status("Cliente já injetado");
        ui.dialog(
            "Cliente já injetado",
            "O cliente já está rodando neste Minecraft.",
            "OK",
            "",
            0,
        );
        return;
    }

    ui.status("Abrindo Minecraft...");
    let pid = match minecraft::launch(&config_snapshot.minecraft) {
        Ok(pid) => pid,
        Err(error) => {
            ui.busy(false);
            ui.status("Falha ao abrir o Minecraft");
            ui.dialog(
                "Falha ao abrir o Minecraft",
                &error.to_string(),
                "OK",
                "",
                0,
            );
            return;
        }
    };

    ui.status("Injetando...");
    if let Err(error) = injector::inject(pid, &library) {
        ui.busy(false);
        ui.status("Falha ao injetar");
        ui.dialog("Falha ao injetar", &error.to_string(), "OK", "", 0);
        return;
    }

    let module = pe::file_name(&library);
    if minecraft::wait_until_injected(pid, &config_snapshot.client, &module) {
        ui.status("Pronto!");
        ui.update(|ui| ui.set_injected(true));
    } else {
        ui.status("Injeção não confirmada");
        ui.dialog(
            "Injeção não confirmada",
            "A DLL foi carregada, mas o cliente não respondeu. Verifique o log em %LOCALAPPDATA%\\Lumina\\Launcher.",
            "OK",
            "",
            0,
        );
    }

    ui.busy(false);
}

pub fn start_polling(ui: &AppWindow, config: Arc<Mutex<Config>>) {
    let ui = Ui::new(ui.as_weak(), Arc::new(Mutex::new(0)));
    let config = config.clone();

    std::thread::spawn(move || {
        loop {
            let config_snapshot = match config.lock() {
                Ok(config) => config.clone(),
                Err(_) => break,
            };

            let status = minecraft::status(&config_snapshot.minecraft);
            let injected = client::is_injected(&config_snapshot.client);
            let dll_valid = config_snapshot
                .client
                .custom_dll
                .as_ref()
                .is_some_and(|path| path.is_file());

            let (installed, version, supported) = match status {
                Ok(status) => {
                    let version = status.version.unwrap_or_else(|| "0.0.0".into());
                    let supported = config_snapshot.is_version_supported(&version);
                    (status.installed, version, supported)
                }
                Err(_) => (false, "0.0.0".to_string(), false),
            };

            ui.update(move |ui| {
                ui.set_game_installed(installed);
                ui.set_game_version(version.into());
                ui.set_game_supported(supported);
                ui.set_injected(injected);
                ui.set_dll_valid(dll_valid);
            });

            std::thread::sleep(Duration::from_secs(2));
        }
    });
}
