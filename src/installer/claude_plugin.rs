use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::error::CopmError;
use crate::paths;

/// Install a Claude plugin from `plugin_dir` to the appropriate install directory.
/// Returns the install path.
pub fn install_plugin_dir(
    plugin_dir: &Path,
    name: &str,
    global: bool,
) -> Result<PathBuf, CopmError> {
    let install_dir = if global {
        paths::global_plugin_dir(name)?
    } else {
        paths::local_plugin_dir(name)
    };

    if install_dir.exists() {
        std::fs::remove_dir_all(&install_dir)?;
    }
    if let Some(parent) = install_dir.parent() {
        std::fs::create_dir_all(parent)?;
    }

    copy_dir_recursive(plugin_dir, &install_dir)?;
    Ok(install_dir)
}

/// Remove an installed plugin
pub fn uninstall_plugin(name: &str, global: bool) -> Result<(), CopmError> {
    let install_dir = if global {
        paths::global_plugin_dir(name)?
    } else {
        paths::local_plugin_dir(name)
    };

    if !install_dir.exists() {
        return Err(CopmError::NotInstalled(name.to_string()));
    }

    std::fs::remove_dir_all(&install_dir)?;
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), CopmError> {
    std::fs::create_dir_all(dst)?;
    for entry in WalkDir::new(src).min_depth(1) {
        let entry = entry.map_err(|e| {
            CopmError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;
        let relative = entry.path().strip_prefix(src).unwrap();
        let dest_path = dst.join(relative);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest_path)?;
        } else {
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}
