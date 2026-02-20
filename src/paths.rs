use std::path::PathBuf;

use crate::error::CopmError;

fn home() -> Result<PathBuf, CopmError> {
    dirs::home_dir().ok_or_else(|| {
        CopmError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine home directory",
        ))
    })
}

// ── Legacy paths (kept for backward compatibility) ───────────────────────────

/// Global plugin install directory: ~/.claude/plugins/copm-packages/{name}/
pub fn global_plugin_dir(name: &str) -> Result<PathBuf, CopmError> {
    Ok(home()?.join(".claude").join("plugins").join("copm-packages").join(name))
}

/// Local plugin install directory: .claude/plugins/{name}/
pub fn local_plugin_dir(name: &str) -> PathBuf {
    PathBuf::from(".claude").join("plugins").join(name)
}

// ── copm config ───────────────────────────────────────────────────────────────

/// Global copm config directory: ~/.copm/
pub fn global_copm_dir() -> Result<PathBuf, CopmError> {
    Ok(home()?.join(".copm"))
}

/// Path to copm.json in the current directory
pub fn copm_json_path() -> PathBuf {
    PathBuf::from("copm.json")
}

/// Path to copm.lock in the current directory
pub fn copm_lock_path() -> PathBuf {
    PathBuf::from("copm.lock")
}

// ── Copilot local paths ───────────────────────────────────────────────────────

/// Path to .github/copilot-instructions.md
pub fn copilot_instructions_path() -> PathBuf {
    PathBuf::from(".github").join("copilot-instructions.md")
}

/// Path to .github/instructions/ directory
pub fn copilot_custom_instructions_dir() -> PathBuf {
    PathBuf::from(".github").join("instructions")
}

/// Path to a specific custom instruction file: .github/instructions/{name}.instructions.md
pub fn copilot_custom_instruction_file(name: &str) -> PathBuf {
    copilot_custom_instructions_dir().join(format!("{name}.instructions.md"))
}

/// Path to .github/agents/ directory
pub fn copilot_agents_dir() -> PathBuf {
    PathBuf::from(".github").join("agents")
}

/// Path to .github/prompts/ directory
pub fn copilot_prompts_dir() -> PathBuf {
    PathBuf::from(".github").join("prompts")
}

/// Path to .github/skills/{name}/ directory
pub fn local_copilot_skills_dir(name: &str) -> PathBuf {
    PathBuf::from(".github").join("skills").join(name)
}

// ── Copilot global paths ──────────────────────────────────────────────────────

/// Path to ~/.copilot/instructions/
pub fn global_copilot_instructions_dir() -> Result<PathBuf, CopmError> {
    Ok(home()?.join(".copilot").join("instructions"))
}

/// Path to ~/.copilot/skills/{name}/
pub fn global_copilot_skills_dir(name: &str) -> Result<PathBuf, CopmError> {
    Ok(home()?.join(".copilot").join("skills").join(name))
}

// ── Claude local paths ────────────────────────────────────────────────────────

/// Path to .claude/skills/{name}/
pub fn local_claude_skills_dir(name: &str) -> PathBuf {
    PathBuf::from(".claude").join("skills").join(name)
}

/// Path to .claude/commands/
pub fn local_claude_commands_dir() -> PathBuf {
    PathBuf::from(".claude").join("commands")
}

// ── Claude global paths ───────────────────────────────────────────────────────

/// Path to ~/.claude/skills/{name}/
pub fn global_claude_skills_dir(name: &str) -> Result<PathBuf, CopmError> {
    Ok(home()?.join(".claude").join("skills").join(name))
}

/// Path to ~/.claude/commands/
pub fn global_claude_commands_dir() -> Result<PathBuf, CopmError> {
    Ok(home()?.join(".claude").join("commands"))
}
