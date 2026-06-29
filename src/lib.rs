pub mod app;
pub mod config;
pub mod everything;
pub mod git;
pub mod logging;
pub mod output;

pub use app::{run, run_with_config, RepoCommitCount};
pub use config::{AppConfig, DEFAULT_CONFIG_PATH};
pub use everything::parse_git_repositories;
pub use git::{parse_commit_count, AuthorSummary, CommitDetail, RepositoryReport};
pub use output::{format_repo_commit_counts, format_repository_reports};
