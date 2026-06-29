use std::collections::HashSet;
use std::env;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoCommitCount {
    pub repo: PathBuf,
    pub commits: u64,
}

pub fn parse_git_repositories(output: &str) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut repositories = Vec::new();

    for line in output.lines() {
        if let Some(repository) = {
            let path = PathBuf::from(line.trim());
            if path.file_name().is_some_and(|name| name == ".git") {
                path.parent().map(PathBuf::from)
            } else {
                None
            }
        } {
            if seen.insert(repository.clone()) {
                repositories.push(repository);
            }
        }
    }

    repositories
}

pub fn parse_commit_count(output: &str) -> Result<u64, String> {
    output
        .trim()
        .parse::<u64>()
        .map_err(|error| format!("failed to parse commit count: {error}"))
}

pub fn format_repo_commit_counts(counts: &[RepoCommitCount]) -> String {
    let mut output = String::new();
    for count in counts {
        output.push_str(&format!("{}\t{}\n", count.repo.display(), count.commits));
    }
    output
}

pub fn run<W: Write, E: Write>(stdout: &mut W, stderr: &mut E) -> Result<(), String> {
    let repositories = discover_git_repositories()?;
    let mut counts = Vec::new();

    for repository in repositories {
        match count_repository_commits(&repository) {
            Ok(commits) => counts.push(RepoCommitCount {
                repo: repository,
                commits,
            }),
            Err(error) => {
                writeln!(stderr, "warning: skipped {}: {error}", repository.display())
                    .map_err(|write_error| format!("failed to write warning: {write_error}"))?;
            }
        }
    }

    write!(stdout, "{}", format_repo_commit_counts(&counts))
        .map_err(|error| format!("failed to write output: {error}"))
}

fn discover_git_repositories() -> Result<Vec<PathBuf>, String> {
    let candidates = everything_executable_candidates(
        env::var("ProgramFiles").ok().as_deref(),
        env::var("ProgramFiles(x86)").ok().as_deref(),
    );
    let mut missing = Vec::new();

    for candidate in &candidates {
        let output = match Command::new(candidate).arg(".git").output() {
            Ok(output) => output,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                missing.push(candidate.display().to_string());
                continue;
            }
            Err(error) => {
                return Err(format!("failed to run {}: {error}", candidate.display()));
            }
        };

        if !output.status.success() {
            return Err(format_command_failure(
                &format!("{} .git", candidate.display()),
                &output.stderr,
            ));
        }

        return Ok(parse_git_repositories(&String::from_utf8_lossy(
            &output.stdout,
        )));
    }

    Err(format!(
        "failed to run es.exe: program not found; checked {}",
        missing.join(", ")
    ))
}

fn count_repository_commits(repository: &Path) -> Result<u64, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .arg("rev-list")
        .arg("--count")
        .arg("HEAD")
        .output()
        .map_err(|error| format!("failed to run git: {error}"))?;

    if !output.status.success() {
        return Err(format_command_failure(
            &format!("git -C {} rev-list --count HEAD", repository.display()),
            &output.stderr,
        ));
    }

    parse_commit_count(&String::from_utf8_lossy(&output.stdout))
}

fn format_command_failure(command: &str, stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr);
    let stderr = stderr.trim();
    if stderr.is_empty() {
        format!("command failed: {command}")
    } else {
        format!("command failed: {command}: {stderr}")
    }
}

fn everything_executable_candidates(
    program_files: Option<&str>,
    program_files_x86: Option<&str>,
) -> Vec<PathBuf> {
    let mut candidates = vec![PathBuf::from("es.exe")];

    if let Some(program_files) = program_files {
        candidates.push(
            PathBuf::from(program_files)
                .join("Everything")
                .join("es.exe"),
        );
    }

    if let Some(program_files_x86) = program_files_x86 {
        candidates.push(
            PathBuf::from(program_files_x86)
                .join("Everything")
                .join("es.exe"),
        );
    }

    candidates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_everything_git_directories_into_repo_roots() {
        let output = "E:\\code\\GitWeave\\.git\r\nD:\\work\\app\\.git\n";

        let repos = parse_git_repositories(output);

        assert_eq!(
            repos,
            vec![
                std::path::PathBuf::from("E:\\code\\GitWeave"),
                std::path::PathBuf::from("D:\\work\\app"),
            ]
        );
    }

    #[test]
    fn deduplicates_repositories_and_skips_non_git_paths() {
        let output = "D:\\work\\app\\.git\nD:\\work\\app\\.git\nD:\\notes\\not-git\n";

        let repos = parse_git_repositories(output);

        assert_eq!(repos, vec![std::path::PathBuf::from("D:\\work\\app")]);
    }

    #[test]
    fn parses_commit_count_output() {
        let count = parse_commit_count("42\n").expect("commit count should parse");

        assert_eq!(count, 42);
    }

    #[test]
    fn formats_repository_commit_counts_for_terminal_output() {
        let counts = vec![RepoCommitCount {
            repo: std::path::PathBuf::from("E:\\code\\GitWeave"),
            commits: 2,
        }];

        let output = format_repo_commit_counts(&counts);

        assert_eq!(output, "E:\\code\\GitWeave\t2\n");
    }

    #[test]
    fn checks_path_and_common_everything_install_locations() {
        let candidates = everything_executable_candidates(
            Some("C:\\Program Files"),
            Some("C:\\Program Files (x86)"),
        );

        assert_eq!(
            candidates,
            vec![
                std::path::PathBuf::from("es.exe"),
                std::path::PathBuf::from("C:\\Program Files\\Everything\\es.exe"),
                std::path::PathBuf::from("C:\\Program Files (x86)\\Everything\\es.exe"),
            ]
        );
    }
}
