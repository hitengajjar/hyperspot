//! Repository traits for data access
//!
//! These traits define the interface for data access operations.
//! Implementations are in infra/storage/repositories.rs

use crate::contract::{GtsType, Setting};
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

/// Repository for GTS type definitions
#[async_trait]
pub trait GtsTypeRepository: Send + Sync {
    /// Create a new GTS type
    async fn create(&self, gts_type: &GtsType) -> Result<GtsType>;

    /// Find a GTS type by identifier
    async fn find_by_type(&self, type_id: &str) -> Result<Option<GtsType>>;

    /// List all GTS types
    async fn list_all(&self) -> Result<Vec<GtsType>>;

    /// Update a GTS type
    async fn update(&self, gts_type: &GtsType) -> Result<GtsType>;

    /// Delete a GTS type
    async fn delete(&self, type_id: &str) -> Result<()>;

    /// Check if a GTS type exists
    async fn exists(&self, type_id: &str) -> Result<bool>;
}

/// Repository for settings
#[async_trait]
pub trait SettingsRepository: Send + Sync {
    /// Create or update a setting
    async fn upsert(&self, setting: &Setting) -> Result<Setting>;

    /// Find a setting by composite key
    async fn find_by_key(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
    ) -> Result<Option<Setting>>;

    /// Find all settings for a GTS type
    async fn find_by_type(
        &self,
        setting_type: &str,
        tenant_id: Option<Uuid>,
    ) -> Result<Vec<Setting>>;

    /// Find all settings for a tenant
    async fn find_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<Setting>>;

    /// Find all settings for a domain object
    async fn find_by_domain_object(
        &self,
        domain_object_id: &str,
    ) -> Result<Vec<Setting>>;

    /// Soft delete a setting
    async fn soft_delete(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
    ) -> Result<()>;

    /// Hard delete a setting
    async fn hard_delete(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
    ) -> Result<()>;

    /// List all settings with pagination
    async fn list_all(&self, limit: u64, offset: u64) -> Result<Vec<Setting>>;
}
