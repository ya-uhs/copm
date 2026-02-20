use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CopmError {
    #[error("Invalid package specifier: {0}")]
    InvalidPackageSpec(String),

    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Failed to download package: {0}")]
    DownloadFailed(String),

    #[error("copm.json not found in {0}")]
    CopmJsonNotFound(PathBuf),

    #[error("copm.json already exists")]
    CopmJsonAlreadyExists,

    #[error("Package '{0}' is not installed")]
    NotInstalled(String),

    #[error("No recognizable targets found in package: {0}")]
    NoTargetsDetected(String),

    #[error("Multiple targets detected in {pkg}:\n{targets}\nUse: copm install {pkg}:<subpath>")]
    AmbiguousTargets { pkg: String, targets: String },

    #[error("Unsupported target type: {0}")]
    UnsupportedTargetType(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}
