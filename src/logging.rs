use std::io::Write;

use regex::Regex;

use crate::config::LogConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Normal,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct LogRuntimeConfig {
    pub normal: bool,
    pub warning: bool,
    pub error: bool,
    pub color: bool,
    pub module_filter: Option<Regex>,
}

impl LogRuntimeConfig {
    pub fn from_config(config: &LogConfig) -> Result<Self, String> {
        Ok(Self {
            normal: config.normal,
            warning: config.warning,
            error: config.error,
            color: config.color,
            module_filter: config
                .module_filter
                .as_deref()
                .map(Regex::new)
                .transpose()
                .map_err(|error| format!("invalid logs.module_filter regex: {error}"))?,
        })
    }
}

pub fn format_warning(message: &str) -> String {
    format!("warning: {message}")
}

pub fn format_log(
    level: LogLevel,
    module: &str,
    message: &str,
    config: &LogRuntimeConfig,
) -> Option<String> {
    if !is_level_enabled(level, config) || !is_module_enabled(module, config) {
        return None;
    }

    let label = match level {
        LogLevel::Normal => "info",
        LogLevel::Warning => "warning",
        LogLevel::Error => "error",
    };
    let line = format!("{label} [{module}] {message}");

    if !config.color {
        return Some(line);
    }

    match level {
        LogLevel::Normal => Some(line),
        LogLevel::Warning => Some(format!("\u{1b}[33m{line}\u{1b}[0m")),
        LogLevel::Error => Some(format!("\u{1b}[31m{line}\u{1b}[0m")),
    }
}

pub fn write_log<W: Write>(
    stderr: &mut W,
    level: LogLevel,
    module: &str,
    message: &str,
    config: &LogRuntimeConfig,
) -> Result<(), String> {
    if let Some(line) = format_log(level, module, message, config) {
        writeln!(stderr, "{line}").map_err(|error| format!("failed to write log: {error}"))?;
    }
    Ok(())
}

pub fn write_warning<W: Write>(
    stderr: &mut W,
    module: &str,
    message: &str,
    config: &LogRuntimeConfig,
) -> Result<(), String> {
    write_log(stderr, LogLevel::Warning, module, message, config)
}

pub fn format_command_failure(command: &str, stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr);
    let stderr = stderr.trim();
    if stderr.is_empty() {
        format!("command failed: {command}")
    } else {
        format!("command failed: {command}: {stderr}")
    }
}

fn is_level_enabled(level: LogLevel, config: &LogRuntimeConfig) -> bool {
    match level {
        LogLevel::Normal => config.normal,
        LogLevel::Warning => config.warning,
        LogLevel::Error => config.error,
    }
}

fn is_module_enabled(module: &str, config: &LogRuntimeConfig) -> bool {
    config
        .module_filter
        .as_ref()
        .is_none_or(|filter| filter.is_match(module))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_warning_message_through_logging_module() {
        let warning = format_warning("skipped E:\\code\\broken: no HEAD");

        assert_eq!(warning, "warning: skipped E:\\code\\broken: no HEAD");
    }

    #[test]
    fn formats_warning_and_error_with_color_when_enabled() {
        let config = LogRuntimeConfig {
            normal: true,
            warning: true,
            error: true,
            color: true,
            module_filter: None,
        };

        assert_eq!(
            format_log(LogLevel::Warning, "git", "skipped repo", &config).as_deref(),
            Some("\u{1b}[33mwarning [git] skipped repo\u{1b}[0m")
        );
        assert_eq!(
            format_log(LogLevel::Error, "config", "bad yaml", &config).as_deref(),
            Some("\u{1b}[31merror [config] bad yaml\u{1b}[0m")
        );
    }

    #[test]
    fn filters_logs_by_level_and_module_regex() {
        let config = LogRuntimeConfig {
            normal: true,
            warning: false,
            error: true,
            color: false,
            module_filter: Some(regex::Regex::new("git|everything").expect("regex")),
        };

        assert!(format_log(LogLevel::Warning, "git", "hidden", &config).is_none());
        assert!(format_log(LogLevel::Error, "config", "hidden", &config).is_none());
        assert_eq!(
            format_log(LogLevel::Normal, "git", "collecting", &config).as_deref(),
            Some("info [git] collecting")
        );
    }
}
