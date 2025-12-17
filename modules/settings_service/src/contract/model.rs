//! Contract models for settings service
//!
//! These models are transport-agnostic and used for inter-module communication.
//! NO serde derives - these are pure domain models.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Setting value per GTS specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setting {
    /// GTS identifier (e.g., "gts.a.p.sm.setting.v1.0~vendor.app.feature.v1.0")
    pub r#type: String,
    /// Tenant ID that owns this setting
    pub tenant_id: Uuid,
    /// Domain object ID (UUID, GTS, AppCode, or "generic")
    pub domain_object_id: String,
    /// Setting value as JSON
    pub data: serde_json::Value,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Soft delete timestamp
    pub deleted_at: Option<DateTime<Utc>>,
}

/// GTS type definition with traits
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GtsType {
    /// GTS identifier
    pub r#type: String,
    /// GTS traits (domain_type, events, options, operations)
    pub traits: GtsTraits,
    /// JSON Schema for validation
    pub schema: Option<serde_json::Value>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// GTS traits configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GtsTraits {
    /// Domain type (TENANT, STORAGE, USER, AGENT, APPLICATION, BRAND, RESOURCE, GLOBAL)
    pub domain_type: DomainType,
    /// Event configuration
    pub events: EventConfig,
    /// Options for inheritance and behavior
    pub options: SettingOptions,
    /// Operation configuration
    pub operation: Option<String>,
}

/// Domain type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainType {
    Tenant,
    Storage,
    User,
    Agent,
    Application,
    Brand,
    Resource,
    Global,
}

/// Event configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventConfig {
    /// Audit event target (SELF, SUBROOT, NONE)
    pub audit: EventTarget,
    /// Notification event target (SELF, SUBROOT, NONE)
    pub notification: EventTarget,
}

/// Event target enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventTarget {
    /// Event for tenant where change happened
    Self_,
    /// Event for each tenant in the subtree
    Subroot,
    /// No event
    None,
}

/// Setting options for inheritance and behavior
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingOptions {
    /// Retention period in days
    pub retention_period: u32,
    /// Whether value is inheritable by child tenants
    pub is_value_inheritable: bool,
    /// Whether value can be overwritten by child tenants
    pub is_value_overwritable: bool,
    /// Whether inheritance is blocked at this level
    pub is_barrier_inheritance: bool,
    /// Whether generic domain object is enabled
    pub enable_generic: bool,
    /// Whether MFA is required for changes
    pub is_mfa_required: bool,
}

impl Default for SettingOptions {
    fn default() -> Self {
        Self {
            retention_period: 30,
            is_value_inheritable: true,
            is_value_overwritable: true,
            is_barrier_inheritance: false,
            enable_generic: true,
            is_mfa_required: false,
        }
    }
}

impl Default for EventConfig {
    fn default() -> Self {
        Self {
            audit: EventTarget::Self_,
            notification: EventTarget::None,
        }
    }
}

/// Authentication context for privilege-aware operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthContext {
    /// Whether the user has root/admin privileges
    pub is_root_admin: bool,
    /// Optional user identifier for audit logging
    pub user_id: Option<String>,
    /// Optional client identifier for audit logging
    pub client_id: Option<String>,
}

impl Default for AuthContext {
    fn default() -> Self {
        Self {
            is_root_admin: false,
            user_id: None,
            client_id: None,
        }
    }
}

impl AuthContext {
    /// Create a new non-admin context
    pub fn non_admin() -> Self {
        Self::default()
    }

    /// Create a new root/admin context
    pub fn root_admin(user_id: Option<String>, client_id: Option<String>) -> Self {
        Self {
            is_root_admin: true,
            user_id,
            client_id,
        }
    }
}
