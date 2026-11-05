use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::{Result, logging, paths};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ClientMode {
    #[default]
    Custom,
    Release,
    Beta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClientConfig {
    pub mode: ClientMode,
    pub custom_dll: Option<PathBuf>,
    pub release_url: Option<String>,
    pub release_sha256: Option<String>,
    pub client_window_class: String,
    pub client_mutex: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            mode: ClientMode::Custom,
            custom_dll: Some(paths::client_dir().join("Flarial.Client.dll")),
            release_url: None,
            release_sha256: None,
            client_window_class: "Flarial Client".into(),
            client_mutex: "Flarial".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MinecraftConfig {
    pub package_family: String,
    pub app_id: String,
    pub process_name: String,
    pub window_class: String,
}

impl Default for MinecraftConfig {
    fn default() -> Self {
        Self {
            package_family: "Microsoft.MinecraftUWP_8wekyb3d8bbwe".into(),
            app_id: "Game".into(),
            process_name: "Minecraft.Windows.exe".into(),
            window_class: "Bedrock".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub version: u32,
    pub automatic_updates: bool,
    pub performance_mode: bool,
    pub launcher_version: String,
    pub client: ClientConfig,
    pub minecraft: MinecraftConfig,
    pub supported_versions: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            automatic_updates: true,
            performance_mode: false,
            launcher_version: env!("CARGO_PKG_VERSION").into(),
            client: ClientConfig::default(),
            minecraft: MinecraftConfig::default(),
            supported_versions: Vec::new(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        if !paths::config_path().exists() {
            let config = Self::default();
            if let Err(error) = config.save() {
                logging::warn(&format!("falha ao criar config padrão: {error}"));
            }
            return config;
        }

        match Self::load_from(&paths::config_path()) {
            Ok(config) => config,
            Err(error) => {
                logging::warn(&format!("falha ao carregar config: {error}"));
                Self::default()
            }
        }
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&text)?)
    }

    pub fn save(&self) -> Result<()> {
        paths::ensure_dirs()?;
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(paths::config_path(), text)?;
        Ok(())
    }

    pub fn is_version_supported(&self, version: &str) -> bool {
        if self.supported_versions.is_empty() {
            return true;
        }
        self.supported_versions
            .iter()
            .any(|supported| version.starts_with(supported))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut config = Config {
            performance_mode: true,
            ..Default::default()
        };
        config.client.custom_dll = Some(PathBuf::from(r"C:\test\client.dll"));

        let text = serde_json::to_string_pretty(&config).unwrap();
        let parsed: Config = serde_json::from_str(&text).unwrap();

        assert!(parsed.performance_mode);
        assert_eq!(
            parsed.client.custom_dll,
            Some(PathBuf::from(r"C:\test\client.dll"))
        );
        assert_eq!(parsed.client.mode, ClientMode::Custom);
    }

    #[test]
    fn empty_supported_list_accepts_everything() {
        let config = Config::default();
        assert!(config.is_version_supported("26.60"));
    }

    #[test]
    fn supported_list_filters_versions() {
        let config = Config {
            supported_versions: vec!["26.60".into()],
            ..Default::default()
        };
        assert!(config.is_version_supported("26.60"));
        assert!(!config.is_version_supported("25.10"));
    }
}
