use crate::error::CopmError;
use crate::installer::copilot;
use crate::paths;

pub fn run(global: bool) -> Result<(), CopmError> {
    let scope = if global { "global" } else { "local" };
    let mut found_any = false;

    if !global {
        // copilot-instructions
        let instructions_path = paths::copilot_instructions_path();
        if instructions_path.exists() {
            println!("[copilot-instructions]  {}", instructions_path.display());
            found_any = true;
        }

        // copilot-custom-instructions
        let custom = copilot::list_custom_instructions()?;
        if !custom.is_empty() {
            println!("[copilot-custom-instructions]  {}:", paths::copilot_custom_instructions_dir().display());
            for name in &custom {
                println!("  {name}");
            }
            found_any = true;
        }

        // copilot-agents
        let agents = copilot::list_agents()?;
        if !agents.is_empty() {
            println!("[copilot-agents]  {}:", paths::copilot_agents_dir().display());
            for name in &agents {
                println!("  {name}");
            }
            found_any = true;
        }

        // copilot-prompts
        let prompts = copilot::list_prompts()?;
        if !prompts.is_empty() {
            println!("[copilot-prompts]  {}:", paths::copilot_prompts_dir().display());
            for name in &prompts {
                println!("  {name}");
            }
            found_any = true;
        }

        // copilot skills
        let copilot_skills_base = std::path::PathBuf::from(".github").join("skills");
        if copilot_skills_base.is_dir() {
            let mut skill_names: Vec<String> = std::fs::read_dir(&copilot_skills_base)?
                .flatten()
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            if !skill_names.is_empty() {
                skill_names.sort();
                println!("[skill → copilot]  {}:", copilot_skills_base.display());
                for name in &skill_names {
                    println!("  {name}/");
                }
                found_any = true;
            }
        }

        // claude skills
        let claude_skills_base = std::path::PathBuf::from(".claude").join("skills");
        if claude_skills_base.is_dir() {
            let mut skill_names: Vec<String> = std::fs::read_dir(&claude_skills_base)?
                .flatten()
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            if !skill_names.is_empty() {
                skill_names.sort();
                println!("[skill → claude]  {}:", claude_skills_base.display());
                for name in &skill_names {
                    println!("  {name}/");
                }
                found_any = true;
            }
        }

        // claude commands
        let commands = copilot::list_claude_commands(false)?;
        if !commands.is_empty() {
            println!("[claude-command]  {}:", paths::local_claude_commands_dir().display());
            for name in &commands {
                println!("  {name}");
            }
            found_any = true;
        }
    } else {
        // Global: copilot skills
        if let Ok(copilot_skills_base) = paths::global_copilot_skills_dir("").map(|p| p.parent().unwrap().to_path_buf()) {
            if copilot_skills_base.is_dir() {
                let mut skill_names: Vec<String> = std::fs::read_dir(&copilot_skills_base)?
                    .flatten()
                    .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect();
                if !skill_names.is_empty() {
                    skill_names.sort();
                    println!("[skill → copilot]  {}:", copilot_skills_base.display());
                    for name in &skill_names {
                        println!("  {name}/");
                    }
                    found_any = true;
                }
            }
        }

        // Global: claude skills
        if let Ok(claude_skills_base) = paths::global_claude_skills_dir("").map(|p| p.parent().unwrap().to_path_buf()) {
            if claude_skills_base.is_dir() {
                let mut skill_names: Vec<String> = std::fs::read_dir(&claude_skills_base)?
                    .flatten()
                    .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect();
                if !skill_names.is_empty() {
                    skill_names.sort();
                    println!("[skill → claude]  {}:", claude_skills_base.display());
                    for name in &skill_names {
                        println!("  {name}/");
                    }
                    found_any = true;
                }
            }
        }

        // Global: copilot custom instructions
        if let Ok(global_instr_dir) = paths::global_copilot_instructions_dir() {
            if global_instr_dir.is_dir() {
                let mut names: Vec<String> = std::fs::read_dir(&global_instr_dir)?
                    .flatten()
                    .filter(|e| e.file_name().to_string_lossy().ends_with(".instructions.md"))
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect();
                if !names.is_empty() {
                    names.sort();
                    println!("[copilot-custom-instructions]  {}:", global_instr_dir.display());
                    for name in &names {
                        println!("  {name}");
                    }
                    found_any = true;
                }
            }
        }

        // Global: claude commands
        let commands = copilot::list_claude_commands(true)?;
        if !commands.is_empty() {
            if let Ok(dir) = paths::global_claude_commands_dir() {
                println!("[claude-command]  {}:", dir.display());
            }
            for name in &commands {
                println!("  {name}");
            }
            found_any = true;
        }
    }

    if !found_any {
        println!("No {scope} packages installed.");
    }

    Ok(())
}
