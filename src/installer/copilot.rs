use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::error::CopmError;
use crate::paths;

// ── copilot-instructions ──────────────────────────────────────────────────────

/// Install copilot-instructions from a source dir.
/// The source_dir should contain a `copilot-instructions.md` file.
/// Copies to `.github/copilot-instructions.md` (global not supported for this type).
pub fn install_instructions(
    source_dir: &Path,
    global: bool,
) -> Result<Vec<PathBuf>, CopmError> {
    if global {
        // Global copilot-instructions is not a defined location; skip
        return Ok(vec![]);
    }

    // Look for copilot-instructions.md at source root, then any single .md
    let candidate = source_dir.join("copilot-instructions.md");
    let source_file = if candidate.exists() {
        candidate
    } else {
        // Try single .md file in directory
        let mut md_files: Vec<PathBuf> = std::fs::read_dir(source_dir)
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|e| {
                        e.file_type().map(|t| t.is_file()).unwrap_or(false)
                            && e.file_name().to_string_lossy().ends_with(".md")
                    })
                    .map(|e| e.path())
                    .collect()
            })
            .unwrap_or_default();
        if md_files.len() == 1 {
            md_files.remove(0)
        } else {
            return Err(CopmError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No copilot-instructions.md found in {}", source_dir.display()),
            )));
        }
    };

    let dest = paths::copilot_instructions_path();
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(&source_file, &dest)?;
    Ok(vec![dest])
}

pub fn uninstall_instructions() -> Result<(), CopmError> {
    let path = paths::copilot_instructions_path();
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

// ── File collection installer (agents, prompts, custom-instructions) ──────────

/// Install a collection of files matching `suffix` from `source_dir` to `dest_dir`.
pub fn install_file_collection(
    source_dir: &Path,
    suffix: &str,
    dest_dir: &PathBuf,
) -> Result<Vec<PathBuf>, CopmError> {
    std::fs::create_dir_all(dest_dir)?;
    let mut installed = Vec::new();

    for entry in std::fs::read_dir(source_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if entry.file_type()?.is_file() && name_str.ends_with(suffix) {
            let dest_file = dest_dir.join(&name);
            std::fs::copy(entry.path(), &dest_file)?;
            installed.push(dest_file);
        }
    }

    Ok(installed)
}

// ── copilot-custom-instructions ───────────────────────────────────────────────

pub fn install_custom_instructions(
    source_dir: &Path,
    global: bool,
) -> Result<Vec<PathBuf>, CopmError> {
    let dest_dir = if global {
        paths::global_copilot_instructions_dir()?
    } else {
        paths::copilot_custom_instructions_dir()
    };
    install_file_collection(source_dir, ".instructions.md", &dest_dir)
}

pub fn uninstall_custom_instructions(name: &str) -> Result<(), CopmError> {
    let file = paths::copilot_custom_instruction_file(name);
    if file.exists() {
        std::fs::remove_file(&file)?;
        return Ok(());
    }

    let dir = paths::copilot_custom_instructions_dir();
    if dir.exists() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let fname = entry.file_name();
            let fname_str = fname.to_string_lossy();
            if fname_str.starts_with(name) && fname_str.ends_with(".instructions.md") {
                std::fs::remove_file(entry.path())?;
            }
        }
    }
    Ok(())
}

pub fn list_custom_instructions() -> Result<Vec<String>, CopmError> {
    let dir = paths::copilot_custom_instructions_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.ends_with(".instructions.md") {
            names.push(fname);
        }
    }
    names.sort();
    Ok(names)
}

// ── copilot-agents ────────────────────────────────────────────────────────────

pub fn install_agents(source_dir: &Path, global: bool) -> Result<Vec<PathBuf>, CopmError> {
    // Global agent path is not standardized yet; use local only for now
    if global {
        return Ok(vec![]);
    }
    install_file_collection(source_dir, ".agent.md", &paths::copilot_agents_dir())
}

pub fn list_agents() -> Result<Vec<String>, CopmError> {
    let dir = paths::copilot_agents_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.ends_with(".agent.md") {
            names.push(fname);
        }
    }
    names.sort();
    Ok(names)
}

