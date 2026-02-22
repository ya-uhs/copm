pub mod claude_plugin;
pub mod copilot;

use std::path::{Path, PathBuf};

use crate::error::CopmError;
use crate::manifest::package_manifest::{PackageManifest, Target};
use crate::paths;

/// Install all targets from a manifest, dispatching to the appropriate installer.
///
/// - `name`: The package name (used for skill directory names).
/// - `tools`: The tools configured in copm.json (affects skill install destinations).
/// - `global`: Whether to install globally.
///
/// Returns `(installed_paths, target_types)`.
pub fn install_targets(
    source_dir: &Path,
    manifest: &PackageManifest,
    name: &str,
    tools: &[String],
    global: bool,
) -> Result<(Vec<PathBuf>, Vec<String>), CopmError> {
    let mut all_paths = Vec::new();
    let mut target_types = Vec::new();

    for target in &manifest.targets {
        let paths = install_target(source_dir, target, name, tools, global)?;
        target_types.push(target.target_type.clone());
        all_paths.extend(paths);
    }

    Ok((all_paths, target_types))
}

fn install_target(
    source_dir: &Path,
    target: &Target,
    name: &str,
    tools: &[String],
    global: bool,
) -> Result<Vec<PathBuf>, CopmError> {
    let target_path = if target.path == "." {
        source_dir.to_path_buf()
    } else {
        source_dir.join(&target.path)
    };

    // Single-file install
    if target_path.is_file() {
        return install_single_file(&target_path, &target.target_type, global);
    }

    // Directory install
    match target.target_type.as_str() {
        "claude-plugin" => {
            // Legacy: kept for backward compatibility
            let path = claude_plugin::install_plugin_dir(&target_path, name, global)?;
            Ok(vec![path])
        }
        "copilot-instructions" => copilot::install_instructions(&target_path, global),
        "copilot-custom-instructions" => copilot::install_custom_instructions(&target_path, global),
        "copilot-agents" => copilot::install_agents(&target_path, global),
        "copilot-prompts" => copilot::install_prompts(&target_path, global),
        "skill" => copilot::install_skill(&target_path, name, tools, global),
        "claude-command" => copilot::install_claude_command(&target_path, global),
        other => Err(CopmError::UnsupportedTargetType(other.to_string())),
    }
}

/// Copy a single file to the appropriate destination directory.
fn install_single_file(
    file_path: &Path,
    target_type: &str,
    global: bool,
) -> Result<Vec<PathBuf>, CopmError> {
    let file_name = file_path.file_name().unwrap();

    // copilot-instructions always installs to a fixed path
    if target_type == "copilot-instructions" {
        if global {
            return Ok(vec![]);
        }
        let dest = paths::copilot_instructions_path();
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(file_path, &dest)?;
        return Ok(vec![dest]);
    }

    let dest_dir = match target_type {
        "copilot-custom-instructions" => {
            if global {
                paths::global_copilot_instructions_dir()?
            } else {
                paths::copilot_custom_instructions_dir()
            }
        }
        "copilot-agents" => {
            if global {
                return Ok(vec![]);
            }
            paths::copilot_agents_dir()
        }
        "copilot-prompts" => {
            if global {
                return Ok(vec![]);
            }
            paths::copilot_prompts_dir()
        }
        "claude-command" => {
            if global {
                paths::global_claude_commands_dir()?
            } else {
                paths::local_claude_commands_dir()
            }
        }
        other => return Err(CopmError::UnsupportedTargetType(other.to_string())),
    };

    std::fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join(file_name);
    std::fs::copy(file_path, &dest)?;
    Ok(vec![dest])
}

/// Uninstall a package using its recorded `installed_files` if available,
/// otherwise fall back to type-based removal.
pub fn uninstall_targets(
    name: &str,
    target_types: &[String],
    installed_files: &[String],
    global: bool,
) -> Result<(), CopmError> {
    if !installed_files.is_empty() {
        return copilot::uninstall_by_files(installed_files);
    }

    // Legacy fallback: type-based removal
    for target_type in target_types {
        match target_type.as_str() {
            "claude-plugin" => claude_plugin::uninstall_plugin(name, global)?,
            "copilot-instructions" => copilot::uninstall_instructions()?,
            "copilot-custom-instructions" => copilot::uninstall_custom_instructions(name)?,
            _ => {} // Skip unknown types
        }
    }
    Ok(())
}
