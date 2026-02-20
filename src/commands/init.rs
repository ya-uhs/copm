use std::io::{self, Write};

use crate::config::copm_json::CopmJson;
use crate::error::CopmError;
use crate::paths;

pub fn run() -> Result<(), CopmError> {
    let path = paths::copm_json_path();
    if path.exists() {
        return Err(CopmError::CopmJsonAlreadyExists);
    }

    let tools = prompt_tools()?;

    let mut config = CopmJson::default();
    config.tools = tools;
    config.save(&path)?;
    println!("Created copm.json");
    Ok(())
}

fn prompt_tools() -> Result<Vec<String>, CopmError> {
    print!("Which tools do you use? [copilot/claude/both] (default: copilot): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    let tools = match input.as_str() {
        "claude" => vec!["claude".to_string()],
        "both" => vec!["copilot".to_string(), "claude".to_string()],
        _ => vec!["copilot".to_string()],
    };

    println!(
        "Tools: {}",
        tools.join(", ")
    );

    Ok(tools)
}
