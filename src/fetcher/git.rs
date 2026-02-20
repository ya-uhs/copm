use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::CopmError;

pub struct FetchResult {
    pub extracted_dir: PathBuf,
    pub integrity: String,
}

/// Parse "owner/repo" or "owner/repo:subpath" into (owner, repo, subpath).
pub fn parse_package_spec(spec: &str) -> Result<(String, String, Option<String>), CopmError> {
    // Split on ':' first to separate sub_path
    let (repo_part, sub_path) = match spec.split_once(':') {
        Some((r, s)) => (r, Some(s.to_string())),
        None => (spec, None),
    };

    let parts: Vec<&str> = repo_part.splitn(2, '/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(CopmError::InvalidPackageSpec(spec.to_string()));
    }

    // Validate sub_path is not empty when provided
    if let Some(ref sp) = sub_path {
        if sp.is_empty() {
            return Err(CopmError::InvalidPackageSpec(spec.to_string()));
        }
    }

    Ok((parts[0].to_string(), parts[1].to_string(), sub_path))
}

/// Download a GitHub repo tarball and extract it to dest_dir.
/// Returns the path to the extracted directory and integrity hash.
pub async fn fetch_github_tarball(
    user: &str,
    repo: &str,
    dest_dir: &Path,
) -> Result<FetchResult, CopmError> {
    let url = format!(
        "https://api.github.com/repos/{user}/{repo}/tarball/HEAD"
    );

    let client = reqwest::Client::builder()
        .user_agent("copm/0.1.0")
        .build()?;

    let response = client
        .get(&url)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| CopmError::DownloadFailed(e.to_string()))?;

    let bytes = response.bytes().await?;

    // Compute integrity hash
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = hasher.finalize();
    let integrity = format!("sha256-{}", hex::encode(hash));

    // Extract tarball
    let decoder = flate2::read::GzDecoder::new(&bytes[..]);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(dest_dir)?;

    // GitHub tarballs extract to a directory like "user-repo-commitsha/"
    // Find that directory
    let mut entries = std::fs::read_dir(dest_dir)?;
    let extracted = entries
        .next()
        .ok_or_else(|| CopmError::DownloadFailed("Empty tarball".to_string()))??;

    Ok(FetchResult {
        extracted_dir: extracted.path(),
        integrity,
    })
}

/// Fallback: clone with git
pub async fn fetch_git_clone(
    user: &str,
    repo: &str,
    dest_dir: &Path,
) -> Result<FetchResult, CopmError> {
    let url = format!("https://github.com/{user}/{repo}.git");
    let clone_dir = dest_dir.join(repo);

    let output = tokio::process::Command::new("git")
        .args(["clone", "--depth", "1", &url])
        .arg(&clone_dir)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CopmError::DownloadFailed(format!("git clone failed: {stderr}")));
    }

    // Get the commit hash for integrity
    let rev_output = tokio::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&clone_dir)
        .output()
        .await?;

    let rev = String::from_utf8_lossy(&rev_output.stdout).trim().to_string();
    let integrity = format!("git-{rev}");

    Ok(FetchResult {
        extracted_dir: clone_dir,
        integrity,
    })
}

/// Fetch a package: try tarball first, then fall back to git clone
pub async fn fetch_package(
    user: &str,
    repo: &str,
    dest_dir: &Path,
) -> Result<FetchResult, CopmError> {
    match fetch_github_tarball(user, repo, dest_dir).await {
        Ok(result) => Ok(result),
        Err(_) => fetch_git_clone(user, repo, dest_dir).await,
    }
}
