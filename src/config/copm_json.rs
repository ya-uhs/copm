use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::CopmError;

fn default_tools() -> Vec<String> {
    vec!["copilot".to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopmJson {
    #[serde(default = "default_tools")]
    pub tools: Vec<String>,

    #[serde(default)]
    pub dependencies: BTreeMap<String, Dependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub source: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_path: Option<String>,
}

impl Default for CopmJson {
    fn default() -> Self {
        Self {
            tools: default_tools(),
            dependencies: BTreeMap::new(),
        }
    }
}

impl CopmJson {
    pub fn load(path: &Path) -> Result<Self, CopmError> {
        if !path.exists() {
            return Err(CopmError::CopmJsonNotFound(path.to_path_buf()));
        }
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn load_or_default(path: &Path) -> Self {
        Self::load(path).unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), CopmError> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content + "\n")?;
        Ok(())
    }

    pub fn add_dependency(&mut self, name: &str, source: &str, version: &str, sub_path: Option<String>) {
        self.dependencies.insert(
            name.to_string(),
            Dependency {
                source: source.to_string(),
                version: version.to_string(),
                sub_path,
            },
        );
    }

    pub fn remove_dependency(&mut self, name: &str) -> bool {
        self.dependencies.remove(name).is_some()
    }
}
