use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::CopmError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopmLock {
    pub version: u32,
    pub packages: Vec<LockedPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub source: LockedSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrity: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<String>,
    /// Files/dirs installed on disk (used for accurate uninstall)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub installed_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub repo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_path: Option<String>,
}

impl Default for CopmLock {
    fn default() -> Self {
        Self {
            version: 1,
            packages: Vec::new(),
        }
    }
}

impl CopmLock {
    pub fn load(path: &Path) -> Result<Self, CopmError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let lock: Self = serde_json::from_str(&content)?;
        Ok(lock)
    }

    pub fn save(&self, path: &Path) -> Result<(), CopmError> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content + "\n")?;
        Ok(())
    }

    pub fn upsert_package(&mut self, pkg: LockedPackage) {
        if let Some(existing) = self.packages.iter_mut().find(|p| p.name == pkg.name) {
            *existing = pkg;
        } else {
            self.packages.push(pkg);
        }
    }

    pub fn remove_package(&mut self, name: &str) -> bool {
        let len_before = self.packages.len();
        self.packages.retain(|p| p.name != name);
        self.packages.len() < len_before
    }
}
