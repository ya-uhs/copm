use std::path::PathBuf;

use copm::config::copm_json::CopmJson;
use copm::config::lock::{CopmLock, LockedPackage, LockedSource};
use copm::fetcher::git::parse_package_spec;
use copm::manifest::package_manifest::PackageManifest;

// ── parse_package_spec ────────────────────────────────────────────────────────

#[test]
fn test_parse_package_spec_simple() {
    let (owner, repo, sub_path) = parse_package_spec("user/repo").unwrap();
    assert_eq!(owner, "user");
    assert_eq!(repo, "repo");
    assert_eq!(sub_path, None);
}

#[test]
fn test_parse_package_spec_with_subpath() {
    let (owner, repo, sub_path) = parse_package_spec("github/awesome-copilot:agents").unwrap();
    assert_eq!(owner, "github");
    assert_eq!(repo, "awesome-copilot");
    assert_eq!(sub_path, Some("agents".to_string()));
}

#[test]
fn test_parse_package_spec_with_nested_subpath() {
    let (owner, repo, sub_path) = parse_package_spec("github/awesome-copilot:skills/planning").unwrap();
    assert_eq!(owner, "github");
    assert_eq!(repo, "awesome-copilot");
    assert_eq!(sub_path, Some("skills/planning".to_string()));
}

#[test]
fn test_parse_package_spec_invalid() {
    assert!(parse_package_spec("invalid").is_err());
    assert!(parse_package_spec("").is_err());
    assert!(parse_package_spec("/repo").is_err());
    assert!(parse_package_spec("user/").is_err());
    assert!(parse_package_spec("user/repo:").is_err());
}

// ── CopmJson ──────────────────────────────────────────────────────────────────

#[test]
fn test_copm_json_default_tools() {
    let config = CopmJson::default();
    assert_eq!(config.tools, vec!["copilot"]);
}

#[test]
fn test_copm_json_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("copm.json");

    let mut config = CopmJson::default();
    config.tools = vec!["copilot".to_string(), "claude".to_string()];
    config.add_dependency("humanizer", "blader/humanizer", "0.0.0", None);
    config.add_dependency(
        "agents",
        "github/awesome-copilot",
        "0.0.0",
        Some("agents".to_string()),
    );
    config.save(&path).unwrap();

    let loaded = CopmJson::load(&path).unwrap();
    assert_eq!(loaded.tools, vec!["copilot", "claude"]);
    assert!(loaded.dependencies.contains_key("humanizer"));
    let dep = &loaded.dependencies["humanizer"];
    assert_eq!(dep.source, "blader/humanizer");
    assert_eq!(dep.sub_path, None);

    let dep2 = &loaded.dependencies["agents"];
    assert_eq!(dep2.source, "github/awesome-copilot");
    assert_eq!(dep2.sub_path, Some("agents".to_string()));
}

#[test]
fn test_copm_json_remove_dependency() {
    let mut config = CopmJson::default();
    config.add_dependency("a", "user/a", "1.0.0", None);
    config.add_dependency("b", "user/b", "2.0.0", None);
    assert!(config.remove_dependency("a"));
    assert!(!config.dependencies.contains_key("a"));
    assert!(config.dependencies.contains_key("b"));
    assert!(!config.remove_dependency("nonexistent"));
}

// ── CopmLock ──────────────────────────────────────────────────────────────────

#[test]
fn test_lock_roundtrip_with_installed_files() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("copm.lock");

    let mut lock = CopmLock::default();
    lock.upsert_package(LockedPackage {
        name: "humanizer".to_string(),
        version: "0.0.0".to_string(),
        source: LockedSource {
            source_type: "github".to_string(),
            repo: "blader/humanizer".to_string(),
            rev: None,
            sub_path: None,
        },
        integrity: Some("sha256-deadbeef".to_string()),
        targets: vec!["skill".to_string()],
        installed_files: vec![
            ".github/skills/humanizer".to_string(),
        ],
    });
    lock.save(&path).unwrap();

    let loaded = CopmLock::load(&path).unwrap();
    assert_eq!(loaded.packages.len(), 1);
    let pkg = &loaded.packages[0];
    assert_eq!(pkg.name, "humanizer");
    assert_eq!(pkg.targets, vec!["skill"]);
    assert_eq!(pkg.installed_files, vec![".github/skills/humanizer"]);
    assert_eq!(pkg.source.sub_path, None);
}

