//! Native client trait for inter-module communication
//!
//! This trait defines the API that other modules use to interact with settings service.
//! NO HTTP - direct function calls for performance.

use super::{error::SettingsError, model::{GtsType, Setting}};
use async_trait::async_trait;
use uuid::Uuid;

/// Settings service API for inter-module communication
#[async_trait]
pub trait SettingsApi: Send + Sync {
    // ===== Setting Operations =====

    /// Get all settings for a specific GTS type
    async fn get_settings_by_type(
        &self,
        setting_type: &str,
        tenant_id: Option<Uuid>,
    ) -> Result<Vec<Setting>, SettingsError>;

    /// Get a specific setting
    async fn get_setting(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
    ) -> Result<Setting, SettingsError>;

    /// Update or create a setting
    async fn upsert_setting(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
        data: serde_json::Value,
    ) -> Result<Setting, SettingsError>;

    /// Delete a setting (soft delete)
    async fn delete_setting(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
    ) -> Result<(), SettingsError>;

    /// Lock a setting for compliance mode
    async fn lock_setting(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
        read_only: bool,
    ) -> Result<(), SettingsError>;

    // ===== GTS Type Operations =====

    /// Register a new GTS type
    async fn register_gts_type(&self, gts_type: GtsType) -> Result<GtsType, SettingsError>;

    /// Get a GTS type by identifier
    async fn get_gts_type(&self, type_id: &str) -> Result<GtsType, SettingsError>;

    /// List all registered GTS types
    async fn list_gts_types(&self) -> Result<Vec<GtsType>, SettingsError>;

    /// Update a GTS type
    async fn update_gts_type(&self, gts_type: GtsType) -> Result<GtsType, SettingsError>;

    /// Delete a GTS type
    async fn delete_gts_type(&self, type_id: &str) -> Result<(), SettingsError>;
}
