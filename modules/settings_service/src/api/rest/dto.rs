//! REST DTOs with serde derives for HTTP API

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ===== Setting DTOs =====

/// Setting response DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SettingDto {
    /// GTS identifier
    #[schema(example = "gts.a.p.sm.setting.v1.0~backup.schedule.v1.0")]
    pub r#type: String,
    
    /// Tenant ID
    pub tenant_id: Uuid,
    
    /// Domain object ID
    #[schema(example = "generic")]
    pub domain_object_id: String,
    
    /// Setting value as JSON
    pub data: serde_json::Value,
    
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Setting update request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateSettingRequest {
    /// Tenant ID
    pub tenant_id: Uuid,
    
    /// Domain object ID (optional, defaults to "generic")
    #[serde(default = "default_domain_object_id")]
    pub domain_object_id: String,
    
    /// Setting value as JSON
    pub data: serde_json::Value,
}

fn default_domain_object_id() -> String {
    "generic".to_string()
}

/// Lock setting request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct LockSettingRequest {
    /// Tenant ID
    pub tenant_id: Uuid,
    
    /// Domain object ID
    pub domain_object_id: String,
    
    /// Whether to enable read-only mode
    #[serde(default)]
    pub read_only: bool,
}

// ===== GTS Type DTOs =====

/// GTS type response DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GtsTypeDto {
    /// GTS identifier
    #[schema(example = "gts.a.p.sm.setting.v1.0~backup.schedule.v1.0")]
    pub r#type: String,
    
    /// GTS traits configuration
    pub traits: GtsTraitsDto,
    
    /// JSON Schema for validation (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
    
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// GTS traits configuration DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GtsTraitsDto {
    /// Domain type
    #[schema(example = "TENANT")]
    pub domain_type: String,
    
    /// Event configuration
    pub events: EventConfigDto,
    
    /// Setting options
    pub options: SettingOptionsDto,
    
    /// Operation configuration (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
}

/// Event configuration DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EventConfigDto {
    /// Audit event target
    #[schema(example = "SELF")]
    pub audit: String,
    
    /// Notification event target
    #[schema(example = "NONE")]
    pub notification: String,
}

/// Setting options DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SettingOptionsDto {
    /// Retention period in days
    #[serde(default = "default_retention_period")]
    pub retention_period: u32,
    
    /// Whether value is inheritable
    #[serde(default = "default_true")]
    pub is_value_inheritable: bool,
    
    /// Whether value can be overwritten
    #[serde(default = "default_true")]
    pub is_value_overwritable: bool,
    
    /// Whether inheritance is blocked
    #[serde(default)]
    pub is_barrier_inheritance: bool,
    
    /// Whether generic domain object is enabled
    #[serde(default = "default_true")]
    pub enable_generic: bool,
    
    /// Whether MFA is required
    #[serde(default)]
    pub is_mfa_required: bool,
}

fn default_retention_period() -> u32 {
    30
}

fn default_true() -> bool {
    true
}

/// Create/Update GTS type request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpsertGtsTypeRequest {
    /// GTS identifier
    #[schema(example = "gts.a.p.sm.setting.v1.0~backup.schedule.v1.0")]
    pub r#type: String,
    
    /// GTS traits configuration
    pub traits: GtsTraitsDto,
    
    /// JSON Schema for validation (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
}

// ===== List Response DTOs =====

/// Paginated list of settings
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SettingsListResponse {
    /// List of settings
    pub items: Vec<SettingDto>,
    
    /// Total count
    pub total: usize,
}

/// List of GTS types
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GtsTypesListResponse {
    /// List of GTS types
    pub items: Vec<GtsTypeDto>,
    
    /// Total count
    pub total: usize,
}

// Note: Conversion implementations moved to mapper.rs per module guidelines