#[test]
fn test_lock_roundtrip_with_subpath() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("copm.lock");

    let mut lock = CopmLock::default();
    lock.upsert_package(LockedPackage {
        name: "awesome-agents".to_string(),
        version: "0.0.0".to_string(),
        source: LockedSource {
            source_type: "github".to_string(),
            repo: "github/awesome-copilot".to_string(),
            rev: None,
            sub_path: Some("agents".to_string()),
        },
        integrity: None,
        targets: vec!["copilot-agents".to_string()],
        installed_files: vec![
            ".github/agents/architect.agent.md".to_string(),
        ],
    });
    lock.save(&path).unwrap();

    let loaded = CopmLock::load(&path).unwrap();
    assert_eq!(loaded.packages[0].source.sub_path, Some("agents".to_string()));
}

#[test]
fn test_lock_upsert_replaces() {
    let mut lock = CopmLock::default();
    lock.upsert_package(LockedPackage {
        name: "pkg".to_string(),
        version: "1.0.0".to_string(),
        source: LockedSource {
            source_type: "github".to_string(),
            repo: "user/repo".to_string(),
            rev: None,
            sub_path: None,
        },
        integrity: None,
        targets: vec![],
        installed_files: vec![],
    });
    lock.upsert_package(LockedPackage {
        name: "pkg".to_string(),
        version: "2.0.0".to_string(),
        source: LockedSource {
            source_type: "github".to_string(),
            repo: "user/repo".to_string(),
            rev: None,
            sub_path: None,
        },
        integrity: None,
        targets: vec![],
        installed_files: vec![],
    });
    assert_eq!(lock.packages.len(), 1);
    assert_eq!(lock.packages[0].version, "2.0.0");
}

#[test]
fn test_lock_remove() {
    let mut lock = CopmLock::default();
    lock.upsert_package(LockedPackage {
        name: "pkg".to_string(),
        version: "1.0.0".to_string(),
        source: LockedSource {
            source_type: "github".to_string(),
            repo: "user/repo".to_string(),
            rev: None,
            sub_path: None,
        },
        integrity: None,
        targets: vec![],
        installed_files: vec![],
    });
    assert!(lock.remove_package("pkg"));
    assert!(lock.packages.is_empty());
    assert!(!lock.remove_package("pkg"));
}

// ── PackageManifest::detect_from_dir ─────────────────────────────────────────

#[test]
fn test_detect_skill_at_root() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("SKILL.md"), "# Humanizer skill").unwrap();
    std::fs::write(tmp.path().join("README.md"), "# Readme").unwrap();

    let manifest = PackageManifest::detect_from_dir(tmp.path(), None, "blader/humanizer").unwrap();
    assert_eq!(manifest.targets.len(), 1);
    assert_eq!(manifest.targets[0].target_type, "skill");
    assert_eq!(manifest.targets[0].path, ".");
}

#[test]
fn test_detect_copilot_instructions_at_root() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("copilot-instructions.md"), "# Instructions").unwrap();

    let manifest = PackageManifest::detect_from_dir(tmp.path(), None, "user/repo").unwrap();
    assert_eq!(manifest.targets.len(), 1);
    assert_eq!(manifest.targets[0].target_type, "copilot-instructions");
}

#[test]
fn test_detect_custom_instructions() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("react.instructions.md"), "Use React").unwrap();
    std::fs::write(tmp.path().join("testing.instructions.md"), "Use vitest").unwrap();

    let manifest = PackageManifest::detect_from_dir(tmp.path(), None, "user/repo").unwrap();
    assert_eq!(manifest.targets.len(), 1);
    assert_eq!(manifest.targets[0].target_type, "copilot-custom-instructions");
}

#[test]
fn test_detect_agents() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("architect.agent.md"), "# Architect").unwrap();

    let manifest = PackageManifest::detect_from_dir(tmp.path(), None, "user/repo").unwrap();
    assert_eq!(manifest.targets.len(), 1);
    assert_eq!(manifest.targets[0].target_type, "copilot-agents");
}

