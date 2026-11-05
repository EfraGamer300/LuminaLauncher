use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::{Error, Result};

const IMAGE_FILE_DLL: u16 = 0x2000;
const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;

#[derive(Debug, Clone)]
pub struct LibraryInfo {
    pub path: PathBuf,
    pub imports: Vec<String>,
}

pub fn inspect(path: &Path) -> Result<LibraryInfo> {
    if !path.is_file() {
        return Err(Error::InvalidLibrary(format!(
            "arquivo não encontrado: {}",
            path.display()
        )));
    }

    let bytes = std::fs::read(path)?;
    let pe = goblin::pe::PE::parse(&bytes)
        .map_err(|error| Error::InvalidLibrary(format!("PE inválido: {error}")))?;

    let machine = pe.header.coff_header.machine;
    if machine != IMAGE_FILE_MACHINE_AMD64 {
        return Err(Error::InvalidLibrary(
            "a DLL precisa ser 64-bit (x86_64)".into(),
        ));
    }

    if pe.header.coff_header.characteristics & IMAGE_FILE_DLL == 0 {
        return Err(Error::InvalidLibrary(
            "o arquivo não é uma DLL (IMAGE_FILE_DLL ausente)".into(),
        ));
    }

    let system32 = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
        .join("System32");

    let mut seen = HashSet::new();
    let mut imports = Vec::new();

    for library in &pe.libraries {
        if library.is_empty() {
            continue;
        }

        let key = library.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }

        let candidate = system32.join(library);
        if candidate.is_file() {
            imports.push(candidate.to_string_lossy().to_string());
        } else if library.contains(['\\', '/']) && Path::new(library).is_file() {
            imports.push(library.to_string());
        } else {
            imports.push((*library).to_string());
        }
    }

    Ok(LibraryInfo {
        path: std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()),
        imports,
    })
}

pub fn file_name(library: &LibraryInfo) -> String {
    library
        .path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn system32(name: &str) -> PathBuf {
        std::env::var_os("SystemRoot")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
            .join("System32")
            .join(name)
    }

    #[test]
    fn reads_a_system_dll() {
        let path = system32("version.dll");
        if !path.is_file() {
            return;
        }

        let info = inspect(&path).unwrap();
        assert!(info.path.is_absolute());
        assert_eq!(file_name(&info), "version.dll");
        assert!(!info.imports.is_empty());
    }

    #[test]
    fn rejects_executables() {
        let path = system32("notepad.exe");
        if !path.is_file() {
            return;
        }

        let error = inspect(&path).unwrap_err();
        assert!(matches!(error, Error::InvalidLibrary(_)));
    }

    #[test]
    fn rejects_missing_files() {
        let error = inspect(Path::new(r"C:\this\does\not\exist.dll")).unwrap_err();
        assert!(matches!(error, Error::InvalidLibrary(_)));
    }
}
