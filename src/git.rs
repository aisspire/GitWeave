use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::logging;

const COMMIT_PREFIX: &str = "__GW_COMMIT__";
const FIELD_SEPARATOR: char = '\u{1f}';

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitDetail {
    pub hash: String,
    pub author: String,
    pub email: String,
    pub time: String,
    pub additions: u64,
    pub deletions: u64,
    pub branches: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorSummary {
    pub author: String,
    pub email: String,
    pub commits: u64,
    pub additions: u64,
    pub deletions: u64,
    pub branches: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryReport {
    pub repo: PathBuf,
    pub branches: Vec<String>,
    pub commits: Vec<CommitDetail>,
    pub author_summaries: Vec<AuthorSummary>,
}

pub fn count_repository_commits(repository: &Path) -> Result<u64, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .arg("rev-list")
        .arg("--count")
        .arg("HEAD")
        .output()
        .map_err(|error| format!("failed to run git: {error}"))?;

    if !output.status.success() {
        return Err(logging::format_command_failure(
            &format!("git -C {} rev-list --count HEAD", repository.display()),
            &output.stderr,
        ));
    }

    parse_commit_count(&String::from_utf8_lossy(&output.stdout))
}

pub fn collect_repository_report(repository: &Path) -> Result<RepositoryReport, String> {
    let branches = list_local_branches(repository)?;
    let mut commits = Vec::new();

    for branch in &branches {
        let branch_commits = collect_branch_commits(repository, branch)?;
        merge_commit_details(&mut commits, branch_commits);
    }

    let author_summaries = summarize_by_author(&commits);

    Ok(RepositoryReport {
        repo: repository.to_path_buf(),
        branches,
        commits,
        author_summaries,
    })
}

fn list_local_branches(repository: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .arg("for-each-ref")
        .arg("--format=%(refname:short)")
        .arg("refs/heads")
        .output()
        .map_err(|error| format!("failed to run git: {error}"))?;

    if !output.status.success() {
        return Err(logging::format_command_failure(
            &format!(
                "git -C {} for-each-ref --format=%(refname:short) refs/heads",
                repository.display()
            ),
            &output.stderr,
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn collect_branch_commits(repository: &Path, branch: &str) -> Result<Vec<CommitDetail>, String> {
    let pretty_format = format!("{COMMIT_PREFIX}%x1f%H%x1f%an%x1f%ae%x1f%aI");
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .arg("log")
        .arg(branch)
        .arg("--date=iso-strict")
        .arg(format!("--pretty=format:{pretty_format}"))
        .arg("--numstat")
        .output()
        .map_err(|error| format!("failed to run git: {error}"))?;

    if !output.status.success() {
        return Err(logging::format_command_failure(
            &format!("git -C {} log {branch} --numstat", repository.display()),
            &output.stderr,
        ));
    }

    parse_branch_git_log(branch, &String::from_utf8_lossy(&output.stdout))
}

pub fn parse_branch_git_log(branch: &str, output: &str) -> Result<Vec<CommitDetail>, String> {
    let mut commits = Vec::new();
    let mut current = None;

    for line in output.lines() {
        if line.starts_with(COMMIT_PREFIX) {
            if let Some(commit) = current.take() {
                commits.push(commit);
            }
            current = Some(parse_commit_header(branch, line)?);
            continue;
        }

        if let Some(commit) = current.as_mut() {
            add_numstat_line(commit, line)?;
        }
    }

    if let Some(commit) = current {
        commits.push(commit);
    }

    Ok(commits)
}

pub fn merge_commit_details(existing: &mut Vec<CommitDetail>, incoming: Vec<CommitDetail>) {
    for incoming_commit in incoming {
        if let Some(existing_commit) = existing
            .iter_mut()
            .find(|commit| commit.hash == incoming_commit.hash)
        {
            let mut branches: BTreeSet<String> = existing_commit.branches.iter().cloned().collect();
            branches.extend(incoming_commit.branches);
            existing_commit.branches = branches.into_iter().collect();
            continue;
        }

        existing.push(incoming_commit);
    }
}

pub fn summarize_by_author(commits: &[CommitDetail]) -> Vec<AuthorSummary> {
    let mut summaries = BTreeMap::<(String, String), AuthorSummary>::new();

    for commit in commits {
        let key = (commit.author.clone(), commit.email.clone());
        let summary = summaries.entry(key).or_insert_with(|| AuthorSummary {
            author: commit.author.clone(),
            email: commit.email.clone(),
            commits: 0,
            additions: 0,
            deletions: 0,
            branches: Vec::new(),
        });

        summary.commits += 1;
        summary.additions += commit.additions;
        summary.deletions += commit.deletions;

        let mut branches: BTreeSet<String> = summary.branches.iter().cloned().collect();
        branches.extend(commit.branches.iter().cloned());
        summary.branches = branches.into_iter().collect();
    }

    summaries.into_values().collect()
}

pub fn parse_commit_count(output: &str) -> Result<u64, String> {
    output
        .trim()
        .parse::<u64>()
        .map_err(|error| format!("failed to parse commit count: {error}"))
}

fn parse_commit_header(branch: &str, line: &str) -> Result<CommitDetail, String> {
    let fields: Vec<_> = line.split(FIELD_SEPARATOR).collect();
    if fields.len() != 5 || fields[0] != COMMIT_PREFIX {
        return Err(format!("failed to parse commit header: {line}"));
    }

    Ok(CommitDetail {
        hash: fields[1].to_string(),
        author: fields[2].to_string(),
        email: fields[3].to_string(),
        time: fields[4].to_string(),
        additions: 0,
        deletions: 0,
        branches: vec![branch.to_string()],
    })
}

fn add_numstat_line(commit: &mut CommitDetail, line: &str) -> Result<(), String> {
    if line.trim().is_empty() {
        return Ok(());
    }

    let mut parts = line.split('\t');
    let Some(additions) = parts.next() else {
        return Ok(());
    };
    let Some(deletions) = parts.next() else {
        return Ok(());
    };

    if additions == "-" || deletions == "-" {
        return Ok(());
    }

    commit.additions += additions
        .parse::<u64>()
        .map_err(|error| format!("failed to parse added lines: {error}"))?;
    commit.deletions += deletions
        .parse::<u64>()
        .map_err(|error| format!("failed to parse deleted lines: {error}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_commit_count_output() {
        let count = parse_commit_count("42\n").expect("commit count should parse");

        assert_eq!(count, 42);
    }

    #[test]
    fn parses_commit_details_with_added_and_deleted_lines() {
        let commits = parse_branch_git_log(
            "main",
            "__GW_COMMIT__\u{1f}abc123\u{1f}Alice\u{1f}alice@example.com\u{1f}2026-06-30T10:00:00+08:00\n10\t2\tsrc/lib.rs\n-\t-\tassets/logo.png\n",
        )
        .expect("git log should parse");

        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].hash, "abc123");
        assert_eq!(commits[0].author, "Alice");
        assert_eq!(commits[0].email, "alice@example.com");
        assert_eq!(commits[0].additions, 10);
        assert_eq!(commits[0].deletions, 2);
        assert_eq!(commits[0].branches, vec!["main"]);
    }

    #[test]
    fn merges_duplicate_commits_across_branches_and_summarizes_authors() {
        let mut commits = parse_branch_git_log(
            "main",
            "__GW_COMMIT__\u{1f}abc123\u{1f}Alice\u{1f}alice@example.com\u{1f}2026-06-30T10:00:00+08:00\n10\t2\tsrc/lib.rs\n",
        )
        .expect("main log should parse");
        let feature_commits = parse_branch_git_log(
            "feature",
            "__GW_COMMIT__\u{1f}abc123\u{1f}Alice\u{1f}alice@example.com\u{1f}2026-06-30T10:00:00+08:00\n10\t2\tsrc/lib.rs\n__GW_COMMIT__\u{1f}def456\u{1f}Bob\u{1f}bob@example.com\u{1f}2026-06-30T11:00:00+08:00\n3\t4\tsrc/main.rs\n",
        )
        .expect("feature log should parse");

        merge_commit_details(&mut commits, feature_commits);
        let summaries = summarize_by_author(&commits);

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].branches, vec!["feature", "main"]);
        assert_eq!(summaries[0].author, "Alice");
        assert_eq!(summaries[0].commits, 1);
        assert_eq!(summaries[0].additions, 10);
        assert_eq!(summaries[0].deletions, 2);
        assert_eq!(summaries[0].branches, vec!["feature", "main"]);
    }
}