#[test]
fn test_detect_prompts() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("code-review.prompt.md"), "# Review").unwrap();

    let manifest = PackageManifest::detect_from_dir(tmp.path(), None, "user/repo").unwrap();
    assert_eq!(manifest.targets.len(), 1);
    assert_eq!(manifest.targets[0].target_type, "copilot-prompts");
}

#[test]
fn test_detect_skill_collection_in_subdir() {
    let tmp = tempfile::tempdir().unwrap();
    // skills/ directory with sub-skills
    let planning = tmp.path().join("skills").join("planning");
    std::fs::create_dir_all(&planning).unwrap();
    std::fs::write(planning.join("SKILL.md"), "# Planning skill").unwrap();
    let writing = tmp.path().join("skills").join("writing");
    std::fs::create_dir_all(&writing).unwrap();
    std::fs::write(writing.join("SKILL.md"), "# Writing skill").unwrap();

    // With subpath "skills" → single skill-collection target
    let manifest = PackageManifest::detect_from_dir(tmp.path(), Some("skills"), "user/repo").unwrap();
    assert_eq!(manifest.targets.len(), 1);
    assert_eq!(manifest.targets[0].target_type, "skill");
    assert_eq!(manifest.targets[0].path, "skills");
}

#[test]
fn test_detect_with_subpath_agents() {
    let tmp = tempfile::tempdir().unwrap();
    let agents_dir = tmp.path().join("agents");
    std::fs::create_dir_all(&agents_dir).unwrap();
    std::fs::write(agents_dir.join("architect.agent.md"), "# Architect").unwrap();
    // decoy in other dir
    let prompts_dir = tmp.path().join("prompts");
    std::fs::create_dir_all(&prompts_dir).unwrap();
    std::fs::write(prompts_dir.join("review.prompt.md"), "# Review").unwrap();

    let manifest = PackageManifest::detect_from_dir(tmp.path(), Some("agents"), "awesome/copilot").unwrap();
    assert_eq!(manifest.targets.len(), 1);
    assert_eq!(manifest.targets[0].target_type, "copilot-agents");
    assert_eq!(manifest.targets[0].path, "agents");
}

#[test]
fn test_detect_ambiguous_error() {
    let tmp = tempfile::tempdir().unwrap();
    // agents/ and prompts/ subdirs → ambiguous
    let agents_dir = tmp.path().join("agents");
    std::fs::create_dir_all(&agents_dir).unwrap();
    std::fs::write(agents_dir.join("a.agent.md"), "").unwrap();
    let prompts_dir = tmp.path().join("prompts");
    std::fs::create_dir_all(&prompts_dir).unwrap();
    std::fs::write(prompts_dir.join("b.prompt.md"), "").unwrap();

    let result = PackageManifest::detect_from_dir(tmp.path(), None, "github/awesome-copilot");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Multiple targets detected"), "got: {err}");
}

#[test]
fn test_detect_no_targets_error() {
    let tmp = tempfile::tempdir().unwrap();
    // Empty directory
    let result = PackageManifest::detect_from_dir(tmp.path(), None, "user/empty");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("No recognizable targets"), "got: {err}");
}

// ── paths ─────────────────────────────────────────────────────────────────────

#[test]
fn test_paths_local() {
    assert_eq!(
        copm::paths::copilot_instructions_path(),
        PathBuf::from(".github/copilot-instructions.md")
    );
    assert_eq!(
        copm::paths::copilot_custom_instructions_dir(),
        PathBuf::from(".github/instructions")
    );
    assert_eq!(
        copm::paths::copilot_agents_dir(),
        PathBuf::from(".github/agents")
    );
    assert_eq!(
        copm::paths::copilot_prompts_dir(),
        PathBuf::from(".github/prompts")
    );
    assert_eq!(
        copm::paths::local_copilot_skills_dir("humanizer"),
        PathBuf::from(".github/skills/humanizer")
    );
    assert_eq!(
        copm::paths::local_claude_skills_dir("humanizer"),
        PathBuf::from(".claude/skills/humanizer")
    );
    assert_eq!(
        copm::paths::local_claude_commands_dir(),
        PathBuf::from(".claude/commands")
    );
}

