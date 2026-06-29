use crate::app::RepoCommitCount;
use crate::config::AppConfig;
use crate::git::RepositoryReport;

pub fn format_repo_commit_counts(counts: &[RepoCommitCount]) -> String {
    let mut output = String::new();
    for count in counts {
        output.push_str(&format!("{}\t{}\n", count.repo.display(), count.commits));
    }
    output
}

pub fn format_repository_reports(reports: &[RepositoryReport], config: &AppConfig) -> String {
    let mut output = String::new();

    for (index, report) in reports.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }

        output.push_str(&format!("== {} ==\n", report.repo.display()));
        output.push_str(&format!("branches: {}\n", report.branches.join(", ")));

        if config.commits.show_details {
            output.push_str("\nCommits\n");
            for commit in &report.commits {
                output.push_str(&format!("commit {}\n", commit.hash));
                if config.commits.fields.author {
                    output.push_str(&format!("  author: {}\n", commit.author));
                }
                if config.commits.fields.email {
                    output.push_str(&format!("  email: {}\n", commit.email));
                }
                if config.commits.fields.time {
                    output.push_str(&format!("  time: {}\n", commit.time));
                }
                if config.commits.fields.lines {
                    output.push_str(&format!(
                        "  lines: +{} -{}\n",
                        commit.additions, commit.deletions
                    ));
                }
                if config.commits.fields.branches {
                    output.push_str(&format!("  branches: {}\n", commit.branches.join(", ")));
                }
            }
        }

        if config.summary.show_author_summary {
            output.push_str("\nAuthor Summary\n");
            output.push_str("author\tcommits\tadditions\tdeletions\tbranches\n");
            for summary in &report.author_summaries {
                output.push_str(&format!(
                    "{}\t{}\t+{}\t-{}\t{}\n",
                    summary.author,
                    summary.commits,
                    summary.additions,
                    summary.deletions,
                    summary.branches.join(", ")
                ));
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn formats_repository_commit_counts_for_terminal_output() {
        let counts = vec![RepoCommitCount {
            repo: PathBuf::from("E:\\code\\GitWeave"),
            commits: 2,
        }];

        let output = format_repo_commit_counts(&counts);

        assert_eq!(output, "E:\\code\\GitWeave\t2\n");
    }

    #[test]
    fn formats_repository_reports_with_project_sections_details_and_summary() {
        let mut config = crate::config::AppConfig::default();
        config.commits.fields.email = false;
        let report = crate::git::RepositoryReport {
            repo: PathBuf::from("E:\\code\\GitWeave"),
            branches: vec!["main".to_string(), "feature".to_string()],
            commits: vec![crate::git::CommitDetail {
                hash: "abc123".to_string(),
                author: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                time: "2026-06-30T10:00:00+08:00".to_string(),
                additions: 10,
                deletions: 2,
                branches: vec!["feature".to_string(), "main".to_string()],
            }],
            author_summaries: vec![crate::git::AuthorSummary {
                author: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                commits: 1,
                additions: 10,
                deletions: 2,
                branches: vec!["feature".to_string(), "main".to_string()],
            }],
        };

        let output = format_repository_reports(&[report], &config);

        assert!(output.contains("== E:\\code\\GitWeave =="));
        assert!(output.contains("branches: main, feature"));
        assert!(output.contains("commit abc123"));
        assert!(output.contains("author: Alice"));
        assert!(!output.contains("alice@example.com"));
        assert!(output.contains("lines: +10 -2"));
        assert!(output.contains("Author Summary"));
        assert!(output.contains("Alice\t1\t+10\t-2\tfeature, main"));
    }

    #[test]
    fn hides_commit_details_when_config_disables_them() {
        let mut config = crate::config::AppConfig::default();
        config.commits.show_details = false;
        let report = crate::git::RepositoryReport {
            repo: PathBuf::from("E:\\code\\GitWeave"),
            branches: vec!["main".to_string()],
            commits: vec![crate::git::CommitDetail {
                hash: "abc123".to_string(),
                author: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                time: "2026-06-30T10:00:00+08:00".to_string(),
                additions: 10,
                deletions: 2,
                branches: vec!["main".to_string()],
            }],
            author_summaries: vec![],
        };

        let output = format_repository_reports(&[report], &config);

        assert!(!output.contains("commit abc123"));
    }
}
