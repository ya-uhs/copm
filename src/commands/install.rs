use crate::config::copm_json::CopmJson;
use crate::config::lock::{CopmLock, LockedPackage, LockedSource};
use crate::error::CopmError;
use crate::fetcher::git::{fetch_package, parse_package_spec};
use crate::installer;
use crate::manifest::package_manifest::PackageManifest;
use crate::paths;

/// Install a single package by specifier (e.g., "owner/repo" or "owner/repo:subpath")
pub async fn run(package: &str, global: bool) -> Result<(), CopmError> {
    let (user, repo, sub_path) = parse_package_spec(package)?;
    println!("Fetching {user}/{repo}...");

    // Load tools config (default to copilot if no copm.json)
    let copm_json_path = paths::copm_json_path();
    let config = CopmJson::load_or_default(&copm_json_path);
    let tools = &config.tools;

    // Download to temp directory
    let tmp_dir = tempfile::tempdir()?;
    let result = fetch_package(&user, &repo, tmp_dir.path()).await?;

    // Detect manifest
    let source_label = format!("{user}/{repo}");
    let manifest = PackageManifest::detect_from_dir(
        &result.extracted_dir,
        sub_path.as_deref(),
        &source_label,
    )?;

    // Derive a clean package name from repo + optional sub_path
    let pkg_name = package_name(&repo, sub_path.as_deref());
    println!("Detected: {} ({} target(s))", pkg_name, manifest.targets.len());
    for t in &manifest.targets {
        println!("  [{}] path={}", t.target_type, t.path);
    }

    // Install all targets
    let (installed_paths, target_types) =
        installer::install_targets(&result.extracted_dir, &manifest, &pkg_name, tools, global)?;

    let installed_files: Vec<String> = installed_paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    for path in &installed_paths {
        println!("  → {}", path.display());
    }
    println!("Installed {pkg_name}");

    // Update copm.json and copm.lock (only for local installs with existing copm.json)
    if !global && copm_json_path.exists() {
        let mut config = CopmJson::load(&copm_json_path)?;
        config.add_dependency(&pkg_name, &source_label, &manifest.version, sub_path.clone());
        config.save(&copm_json_path)?;

        let lock_path = paths::copm_lock_path();
        let mut lock = CopmLock::load(&lock_path)?;
        lock.upsert_package(LockedPackage {
            name: pkg_name.clone(),
            version: manifest.version.clone(),
            source: LockedSource {
                source_type: "github".to_string(),
                repo: source_label,
                rev: None,
                sub_path,
            },
            integrity: Some(result.integrity),
            targets: target_types,
            installed_files,
        });
        lock.save(&lock_path)?;
        println!("Updated copm.json and copm.lock");
    }

    Ok(())
}

/// Install all dependencies from copm.json
pub async fn run_all() -> Result<(), CopmError> {
    let copm_json_path = paths::copm_json_path();
    let config = CopmJson::load(&copm_json_path)?;

    if config.dependencies.is_empty() {
        println!("No dependencies in copm.json.");
        return Ok(());
    }

    let count = config.dependencies.len();
    println!("Installing {count} package(s) from copm.json...");

    for (name, dep) in &config.dependencies {
        println!();
        // Reconstruct the original specifier
        let spec = match &dep.sub_path {
            Some(sp) => format!("{}:{sp}", dep.source),
            None => dep.source.clone(),
        };
        if let Err(e) = run(&spec, false).await {
            eprintln!("Failed to install {name}: {e}");
        }
    }

    println!();
    println!("Done.");
    Ok(())
}

/// Derive a package name from repo + optional sub_path.
/// "awesome-copilot" + Some("agents")                        → "awesome-copilot-agents"
/// "awesome-copilot" + Some("prompts/update-llms.prompt.md") → "awesome-copilot-update-llms"
/// "humanizer"       + None                                  → "humanizer"
fn package_name(repo: &str, sub_path: Option<&str>) -> String {
    match sub_path {
        Some(sp) => {
            // Use the last segment of sub_path, stripping any file extension
            let last = sp.split('/').last().unwrap_or(sp);
            let suffix = strip_md_extensions(last);
            if repo.ends_with(suffix) {
                repo.to_string()
            } else {
                format!("{repo}-{suffix}")
            }
        }
        None => repo.to_string(),
    }
}

/// Strip known Markdown-based extensions from a filename stem.
/// "update-llms.prompt.md" → "update-llms"
/// "agents"                → "agents"  (no change)
fn strip_md_extensions(name: &str) -> &str {
    for ext in [".prompt.md", ".agent.md", ".instructions.md", ".md"] {
        if let Some(stem) = name.strip_suffix(ext) {
            return stem;
        }
    }
    name
}