// ── copilot-prompts ───────────────────────────────────────────────────────────

pub fn install_prompts(source_dir: &Path, global: bool) -> Result<Vec<PathBuf>, CopmError> {
    if global {
        return Ok(vec![]);
    }
    install_file_collection(source_dir, ".prompt.md", &paths::copilot_prompts_dir())
}

pub fn list_prompts() -> Result<Vec<String>, CopmError> {
    let dir = paths::copilot_prompts_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.ends_with(".prompt.md") {
            names.push(fname);
        }
    }
    names.sort();
    Ok(names)
}

// ── skill ─────────────────────────────────────────────────────────────────────

/// Install a skill or skill collection.
///
/// If `source_dir` contains `SKILL.md` directly, it is installed as a single skill
/// named `skill_name`.  If it contains sub-directories that each have `SKILL.md`,
/// each sub-directory is installed as a separately named skill.
///
/// Destinations are determined by `tools`:
///   - "copilot" → `.github/skills/<name>/` or `~/.copilot/skills/<name>/`
///   - "claude"  → `.claude/skills/<name>/` or `~/.claude/skills/<name>/`
pub fn install_skill(
    source_dir: &Path,
    skill_name: &str,
    tools: &[String],
    global: bool,
) -> Result<Vec<PathBuf>, CopmError> {
    let mut installed = Vec::new();

    if source_dir.join("SKILL.md").exists() {
        // Single skill
        install_single_skill(source_dir, skill_name, tools, global, &mut installed)?;
    } else {
        // Collection: install each subdir that contains SKILL.md
        if let Ok(entries) = std::fs::read_dir(source_dir) {
            let mut subdirs: Vec<_> = entries
                .flatten()
                .filter(|e| {
                    e.file_type().map(|t| t.is_dir()).unwrap_or(false)
                        && e.path().join("SKILL.md").exists()
                })
                .collect();
            subdirs.sort_by_key(|e| e.file_name());

            for entry in subdirs {
                let name = entry.file_name().to_string_lossy().to_string();
                install_single_skill(&entry.path(), &name, tools, global, &mut installed)?;
            }
        }
    }

    Ok(installed)
}

fn install_single_skill(
    skill_dir: &Path,
    name: &str,
    tools: &[String],
    global: bool,
    installed: &mut Vec<PathBuf>,
) -> Result<(), CopmError> {
    for tool in tools {
        let dest = match tool.as_str() {
            "copilot" => {
                if global {
                    paths::global_copilot_skills_dir(name)?
                } else {
                    paths::local_copilot_skills_dir(name)
                }
            }
            "claude" => {
                if global {
                    paths::global_claude_skills_dir(name)?
                } else {
                    paths::local_claude_skills_dir(name)
                }
            }
            _ => continue,
        };

        if dest.exists() {
            std::fs::remove_dir_all(&dest)?;
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        copy_dir_recursive(skill_dir, &dest)?;
        installed.push(dest);
    }
    Ok(())
}

// ── claude-command ────────────────────────────────────────────────────────────

pub fn install_claude_command(
    source_dir: &Path,
    global: bool,
) -> Result<Vec<PathBuf>, CopmError> {
    let dest_dir = if global {
        paths::global_claude_commands_dir()?
    } else {
        paths::local_claude_commands_dir()
    };
    install_file_collection(source_dir, ".md", &dest_dir)
}

pub fn list_claude_commands(global: bool) -> Result<Vec<String>, CopmError> {
    let dir = if global {
        paths::global_claude_commands_dir()?
    } else {
        paths::local_claude_commands_dir()
    };
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.ends_with(".md") {
            names.push(fname);
        }
    }
    names.sort();
    Ok(names)
}

// ── File-based uninstall ──────────────────────────────────────────────────────

/// Remove all paths listed in `files` (supports both files and directories).
pub fn uninstall_by_files(files: &[String]) -> Result<(), CopmError> {
    for file in files {
        let path = PathBuf::from(file);
        if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
        } else if path.exists() {
            std::fs::remove_file(&path)?;
        }
    }
    Ok(())
}

// ── Utilities ─────────────────────────────────────────────────────────────────

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
