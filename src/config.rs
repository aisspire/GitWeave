use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::Deserialize;

pub const DEFAULT_CONFIG_PATH: &str = "config.yaml";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub everything: EverythingConfig,
    pub repositories: RepositoryConfig,
    pub commits: CommitDisplayConfig,
    pub summary: SummaryConfig,
    pub logs: LogConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            everything: EverythingConfig::default(),
            repositories: RepositoryConfig::default(),
            commits: CommitDisplayConfig::default(),
            summary: SummaryConfig::default(),
            logs: LogConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct EverythingConfig {
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct RepositoryConfig {
    pub limit: usize,
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        Self { limit: 0 }
    }
}

impl RepositoryConfig {
    pub fn is_unlimited(&self) -> bool {
        self.limit == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct CommitDisplayConfig {
    pub show_details: bool,
    pub fields: CommitFieldConfig,
}

impl Default for CommitDisplayConfig {
    fn default() -> Self {
        Self {
            show_details: true,
            fields: CommitFieldConfig::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct CommitFieldConfig {
    pub author: bool,
    pub email: bool,
    pub time: bool,
    pub lines: bool,
    pub branches: bool,
}

impl Default for CommitFieldConfig {
    fn default() -> Self {
        Self {
            author: true,
            email: true,
            time: true,
            lines: true,
            branches: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct SummaryConfig {
    pub show_author_summary: bool,
}

impl Default for SummaryConfig {
    fn default() -> Self {
        Self {
            show_author_summary: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct LogConfig {
    pub normal: bool,
    pub warning: bool,
    pub error: bool,
    pub color: bool,
    pub module_filter: Option<String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            normal: true,
            warning: true,
            error: true,
            color: true,
            module_filter: None,
        }
    }
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

pub(crate) fn parse_config_yaml(content: &str) -> Result<AppConfig, String> {
    let config: AppConfig = serde_yaml::from_str(content)
        .map_err(|error| format!("failed to parse config: {error}"))?;

    if let Some(module_filter) = &config.logs.module_filter {
        regex::Regex::new(module_filter)
            .map_err(|error| format!("invalid logs.module_filter regex: {error}"))?;
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_everything_path_from_yaml_config() {
        let config =
            parse_config_yaml("everything:\n  path: E:\\code\\GitWeave\\docs\\resource\\es.exe\n")
                .expect("config should parse");

        assert_eq!(
            config.everything.path,
            Some(PathBuf::from("E:\\code\\GitWeave\\docs\\resource\\es.exe"))
        );
    }

    #[test]
    fn parses_quoted_everything_path_from_yaml_config() {
        let config = parse_config_yaml(
            "everything:\n  path: 'E:\\code\\GitWeave\\docs\\resource\\es.exe'\n",
        )
        .expect("config should parse");

        assert_eq!(
            config.everything.path,
            Some(PathBuf::from("E:\\code\\GitWeave\\docs\\resource\\es.exe"))
        );
    }

    #[test]
    fn parses_full_config_schema_with_display_and_log_options() {
        let config = parse_config_yaml(
            r#"
everything:
  path: C:\Program Files\Everything\es.exe
repositories:
  limit: 3
commits:
  show_details: false
  fields:
    author: true
    email: false
    time: true
    lines: true
    branches: false
summary:
  show_author_summary: true
logs:
  normal: false
  warning: true
  error: true
  color: false
  module_filter: "git|everything"
"#,
        )
        .expect("config should parse");

        assert_eq!(
            config.everything.path,
            Some(PathBuf::from("C:\\Program Files\\Everything\\es.exe"))
        );
        assert_eq!(config.repositories.limit, 3);
        assert!(!config.commits.show_details);
        assert!(config.commits.fields.author);
        assert!(!config.commits.fields.email);
        assert!(config.summary.show_author_summary);
        assert!(!config.logs.normal);
        assert_eq!(config.logs.module_filter.as_deref(), Some("git|everything"));
    }

    #[test]
    fn rejects_invalid_log_module_regex() {
        let error = parse_config_yaml("logs:\n  module_filter: \"[\"\n")
            .expect_err("invalid regex should fail");

        assert!(error.contains("invalid logs.module_filter regex"));
    }

    #[test]
    fn default_config_treats_repository_limit_zero_as_unlimited() {
        let config = AppConfig::default();

        assert_eq!(config.repositories.limit, 0);
        assert!(config.repositories.is_unlimited());
    }
}
