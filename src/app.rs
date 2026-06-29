use std::io::Write;

use crate::config::{AppConfig, DEFAULT_CONFIG_PATH};
use crate::everything;
use crate::git;
use crate::logging;
use crate::output;

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoCommitCount {
    pub repo: PathBuf,
    pub commits: u64,
}

pub fn run<W: Write, E: Write>(stdout: &mut W, stderr: &mut E) -> Result<(), String> {
    let config = match AppConfig::from_file(DEFAULT_CONFIG_PATH) {
        Ok(config) => config,
        Err(error) => {
            let log_config = logging::LogRuntimeConfig::from_config(&Default::default())?;
            logging::write_log(
                stderr,
                logging::LogLevel::Error,
                "config",
                &error,
                &log_config,
            )?;
            return Err(error);
        }
    };

    run_with_config(&config, stdout, stderr)
}

pub fn run_with_config<W: Write, E: Write>(
    config: &AppConfig,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), String> {
    let log_config = logging::LogRuntimeConfig::from_config(&config.logs)?;
    let mut repositories = match everything::discover_git_repositories(config) {
        Ok(repositories) => repositories,
        Err(error) => {
            logging::write_log(
                stderr,
                logging::LogLevel::Error,
                "everything",
                &error,
                &log_config,
            )?;
            return Err(error);
        }
    };
    if !config.repositories.is_unlimited() {
        repositories.truncate(config.repositories.limit);
    }

    let mut reports = Vec::new();

    for repository in repositories {
        logging::write_log(
            stderr,
            logging::LogLevel::Normal,
            "git",
            &format!("collecting {}", repository.display()),
            &log_config,
        )?;

        match git::collect_repository_report(&repository) {
            Ok(report) => reports.push(report),
            Err(error) => logging::write_warning(
                stderr,
                "git",
                &format!("skipped {}: {error}", repository.display()),
                &log_config,
            )?,
        }
    }

    write!(
        stdout,
        "{}",
        output::format_repository_reports(&reports, config)
    )
    .map_err(|error| format!("failed to write output: {error}"))
}