#[test]
fn test_paths_global() {
    let global_copilot = copm::paths::global_copilot_skills_dir("humanizer").unwrap();
    assert!(global_copilot.to_string_lossy().contains(".copilot/skills/humanizer"));

    let global_claude = copm::paths::global_claude_skills_dir("humanizer").unwrap();
    assert!(global_claude.to_string_lossy().contains(".claude/skills/humanizer"));
}

// ── Installer: install_file_collection ───────────────────────────────────────

#[test]
fn test_install_agents() {
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("source");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(source.join("architect.agent.md"), "# Architect").unwrap();
    std::fs::write(source.join("reviewer.agent.md"), "# Reviewer").unwrap();
    std::fs::write(source.join("README.md"), "Not an agent").unwrap();

    let dest = tmp.path().join("dest");
    let installed = copm::installer::copilot::install_file_collection(&source, ".agent.md", &dest).unwrap();

    assert_eq!(installed.len(), 2);
    assert!(dest.join("architect.agent.md").exists());
    assert!(dest.join("reviewer.agent.md").exists());
    assert!(!dest.join("README.md").exists());
}

#[test]
fn test_install_prompts() {
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("source");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(source.join("code-review.prompt.md"), "# Code review").unwrap();
    std::fs::write(source.join("other.md"), "other").unwrap();

    let dest = tmp.path().join("dest");
    let installed = copm::installer::copilot::install_file_collection(&source, ".prompt.md", &dest).unwrap();

    assert_eq!(installed.len(), 1);
    assert!(dest.join("code-review.prompt.md").exists());
    assert!(!dest.join("other.md").exists());
}

#[test]
fn test_install_skill_single() {
    let tmp = tempfile::tempdir().unwrap();

    // Skill dir with SKILL.md
    let skill_dir = tmp.path().join("source");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), "# Humanizer").unwrap();
    std::fs::write(skill_dir.join("prompt.md"), "Do something").unwrap();

    // Override install dirs to use tmp
    // We'll test the copy logic directly via a helper
    let dest = tmp.path().join(".github").join("skills").join("humanizer");
    std::fs::create_dir_all(dest.parent().unwrap()).unwrap();

    // Replicate what install_single_skill does
    walkdir::WalkDir::new(&skill_dir)
        .min_depth(1)
        .into_iter()
        .flatten()
        .for_each(|e| {
            let rel = e.path().strip_prefix(&skill_dir).unwrap();
            let dp = dest.join(rel);
            if e.file_type().is_dir() {
                std::fs::create_dir_all(&dp).unwrap();
            } else {
                std::fs::create_dir_all(dp.parent().unwrap()).unwrap();
                std::fs::copy(e.path(), &dp).unwrap();
            }
        });

    assert!(dest.join("SKILL.md").exists());
    assert!(dest.join("prompt.md").exists());
}

#[test]
fn test_uninstall_by_files() {
    let tmp = tempfile::tempdir().unwrap();

    // Create files and dirs to remove
    let file1 = tmp.path().join("file1.md");
    std::fs::write(&file1, "content").unwrap();

    let dir1 = tmp.path().join("somedir");
    std::fs::create_dir_all(&dir1).unwrap();
    std::fs::write(dir1.join("nested.md"), "nested").unwrap();

    let files = vec![
        file1.to_string_lossy().to_string(),
        dir1.to_string_lossy().to_string(),
    ];

    copm::installer::copilot::uninstall_by_files(&files).unwrap();

    assert!(!file1.exists());
    assert!(!dir1.exists());
}

// ── Single-file install ───────────────────────────────────────────────────────

#[test]
fn test_detect_single_file_prompt() {
    let tmp = tempfile::tempdir().unwrap();
    let prompts_dir = tmp.path().join("prompts");
    std::fs::create_dir_all(&prompts_dir).unwrap();
    std::fs::write(prompts_dir.join("update-llms.prompt.md"), "# Update LLMs").unwrap();
    std::fs::write(prompts_dir.join("other.prompt.md"), "# Other").unwrap();

    // Specifying a single file → only that file is the target
    let manifest = PackageManifest::detect_from_dir(
        tmp.path(),
        Some("prompts/update-llms.prompt.md"),
        "github/awesome-copilot",
    )
    .unwrap();
    assert_eq!(manifest.targets.len(), 1);
    assert_eq!(manifest.targets[0].target_type, "copilot-prompts");
    assert_eq!(manifest.targets[0].path, "prompts/update-llms.prompt.md");
}

