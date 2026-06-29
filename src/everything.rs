use std::collections::HashSet;
use std::env;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::{AppConfig, DEFAULT_CONFIG_PATH};
use crate::logging;

pub fn discover_git_repositories(config: &AppConfig) -> Result<Vec<PathBuf>, String> {
    let candidates = everything_executable_candidates(
        config.everything.path.as_deref(),
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
            return Err(logging::format_command_failure(
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

fn missing_everything_error(missing: &[String]) -> String {
    format!(
        "failed to run es.exe: program not found; set everything.path in {DEFAULT_CONFIG_PATH} or checked {}",
        missing.join(", ")
    )
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
                PathBuf::from("E:\\code\\GitWeave"),
                PathBuf::from("D:\\work\\app"),
            ]
        );
    }

    #[test]
    fn deduplicates_repositories_and_skips_non_git_paths() {
        let output = "D:\\work\\app\\.git\nD:\\work\\app\\.git\nD:\\notes\\not-git\n";

        let repos = parse_git_repositories(output);

        assert_eq!(repos, vec![PathBuf::from("D:\\work\\app")]);
    }

    #[test]
    fn checks_configured_path_and_common_everything_install_locations() {
        let candidates = everything_executable_candidates(
            Some(Path::new("E:\\code\\GitWeave\\docs\\resource\\es.exe")),
            Some("C:\\Program Files"),
            Some("C:\\Program Files (x86)"),
        );

        assert_eq!(
            candidates,
            vec![
                PathBuf::from("E:\\code\\GitWeave\\docs\\resource\\es.exe"),
                PathBuf::from("es.exe"),
                PathBuf::from("C:\\Program Files\\Everything\\es.exe"),
                PathBuf::from("C:\\Program Files (x86)\\Everything\\es.exe"),
            ]
        );
    }

    #[test]
    fn configured_everything_path_has_priority() {
        let candidates = everything_executable_candidates(
            Some(Path::new("D:\\tools\\Everything\\es.exe")),
            Some("C:\\Program Files"),
            Some("C:\\Program Files (x86)"),
        );

        assert_eq!(
            candidates,
            vec![
                PathBuf::from("D:\\tools\\Everything\\es.exe"),
                PathBuf::from("es.exe"),
                PathBuf::from("C:\\Program Files\\Everything\\es.exe"),
                PathBuf::from("C:\\Program Files (x86)\\Everything\\es.exe"),
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
