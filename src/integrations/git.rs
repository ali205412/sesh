//! Git integration
//!
//! Provides git status information for sessions.

use std::path::Path;

/// Git repository status
#[derive(Debug, Clone)]
pub struct GitStatus {
    /// Current branch name
    pub branch: String,
    /// Whether the working directory is clean
    pub is_clean: bool,
    /// Number of uncommitted changes
    pub changes: usize,
}

/// Get git status for a directory
pub fn get_git_status(dir: &Path) -> Option<GitStatus> {
    // Try to open the repository
    let repo = git2::Repository::discover(dir).ok()?;

    // Get current branch
    let head = repo.head().ok()?;
    let branch = head
        .shorthand()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "HEAD".to_string());

    // Check for changes
    let statuses = repo.statuses(None).ok()?;
    let changes = statuses.len();
    let is_clean = changes == 0;

    Some(GitStatus {
        branch,
        is_clean,
        changes,
    })
}

/// Get git branch name for a directory
pub fn get_branch(dir: &Path) -> Option<String> {
    let repo = git2::Repository::discover(dir).ok()?;
    let head = repo.head().ok()?;
    head.shorthand().map(|s| s.to_string())
}

/// Check if directory is clean (no uncommitted changes)
pub fn is_clean(dir: &Path) -> Option<bool> {
    let repo = git2::Repository::discover(dir).ok()?;
    let statuses = repo.statuses(None).ok()?;
    Some(statuses.is_empty())
}

/// Get the repository root directory
pub fn get_repo_root(dir: &Path) -> Option<String> {
    let repo = git2::Repository::discover(dir).ok()?;
    repo.workdir().map(|p| p.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_git_status() {
        // This test will only work if run from within a git repo
        let current_dir = PathBuf::from(".");
        let status = get_git_status(&current_dir);
        // We just check it doesn't panic
        // In a real git repo, it should return Some
    }
}