#[test]
fn test_detect_single_file_agent() {
    let tmp = tempfile::tempdir().unwrap();
    let agents_dir = tmp.path().join("agents");
    std::fs::create_dir_all(&agents_dir).unwrap();
    std::fs::write(agents_dir.join("architect.agent.md"), "# Architect").unwrap();

    let manifest = PackageManifest::detect_from_dir(
        tmp.path(),
        Some("agents/architect.agent.md"),
        "github/awesome-copilot",
    )
    .unwrap();
    assert_eq!(manifest.targets[0].target_type, "copilot-agents");
    assert_eq!(manifest.targets[0].path, "agents/architect.agent.md");
}

#[test]
fn test_detect_single_file_instructions() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("react.instructions.md"), "Use React").unwrap();

    let manifest =
        PackageManifest::detect_from_dir(tmp.path(), Some("react.instructions.md"), "user/repo")
            .unwrap();
    assert_eq!(manifest.targets[0].target_type, "copilot-custom-instructions");
}

#[test]
fn test_detect_single_file_unrecognized() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("Cargo.toml"), "[package]").unwrap();

    let result =
        PackageManifest::detect_from_dir(tmp.path(), Some("Cargo.toml"), "user/repo");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unrecognized file type"));
}

#[test]
fn test_install_single_file_prompt() {
    let tmp = tempfile::tempdir().unwrap();

    // Source: a prompts/ dir with two files
    let prompts_dir = tmp.path().join("source").join("prompts");
    std::fs::create_dir_all(&prompts_dir).unwrap();
    std::fs::write(prompts_dir.join("update-llms.prompt.md"), "# Update LLMs").unwrap();
    std::fs::write(prompts_dir.join("other.prompt.md"), "# Other").unwrap();

    // Detect with file subpath
    let manifest = PackageManifest::detect_from_dir(
        &tmp.path().join("source"),
        Some("prompts/update-llms.prompt.md"),
        "github/awesome-copilot",
    )
    .unwrap();
    assert_eq!(manifest.targets[0].target_type, "copilot-prompts");

    // Verify only one file would be installed (path points to a file)
    let target = &manifest.targets[0];
    let target_path = tmp.path().join("source").join(&target.path);
    assert!(target_path.is_file());

    // Manually replicate install_single_file to avoid CWD dependency
    let dest_dir = tmp.path().join("project").join(".github").join("prompts");
    std::fs::create_dir_all(&dest_dir).unwrap();
    let dest = dest_dir.join(target_path.file_name().unwrap());
    std::fs::copy(&target_path, &dest).unwrap();

    assert!(dest_dir.join("update-llms.prompt.md").exists());
    assert!(!dest_dir.join("other.prompt.md").exists()); // only one file copied
}

// ── Real-world repo structure compatibility tests ─────────────────────────────

#[test]
fn test_compat_anthropics_skills_style() {
    // anthropics/skills: skills/ 以下にスキルフォルダが並ぶ
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("skills/webapp-testing")).unwrap();
    std::fs::create_dir_all(root.join("skills/pdf")).unwrap();
    std::fs::create_dir_all(root.join(".claude-plugin")).unwrap();
    std::fs::write(root.join("skills/webapp-testing/SKILL.md"), "").unwrap();
    std::fs::write(root.join("skills/pdf/SKILL.md"), "").unwrap();

    let m = PackageManifest::detect_from_dir(root, None, "anthropics/skills").unwrap();
    assert_eq!(m.targets.len(), 1);
    assert_eq!(m.targets[0].target_type, "skill");
    assert_eq!(m.targets[0].path, "skills"); // skills/ ディレクトリ全体をコレクションとして認識
}

#[test]
fn test_compat_root_level_skills_style() {
    // ArtemisAI/skills-for-copilot: スキルフォルダがルート直下に並ぶ
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("webapp-testing")).unwrap();
    std::fs::create_dir_all(root.join("pdf")).unwrap();
    std::fs::write(root.join("webapp-testing/SKILL.md"), "").unwrap();
    std::fs::write(root.join("pdf/SKILL.md"), "").unwrap();

    let m = PackageManifest::detect_from_dir(root, None, "artemis/skills-for-copilot").unwrap();
    assert_eq!(m.targets.len(), 1);
    assert_eq!(m.targets[0].target_type, "skill");
    assert_eq!(m.targets[0].path, "."); // root 全体をスキルコレクションとして認識
}

