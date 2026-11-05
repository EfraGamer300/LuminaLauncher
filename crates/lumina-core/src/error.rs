use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("erro do Windows: {0}")]
    Win32(#[from] windows::core::Error),

    #[error("erro de I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("erro de JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Minecraft não está instalado")]
    MinecraftNotInstalled,

    #[error("Gaming Services não está instalado")]
    GamingServicesNotInstalled,

    #[error("instalação sideloaded não é suportada nesta versão")]
    SideloadedUnsupported,

    #[error("o Minecraft não abriu a tempo")]
    LaunchTimeout,

    #[error("DLL inválida: {0}")]
    InvalidLibrary(String),

    #[error("falha ao injetar a DLL")]
    InjectionFailed,

    #[error("o cliente já está injetado")]
    ClientAlreadyInjected,

    #[error("{0}")]
    Message(String),
}
