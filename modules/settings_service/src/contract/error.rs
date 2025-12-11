//! Contract error types for settings service
//!
//! These errors are transport-agnostic and used for inter-module communication.

/// Settings service domain errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsError {
    /// Setting or GTS type not found
    NotFound {
        /// Resource type (setting, gts_type)
        resource: String,
        /// Resource identifier
        id: String,
    },
    /// Conflict (duplicate, locked, etc.)
    Conflict {
        /// Conflict reason
        reason: String,
    },
    /// Validation error
    Validation {
        /// Validation error message
        message: String,
    },
    /// Setting is locked (compliance mode)
    Locked {
        /// Setting type
        setting_type: String,
    },
    /// GTS type not registered
    TypeNotRegistered {
        /// GTS type identifier
        gts_type: String,
    },
    /// Invalid GTS format
    InvalidGtsFormat {
        /// Invalid GTS string
        gts: String,
        /// Error details
        details: String,
    },
    /// JSON Schema validation failed
    SchemaValidation {
        /// Validation errors
        errors: Vec<String>,
    },
    /// Internal error
    Internal,
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { resource, id } => {
                write!(f, "{} not found: {}", resource, id)
            }
            Self::Conflict { reason } => {
                write!(f, "Conflict: {}", reason)
            }
            Self::Validation { message } => {
                write!(f, "Validation error: {}", message)
            }
            Self::Locked { setting_type } => {
                write!(f, "Setting is locked: {}", setting_type)
            }
            Self::TypeNotRegistered { gts_type } => {
                write!(f, "GTS type not registered: {}", gts_type)
            }
            Self::InvalidGtsFormat { gts, details } => {
                write!(f, "Invalid GTS format '{}': {}", gts, details)
            }
            Self::SchemaValidation { errors } => {
                write!(f, "Schema validation failed: {}", errors.join(", "))
            }
            Self::Internal => {
                write!(f, "Internal error")
            }
        }
    }
}

impl std::error::Error for SettingsError {}
