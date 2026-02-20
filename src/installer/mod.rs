pub mod claude_plugin;
pub mod copilot;

use std::path::{Path, PathBuf};

use crate::error::CopmError;
use crate::manifest::package_manifest::{PackageManifest, Target};

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
    let target_dir = if target.path == "." {
        source_dir.to_path_buf()
    } else {
        source_dir.join(&target.path)
    };

    match target.target_type.as_str() {
        "claude-plugin" => {
            // Legacy: kept for backward compatibility
            let path = claude_plugin::install_plugin_dir(&target_dir, name, global)?;
            Ok(vec![path])
        }
        "copilot-instructions" => copilot::install_instructions(&target_dir, global),
        "copilot-custom-instructions" => copilot::install_custom_instructions(&target_dir, global),
        "copilot-agents" => copilot::install_agents(&target_dir, global),
        "copilot-prompts" => copilot::install_prompts(&target_dir, global),
        "skill" => copilot::install_skill(&target_dir, name, tools, global),
        "claude-command" => copilot::install_claude_command(&target_dir, global),
        other => Err(CopmError::UnsupportedTargetType(other.to_string())),
    }
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
