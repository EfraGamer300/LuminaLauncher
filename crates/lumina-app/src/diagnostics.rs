use lumina_core::config::Config;
use lumina_core::{client, minecraft, paths};

pub fn report(config: &Config) -> String {
    let mut output = String::new();

    output.push_str(&format!("Lumina Launcher {}\n", env!("CARGO_PKG_VERSION")));
    output.push_str(&format!("config: {}\n", paths::config_path().display()));
    output.push_str(&format!(
        "launcher dir: {}\n",
        paths::launcher_dir().display()
    ));
    output.push_str(&format!(
        "client dir: {}\n\n",
        paths::client_dir().display()
    ));

    output.push_str("--- config ---\n");
    output.push_str(&format!("{config:#?}\n\n"));

    output.push_str(&format!(
        "gaming services: {}\n",
        minecraft::gaming_services_installed()
    ));

    match minecraft::status(&config.minecraft) {
        Ok(status) => {
            output.push_str(&format!(
                "minecraft: installed={} version={:?} store={} running={} pid={:?} window_visible={}\n",
                status.installed,
                status.version,
                status.is_store,
                status.running,
                status.pid,
                status.window_visible
            ));
            if let Some(path) = status.installed_path {
                output.push_str(&format!("minecraft path: {}\n", path.display()));
            }
        }
        Err(error) => output.push_str(&format!("minecraft: erro: {error}\n")),
    }

    match client::resolve(&config.client) {
        Ok(library) => {
            output.push_str(&format!(
                "client: {} ({} imports)\n",
                library.path.display(),
                library.imports.len()
            ));
            for import in &library.imports {
                output.push_str(&format!("  import: {import}\n"));
            }
        }
        Err(error) => output.push_str(&format!("client: erro: {error}\n")),
    }

    output.push_str(&format!(
        "client injected: {}\n",
        client::is_injected(&config.client)
    ));

    output
}
