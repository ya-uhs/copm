use std::path::Path;

use crate::error::CopmError;

#[derive(Debug, Clone)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    pub targets: Vec<Target>,
}

#[derive(Debug, Clone)]
pub struct Target {
    pub target_type: String,
    /// Path relative to extracted_dir
    pub path: String,
}

impl PackageManifest {
    /// Detect what targets are available in a fetched package directory.
    ///
    /// - `dir`: The root of the extracted package.
    /// - `sub_path`: Optional sub-path within the package to scope detection to.
    /// - `source`: The original package specifier (used in error messages).
    pub fn detect_from_dir(
        dir: &Path,
        sub_path: Option<&str>,
        source: &str,
    ) -> Result<Self, CopmError> {
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        if let Some(sp) = sub_path {
            // Scoped to a specific sub-directory: return exactly one target
            let scoped = dir.join(sp);
            if !scoped.exists() {
                return Err(CopmError::NoTargetsDetected(format!(
                    "{source}:{sp} does not exist"
                )));
            }
            let target_type = classify_dir(&scoped).ok_or_else(|| {
                CopmError::NoTargetsDetected(format!(
                    "No recognizable content found in {source}:{sp}"
                ))
            })?;
            return Ok(Self {
                name,
                version: "0.0.0".to_string(),
                targets: vec![Target {
                    target_type,
                    path: sp.to_string(),
                }],
            });
        }

        // Scan the full root
        let candidates = scan_root(dir);

        match candidates.len() {
            0 => Err(CopmError::NoTargetsDetected(format!(
                "No recognizable targets found in {source}\nTry specifying a sub-path: copm install {source}:<subpath>"
            ))),
            1 => {
                let (path, target_type) = candidates.into_iter().next().unwrap();
                Ok(Self {
                    name,
                    version: "0.0.0".to_string(),
                    targets: vec![Target { target_type, path }],
                })
            }
            _ => {
                let target_list = candidates
                    .iter()
                    .map(|(path, t)| format!("  {path:<20} ({t})"))
                    .collect::<Vec<_>>()
                    .join("\n");
                Err(CopmError::AmbiguousTargets {
                    pkg: source.to_string(),
                    targets: target_list,
                })
            }
        }
    }
}

/// Scan a package root for detectable targets.
/// Returns Vec<(relative_path, target_type)> sorted by path.
fn scan_root(root: &Path) -> Vec<(String, String)> {
    let mut results = Vec::new();

    // Check the root itself first
    if let Some(t) = classify_dir(root) {
        results.push((".".to_string(), t));
        return results;
    }

    // Check immediate subdirectories
    if let Ok(entries) = std::fs::read_dir(root) {
        let mut subdir_results: Vec<(String, String)> = entries
            .flatten()
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                classify_dir(&e.path()).map(|t| (name, t))
            })
            .collect();
        subdir_results.sort_by(|a, b| a.0.cmp(&b.0));
        results.extend(subdir_results);
    }

    results
}

/// Determine the target type of a directory by examining its contents.
/// Returns None if nothing recognizable is found.
fn classify_dir(dir: &Path) -> Option<String> {
    // SKILL.md at this level → single skill
    if dir.join("SKILL.md").exists() {
        return Some("skill".to_string());
    }

    // copilot-instructions.md at this level
    if dir.join("copilot-instructions.md").exists() {
        return Some("copilot-instructions".to_string());
    }

    let mut has_instructions = false;
    let mut has_agents = false;
    let mut has_prompts = false;
    let mut has_skill_subdirs = false;

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };
            let name = entry.file_name().to_string_lossy().to_string();

            if file_type.is_file() {
                if name.ends_with(".instructions.md") {
                    has_instructions = true;
                } else if name.ends_with(".agent.md") {
                    has_agents = true;
                } else if name.ends_with(".prompt.md") {
                    has_prompts = true;
                }
            } else if file_type.is_dir() && entry.path().join("SKILL.md").exists() {
                has_skill_subdirs = true;
            }
        }
    }

    if has_agents {
        Some("copilot-agents".to_string())
    } else if has_prompts {
        Some("copilot-prompts".to_string())
    } else if has_instructions {
        Some("copilot-custom-instructions".to_string())
    } else if has_skill_subdirs {
        Some("skill".to_string())
    } else {
        None
    }
}
