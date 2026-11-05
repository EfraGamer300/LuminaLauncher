use crate::config::{ClientConfig, ClientMode};
use crate::pe::{self, LibraryInfo};
use crate::win::{single_instance, window};
use crate::{Error, Result};

pub fn resolve(config: &ClientConfig) -> Result<LibraryInfo> {
    match config.mode {
        ClientMode::Custom => {
            let path = config
                .custom_dll
                .clone()
                .ok_or_else(|| Error::InvalidLibrary("nenhuma DLL configurada".into()))?;
            pe::inspect(&path)
        }
        ClientMode::Release | ClientMode::Beta => Err(Error::Message(
            "canais remotos ainda não estão disponíveis nesta versão".into(),
        )),
    }
}

pub fn is_injected(config: &ClientConfig) -> bool {
    single_instance::exists(&config.client_mutex)
        || window::find_window(&config.client_window_class, None).is_some()
}
