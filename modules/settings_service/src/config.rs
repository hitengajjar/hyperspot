//! Configuration for settings service module

use serde::Deserialize;

/// Settings service configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Feature flags for the module
    #[serde(default)]
    pub feature_flags: Vec<String>,

    /// Default retention period for deleted settings (days)
    #[serde(default = "default_retention_period")]
    pub default_retention_period: u32,

    /// Enable strict GTS validation
    #[serde(default = "default_true")]
    pub strict_cti_validation: bool,

    /// Enable JSON Schema validation
    #[serde(default = "default_true")]
    pub enable_schema_validation: bool,

    /// Maximum setting data size in bytes
    #[serde(default = "default_max_data_size")]
    pub max_data_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            feature_flags: Vec::new(),
            default_retention_period: default_retention_period(),
            strict_cti_validation: true,
            enable_schema_validation: true,
            max_data_size: default_max_data_size(),
        }
    }
}

fn default_retention_period() -> u32 {
    30
}

fn default_true() -> bool {
    true
}

fn default_max_data_size() -> usize {
    1024 * 1024 // 1MB
}
