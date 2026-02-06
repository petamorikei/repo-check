use serde::Serialize;
use std::path::PathBuf;

/// Repository check status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Status {
    /// Safe to delete
    Safe,
    /// Local-only changes exist (cannot delete)
    Unsafe,
    /// Cannot determine (deletion not recommended)
    Unknown,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Safe => write!(f, "SAFE"),
            Status::Unsafe => write!(f, "UNSAFE"),
            Status::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Reason for the check result
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    /// Uncommitted changes exist
    UncommittedChanges,
    /// Stash entries exist
    StashExists,
    /// Local-only commits exist
    LocalOnlyCommits,
    /// No remote tracking refs
    NoRemoteRefs,
    /// Git error occurred
    GitError(String),
    /// All checks passed
    AllChecksOk,
}

impl std::fmt::Display for Reason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reason::UncommittedChanges => write!(f, "Uncommitted changes exist"),
            Reason::StashExists => write!(f, "Stash entries exist"),
            Reason::LocalOnlyCommits => write!(f, "Local-only commits exist"),
            Reason::NoRemoteRefs => write!(f, "No remote tracking refs found"),
            Reason::GitError(msg) => write!(f, "Git error: {}", msg),
            Reason::AllChecksOk => write!(f, "All checks passed"),
        }
    }
}

/// Repository check result
#[derive(Debug, Clone, Serialize)]
pub struct RepoResult {
    /// Repository path
    pub path: PathBuf,
    /// Check status
    pub status: Status,
    /// Reasons for the status (multiple possible)
    pub reasons: Vec<Reason>,
    /// Number of dirty files
    pub dirty_count: usize,
    /// Number of stash entries
    pub stash_count: usize,
    /// Number of local-only commits
    pub local_only_commit_count: usize,
    /// Error messages (if any)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
}

impl RepoResult {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            status: Status::Safe,
            reasons: Vec::new(),
            dirty_count: 0,
            stash_count: 0,
            local_only_commit_count: 0,
            errors: Vec::new(),
        }
    }

    /// Mark as UNSAFE
    pub fn mark_unsafe(&mut self, reason: Reason) {
        self.status = Status::Unsafe;
        self.reasons.push(reason);
    }

    /// Mark as UNKNOWN (only if not already UNSAFE)
    pub fn mark_unknown(&mut self, reason: Reason) {
        if self.status != Status::Unsafe {
            self.status = Status::Unknown;
        }
        self.reasons.push(reason);
    }

    /// Finalize as SAFE
    pub fn finalize_safe(&mut self) {
        if self.status == Status::Safe {
            self.reasons.push(Reason::AllChecksOk);
        }
    }
}

/// User response for deletion confirmation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteConfirm {
    Yes,
    No,
    All,
    Quit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_priority_unsafe_over_unknown() {
        let mut result = RepoResult::new(PathBuf::from("/test"));
        result.mark_unsafe(Reason::UncommittedChanges);
        result.mark_unknown(Reason::NoRemoteRefs);
        // UNSAFE should take priority
        assert_eq!(result.status, Status::Unsafe);
        // Both reasons should be recorded
        assert_eq!(result.reasons.len(), 2);
    }

    #[test]
    fn test_finalize_safe() {
        let mut result = RepoResult::new(PathBuf::from("/test"));
        result.finalize_safe();
        assert_eq!(result.status, Status::Safe);
        assert!(result.reasons.contains(&Reason::AllChecksOk));
    }

    #[test]
    fn test_finalize_not_safe_when_unsafe() {
        let mut result = RepoResult::new(PathBuf::from("/test"));
        result.mark_unsafe(Reason::StashExists);
        result.finalize_safe();
        // Should NOT add AllChecksOk when UNSAFE
        assert!(!result.reasons.contains(&Reason::AllChecksOk));
    }
}
