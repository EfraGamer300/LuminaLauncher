use std::path::PathBuf;

use windows::ApplicationModel::{Package, PackageSignatureKind};
use windows::Management::Deployment::PackageManager;
use windows::core::HSTRING;

use crate::Result;
use crate::win::com;

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub version: String,
    pub installed_path: PathBuf,
    pub is_store: bool,
}

fn find(family: &str) -> Result<Option<Package>> {
    com::initialize_mta()?;
    let manager = PackageManager::new()?;
    let packages = manager
        .FindPackagesByUserSecurityIdPackageFamilyName(&HSTRING::new(), &HSTRING::from(family))?;
    let iterator = packages.First()?;

    if iterator.HasCurrent()? {
        Ok(Some(iterator.Current()?))
    } else {
        Ok(None)
    }
}

pub fn find_info(family: &str) -> Result<Option<PackageInfo>> {
    let Some(package) = find(family)? else {
        return Ok(None);
    };

    let version = package.Id()?.Version()?;
    let installed_path = PathBuf::from(package.InstalledPath()?.to_string());
    let is_store = package.SignatureKind()? == PackageSignatureKind::Store;

    Ok(Some(PackageInfo {
        version: format_version(version.Major, version.Minor, version.Build),
        installed_path,
        is_store,
    }))
}

pub fn is_installed(family: &str) -> bool {
    find_info(family).ok().flatten().is_some()
}

pub fn format_version(major: u16, minor: u16, build: u16) -> String {
    let build = build / 100;
    if minor >= 26 {
        format!("{minor}.{build}")
    } else {
        format!("{major}.{minor}.{build}")
    }
}

#[cfg(test)]
mod tests {
    use super::format_version;

    #[test]
    fn new_scheme_uses_minor_build() {
        assert_eq!(format_version(1, 26, 6005), "26.60");
    }

    #[test]
    fn old_scheme_uses_major_minor_build() {
        assert_eq!(format_version(1, 21, 2000), "1.21.20");
    }
}