#[test]
fn test_compat_category_skill_hierarchy_ambiguous() {
    // alirezarezvani/claude-skills: カテゴリ→スキルの2段階 + agents/ が混在
    // → AmbiguousTargets になる（agentsとskillが混在するため）
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("engineering/senior-dev")).unwrap();
    std::fs::create_dir_all(root.join("marketing/content")).unwrap();
    std::fs::create_dir_all(root.join("agents")).unwrap();
    std::fs::write(root.join("engineering/senior-dev/SKILL.md"), "").unwrap();
    std::fs::write(root.join("marketing/content/SKILL.md"), "").unwrap();
    std::fs::write(root.join("agents/architect.agent.md"), "").unwrap();

    let result = PackageManifest::detect_from_dir(root, None, "user/claude-skills");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Multiple targets detected"), "got: {err}");
}

#[test]
fn test_compat_category_skill_hierarchy_subpath() {
    // alirezarezvani/claude-skills: :engineering でサブパス指定 → skill コレクション
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("engineering/senior-dev")).unwrap();
    std::fs::create_dir_all(root.join("engineering/junior-dev")).unwrap();
    std::fs::write(root.join("engineering/senior-dev/SKILL.md"), "").unwrap();
    std::fs::write(root.join("engineering/junior-dev/SKILL.md"), "").unwrap();

    let m = PackageManifest::detect_from_dir(root, Some("engineering"), "user/claude-skills").unwrap();
    assert_eq!(m.targets[0].target_type, "skill");
    assert_eq!(m.targets[0].path, "engineering");
}

#[test]
fn test_compat_github_skills_dir_not_autodetected() {
    // microsoft/azure-devops-skills: .github/skills/ 配下のスキルはサブパス必須
    // → root スキャンでは NoTargetsDetected
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join(".github/skills/boards-my-work")).unwrap();
    std::fs::write(root.join(".github/skills/boards-my-work/SKILL.md"), "").unwrap();

    // サブパス無し → 検出されない
    let result = PackageManifest::detect_from_dir(root, None, "microsoft/azure-devops-skills");
    assert!(result.is_err());

    // :(.github/skills) サブパス指定 → 検出される
    let m = PackageManifest::detect_from_dir(root, Some(".github/skills"), "microsoft/azure-devops-skills").unwrap();
    assert_eq!(m.targets[0].target_type, "skill");
}

#[test]
fn test_compat_root_copilot_instructions_wins() {
    // SebastienDegodez スタイル: root の copilot-instructions.md が優先される
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join(".github/instructions")).unwrap();
    std::fs::write(root.join("copilot-instructions.md"), "# Main").unwrap();
    std::fs::write(root.join(".github/instructions/ddd.instructions.md"), "DDD").unwrap();

    // root スキャン → copilot-instructions.md を検出
    let m = PackageManifest::detect_from_dir(root, None, "user/copilot-instructions").unwrap();
    assert_eq!(m.targets[0].target_type, "copilot-instructions");

    // .github/instructions サブパス → custom-instructions を検出
    let m2 = PackageManifest::detect_from_dir(root, Some(".github/instructions"), "user/copilot-instructions").unwrap();
    assert_eq!(m2.targets[0].target_type, "copilot-custom-instructions");
}

#[test]
fn test_compat_deep_nested_instructions_need_subpath() {
    // Code-and-Sorts スタイル: instructions/languages/python/ に .instructions.md
    // → :instructions では検出されない（1段深い）
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("instructions/languages/python")).unwrap();
    std::fs::write(root.join("instructions/languages/python/python.instructions.md"), "").unwrap();

    // :instructions/languages では NoTargetsDetected（files が更に1段深い）
    let result = PackageManifest::detect_from_dir(root, Some("instructions/languages"), "user/repo");
    assert!(result.is_err());

    // :instructions/languages/python まで指定すると検出できる
    let m = PackageManifest::detect_from_dir(root, Some("instructions/languages/python"), "user/repo").unwrap();
    assert_eq!(m.targets[0].target_type, "copilot-custom-instructions");
}
