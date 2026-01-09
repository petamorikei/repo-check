use serde::Serialize;
use std::path::PathBuf;

/// リポジトリの判定ステータス
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Status {
    /// 削除しても安全
    Safe,
    /// ローカル固有の変更が存在（削除不可）
    Unsafe,
    /// 判定不能（原則削除不可）
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

/// 判定理由の種類
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    /// 未コミット変更あり
    UncommittedChanges,
    /// stash あり
    StashExists,
    /// ローカル専用コミットあり
    LocalOnlyCommits,
    /// リモート追跡参照なし
    NoRemoteRefs,
    /// Gitエラー
    GitError(String),
    /// 全チェックOK
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

/// リポジトリのチェック結果
#[derive(Debug, Clone, Serialize)]
pub struct RepoResult {
    /// リポジトリのパス
    pub path: PathBuf,
    /// 判定ステータス
    pub status: Status,
    /// 判定理由（複数）
    pub reasons: Vec<Reason>,
    /// 未コミット変更のファイル数
    pub dirty_count: usize,
    /// stash件数
    pub stash_count: usize,
    /// ローカル専用コミット数
    pub local_only_commit_count: usize,
    /// エラーメッセージ（あれば）
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

    /// UNSAFEとしてマーク
    pub fn mark_unsafe(&mut self, reason: Reason) {
        self.status = Status::Unsafe;
        self.reasons.push(reason);
    }

    /// UNKNOWNとしてマーク（既にUNSAFEでない場合）
    pub fn mark_unknown(&mut self, reason: Reason) {
        if self.status != Status::Unsafe {
            self.status = Status::Unknown;
        }
        self.reasons.push(reason);
    }

    /// SAFEを確定
    pub fn finalize_safe(&mut self) {
        if self.status == Status::Safe {
            self.reasons.push(Reason::AllChecksOk);
        }
    }
}

/// 削除時のユーザー応答
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteConfirm {
    Yes,
    No,
    All,
    Quit,
}
