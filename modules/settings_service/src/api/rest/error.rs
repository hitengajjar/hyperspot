//! HTTP error mapping to RFC-9457 Problem Details

use crate::contract::SettingsError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// RFC-9457 Problem Details for HTTP API errors
#[derive(Debug, Serialize)]
pub struct Problem {
    /// A URI reference that identifies the problem type
    #[serde(rename = "type")]
    pub type_uri: String,
    
    /// A short, human-readable summary of the problem type
    pub title: String,
    
    /// The HTTP status code
    pub status: u16,
    
    /// A human-readable explanation specific to this occurrence
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    
    /// A URI reference that identifies the specific occurrence
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
}

impl Problem {
    /// Create a new Problem Details response
    pub fn new(status: StatusCode, title: impl Into<String>) -> Self {
        Self {
            type_uri: format!("https://httpstatuses.io/{}", status.as_u16()),
            title: title.into(),
            status: status.as_u16(),
            detail: None,
            instance: None,
        }
    }

    /// Add detail message
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Add instance URI
    pub fn with_instance(mut self, instance: impl Into<String>) -> Self {
        self.instance = Some(instance.into());
        self
    }
}

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

/// Map domain errors to HTTP Problem Details
pub fn map_domain_error(error: SettingsError) -> Problem {
    match error {
        SettingsError::NotFound { resource, id } => Problem::new(
            StatusCode::NOT_FOUND,
            format!("{} Not Found", resource),
        )
        .with_detail(format!("{} with id '{}' was not found", resource, id)),

        SettingsError::Conflict { reason } => {
            Problem::new(StatusCode::CONFLICT, "Conflict").with_detail(reason)
        }

        SettingsError::Validation { message } => Problem::new(
            StatusCode::BAD_REQUEST,
            "Validation Error",
        )
        .with_detail(message),

        SettingsError::Locked { setting_type } => Problem::new(
            StatusCode::LOCKED,
            "Setting Locked",
        )
        .with_detail(format!(
            "Setting '{}' is locked and cannot be modified",
            setting_type
        )),

        SettingsError::TypeNotRegistered { gts_type } => Problem::new(
            StatusCode::BAD_REQUEST,
            "GTS Type Not Registered",
        )
        .with_detail(format!(
            "GTS type '{}' must be registered before creating settings",
            gts_type
        )),

        SettingsError::InvalidGtsFormat { gts, details } => Problem::new(
            StatusCode::BAD_REQUEST,
            "Invalid GTS Format",
        )
        .with_detail(format!("Invalid GTS '{}': {}", gts, details)),

        SettingsError::SchemaValidation { errors } => Problem::new(
            StatusCode::BAD_REQUEST,
            "Schema Validation Failed",
        )
        .with_detail(format!("Validation errors: {}", errors.join(", "))),

        SettingsError::Internal => Problem::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error",
        )
        .with_detail("An unexpected error occurred"),
    }
}

/// Helper to convert anyhow errors to Problem Details
pub fn map_anyhow_error(error: anyhow::Error) -> Problem {
    tracing::error!("Internal error: {:?}", error);
    Problem::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal Server Error",
    )
    .with_detail("An unexpected error occurred")
}
