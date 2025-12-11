//! Native client implementation - wraps domain service for in-process calls

use crate::contract::{GtsType, Setting, SettingsApi, SettingsError};
use crate::domain::Service;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

/// Native client implementation that directly calls the domain service
/// 
/// This client is used for in-process communication without HTTP overhead.
/// It's registered in the ClientHub for dependency injection.
#[derive(Clone)]
pub struct NativeClient {
    service: Arc<Service>,
}

impl NativeClient {
    /// Create a new native client
    pub fn new(service: Arc<Service>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl SettingsApi for NativeClient {
    async fn get_setting(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
    ) -> Result<Setting, SettingsError> {
        self.service
            .get_setting(setting_type, tenant_id, domain_object_id)
            .await
    }

    async fn upsert_setting(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
        data: serde_json::Value,
    ) -> Result<Setting, SettingsError> {
        self.service
            .upsert_setting(setting_type, tenant_id, domain_object_id, data)
            .await
    }

    async fn delete_setting(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
    ) -> Result<(), SettingsError> {
        self.service
            .delete_setting(setting_type, tenant_id, domain_object_id)
            .await
    }

    async fn get_settings_by_type(
        &self,
        setting_type: &str,
        tenant_id: Option<Uuid>,
    ) -> Result<Vec<Setting>, SettingsError> {
        self.service
            .get_settings_by_type(setting_type, tenant_id)
            .await
    }

    async fn lock_setting(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
        read_only: bool,
    ) -> Result<(), SettingsError> {
        self.service
            .lock_setting(setting_type, tenant_id, domain_object_id, read_only)
            .await
    }

    async fn register_gts_type(&self, gts_type: GtsType) -> Result<GtsType, SettingsError> {
        self.service.register_gts_type(gts_type).await
    }

    async fn get_gts_type(&self, type_id: &str) -> Result<GtsType, SettingsError> {
        self.service.get_gts_type(type_id).await
    }

    async fn list_gts_types(&self) -> Result<Vec<GtsType>, SettingsError> {
        self.service.list_gts_types().await
    }

    async fn update_gts_type(&self, gts_type: GtsType) -> Result<GtsType, SettingsError> {
        self.service.update_gts_type(gts_type).await
    }

    async fn delete_gts_type(&self, type_id: &str) -> Result<(), SettingsError> {
        self.service.delete_gts_type(type_id).await
    }
}
