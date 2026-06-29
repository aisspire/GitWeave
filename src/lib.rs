use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub const DEFAULT_CONFIG_PATH: &str = "config.yaml";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppConfig {
    pub everything_path: Option<PathBuf>,
}

impl AppConfig {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        match fs::read_to_string(path) {
            Ok(content) => parse_config_yaml(&content),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(format!("failed to read config {}: {error}", path.display())),
        }
    }
}

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
    run_with_config(&AppConfig::from_file(DEFAULT_CONFIG_PATH)?, stdout, stderr)
}

pub fn run_with_config<W: Write, E: Write>(
    config: &AppConfig,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), String> {
    let repositories = discover_git_repositories(config)?;
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

fn discover_git_repositories(config: &AppConfig) -> Result<Vec<PathBuf>, String> {
    let candidates = everything_executable_candidates(
        config.everything_path.as_deref(),
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

    Err(missing_everything_error(&missing))
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

fn missing_everything_error(missing: &[String]) -> String {
    format!(
        "failed to run es.exe: program not found; set everything.path in {DEFAULT_CONFIG_PATH} or checked {}",
        missing.join(", ")
    )
}

fn parse_config_yaml(content: &str) -> Result<AppConfig, String> {
    let mut in_everything = false;
    let mut everything_path = None;

    for raw_line in content.lines() {
        let line_without_comment = raw_line.split_once('#').map_or(raw_line, |(line, _)| line);
        if line_without_comment.trim().is_empty() {
            continue;
        }

        if !line_without_comment.starts_with(' ') && !line_without_comment.starts_with('\t') {
            in_everything = line_without_comment.trim() == "everything:";
            continue;
        }

        if in_everything {
            let trimmed = line_without_comment.trim();
            if let Some(raw_path) = trimmed.strip_prefix("path:") {
                let path = raw_path.trim().trim_matches('"').trim_matches('\'');
                if !path.is_empty() {
                    everything_path = Some(PathBuf::from(path));
                }
            }
        }
    }

    Ok(AppConfig { everything_path })
}

fn everything_executable_candidates(
    configured_path: Option<&Path>,
    program_files: Option<&str>,
    program_files_x86: Option<&str>,
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(configured_path) = configured_path {
        candidates.push(configured_path.to_path_buf());
    }

    candidates.push(PathBuf::from("es.exe"));

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
    fn parses_everything_path_from_yaml_config() {
        let config =
            parse_config_yaml("everything:\n  path: E:\\code\\GitWeave\\docs\\resource\\es.exe\n")
                .expect("config should parse");

        assert_eq!(
            config,
            AppConfig {
                everything_path: Some(std::path::PathBuf::from(
                    "E:\\code\\GitWeave\\docs\\resource\\es.exe"
                )),
            }
        );
    }

    #[test]
    fn parses_quoted_everything_path_from_yaml_config() {
        let config = parse_config_yaml(
            "everything:\n  path: \"E:\\code\\GitWeave\\docs\\resource\\es.exe\"\n",
        )
        .expect("config should parse");

        assert_eq!(
            config.everything_path,
            Some(std::path::PathBuf::from(
                "E:\\code\\GitWeave\\docs\\resource\\es.exe"
            ))
        );
    }

    #[test]
    fn checks_configured_path_and_common_everything_install_locations() {
        let candidates = everything_executable_candidates(
            Some(std::path::Path::new(
                "E:\\code\\GitWeave\\docs\\resource\\es.exe",
            )),
            Some("C:\\Program Files"),
            Some("C:\\Program Files (x86)"),
        );

        assert_eq!(
            candidates,
            vec![
                std::path::PathBuf::from("E:\\code\\GitWeave\\docs\\resource\\es.exe"),
                std::path::PathBuf::from("es.exe"),
                std::path::PathBuf::from("C:\\Program Files\\Everything\\es.exe"),
                std::path::PathBuf::from("C:\\Program Files (x86)\\Everything\\es.exe"),
            ]
        );
    }

    #[test]
    fn configured_everything_path_has_priority() {
        let candidates = everything_executable_candidates(
            Some(std::path::Path::new("D:\\tools\\Everything\\es.exe")),
            Some("C:\\Program Files"),
            Some("C:\\Program Files (x86)"),
        );

        assert_eq!(
            candidates,
            vec![
                std::path::PathBuf::from("D:\\tools\\Everything\\es.exe"),
                std::path::PathBuf::from("es.exe"),
                std::path::PathBuf::from("C:\\Program Files\\Everything\\es.exe"),
                std::path::PathBuf::from("C:\\Program Files (x86)\\Everything\\es.exe"),
            ]
        );
    }

    #[test]
    fn missing_everything_error_mentions_config_file() {
        let error = missing_everything_error(&["es.exe".to_string()]);

        assert!(error.contains(DEFAULT_CONFIG_PATH));
        assert!(error.contains("everything.path"));
    }
}
