//! Domain service - business logic orchestration

use crate::contract::{AuthContext, GtsType, Setting, SettingsError};
use super::events::{EventPublisher, SettingEvent};
use super::repository::{GtsTypeRepository, SettingsRepository};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Lock key for identifying locked settings
type LockKey = (String, Uuid, String); // (setting_type, tenant_id, domain_object_id)

/// Domain service for settings management
pub struct Service {
    settings_repo: Arc<dyn SettingsRepository>,
    gts_type_repo: Arc<dyn GtsTypeRepository>,
    event_publisher: Arc<dyn EventPublisher>,
    /// In-memory lock storage (setting_type, tenant_id, domain_object_id) -> read_only
    locks: Arc<RwLock<HashMap<LockKey, bool>>>,
}

impl Service {
    /// Create a new service instance
    pub fn new(
        settings_repo: Arc<dyn SettingsRepository>,
        gts_type_repo: Arc<dyn GtsTypeRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            settings_repo,
            gts_type_repo,
            event_publisher,
            locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // ===== Setting Operations =====

    /// Get all settings for a specific GTS type
    pub async fn get_settings_by_type(
        &self,
        setting_type: &str,
        tenant_id: Option<Uuid>,
    ) -> Result<Vec<Setting>, SettingsError> {
        // Validate GTS type exists
        self.validate_gts_type_exists(setting_type).await?;

        self.settings_repo
            .find_by_type(setting_type, tenant_id)
            .await
            .map_err(|_| SettingsError::Internal)
    }

    /// Get a specific setting
    pub async fn get_setting(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
    ) -> Result<Setting, SettingsError> {
        // Validate GTS type exists
        self.validate_gts_type_exists(setting_type).await?;

        self.settings_repo
            .find_by_key(setting_type, tenant_id, domain_object_id)
            .await
            .map_err(|_| SettingsError::Internal)?
            .ok_or_else(|| SettingsError::NotFound {
                resource: "setting".to_string(),
                id: format!("{}/{}/{}", setting_type, tenant_id, domain_object_id),
            })
    }

    /// Update or create a setting (backward compatible - non-admin context)
    pub async fn upsert_setting(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
        data: serde_json::Value,
    ) -> Result<Setting, SettingsError> {
        // Call the auth-aware version with default non-admin context
        self.upsert_setting_with_auth(
            setting_type,
            tenant_id,
            domain_object_id,
            data,
            &AuthContext::non_admin(),
        )
        .await
    }

    /// Update or create a setting with authentication context (root/admin override support)
    pub async fn upsert_setting_with_auth(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
        data: serde_json::Value,
        auth_context: &AuthContext,
    ) -> Result<Setting, SettingsError> {
        // Validate GTS type exists and get traits
        let gts_type = self.get_gts_type(setting_type).await?;

        // Validate data against JSON schema if present
        if let Some(schema) = &gts_type.schema {
            crate::domain::validation::validate_against_schema(&data, schema)?;
        }

        // Check if setting is locked (compliance mode)
        // Root/admin can modify locked settings
        let lock_key = (setting_type.to_string(), tenant_id, domain_object_id.to_string());
        if !auth_context.is_root_admin {
            if let Some(&read_only) = self.locks.read().get(&lock_key) {
                if read_only {
                    return Err(SettingsError::Conflict {
                        reason: format!(
                            "Setting is locked for compliance: {}/{}/{}",
                            setting_type, tenant_id, domain_object_id
                        ),
                    });
                }
            }
        }

        // Check is_value_overwritable constraint
        // Root/admin can bypass this constraint
        if !auth_context.is_root_admin && !gts_type.traits.options.is_value_overwritable {
            // Check if a parent setting exists (simplified check - in production, would traverse hierarchy)
            // For now, we'll just check if any setting with this type exists for a different tenant
            // This is a placeholder for proper hierarchy traversal
            let existing_settings = self
                .settings_repo
                .find_by_type(setting_type, None)
                .await
                .map_err(|_| SettingsError::Internal)?;

            // If there are existing settings for other tenants, this might be an override attempt
            let has_parent_setting = existing_settings
                .iter()
                .any(|s| s.tenant_id != tenant_id && s.domain_object_id == domain_object_id);

            if has_parent_setting {
                return Err(SettingsError::Conflict {
                    reason: format!(
                        "Setting is not overwritable (is_value_overwritable=false): {}/{}/{}",
                        setting_type, tenant_id, domain_object_id
                    ),
                });
            }
        }

        let setting = Setting {
            r#type: setting_type.to_string(),
            tenant_id,
            domain_object_id: domain_object_id.to_string(),
            data,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        };

        // Check if setting already exists to determine if this is create or update
        let is_new = self
            .settings_repo
            .find_by_key(setting_type, tenant_id, domain_object_id)
            .await
            .is_err();

        let result = self
            .settings_repo
            .upsert(&setting)
            .await
            .map_err(|_| SettingsError::Internal)?;

        // Publish events based on GTS traits
        if let Ok(Some(gts_type)) = self.gts_type_repo.find_by_type(setting_type).await {
            // TODO: Convert string user_id/client_id to UUID for event publishing
            // For now, pass None - will be enhanced in Phase 3 with proper user context
            let event = SettingEvent::upserted(&result, is_new, None);

            // Log root/admin override for audit trail
            if auth_context.is_root_admin {
                let user_context = auth_context.user_id.clone()
                    .or_else(|| auth_context.client_id.clone())
                    .unwrap_or_else(|| "unknown".to_string());
                
                eprintln!(
                    "🔐 ROOT/ADMIN OVERRIDE: User/Client {} modified setting {}/{}/{} (is_value_overwritable={}, locked={})",
                    user_context,
                    setting_type,
                    tenant_id,
                    domain_object_id,
                    gts_type.traits.options.is_value_overwritable,
                    self.is_locked(setting_type, tenant_id, domain_object_id)
                );
            }

            // Publish audit event if configured
            if let Err(e) = self
                .event_publisher
                .publish_audit(event.clone(), gts_type.traits.events.audit, tenant_id)
                .await
            {
                // Log error but don't fail the operation
                eprintln!("Failed to publish audit event: {}", e);
            }

            // Publish notification event if configured
            if let Err(e) = self
                .event_publisher
                .publish_notification(event, gts_type.traits.events.notification, tenant_id)
                .await
            {
                // Log error but don't fail the operation
                eprintln!("Failed to publish notification event: {}", e);
            }
        }

        Ok(result)
    }

    /// Delete a setting (soft delete)
    pub async fn delete_setting(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
    ) -> Result<(), SettingsError> {
        // Validate GTS type exists
        self.validate_gts_type_exists(setting_type).await?;

        // Check if setting is locked (compliance mode)
        let lock_key = (setting_type.to_string(), tenant_id, domain_object_id.to_string());
        if let Some(&read_only) = self.locks.read().get(&lock_key) {
            if read_only {
                return Err(SettingsError::Conflict {
                    reason: format!(
                        "Setting is locked for compliance: {}/{}/{}",
                        setting_type, tenant_id, domain_object_id
                    ),
                });
            }
        }

        self.settings_repo
            .soft_delete(setting_type, tenant_id, domain_object_id)
            .await
            .map_err(|_| SettingsError::Internal)?;

        // Publish events based on GTS traits
        if let Ok(Some(gts_type)) = self.gts_type_repo.find_by_type(setting_type).await {
            let event = SettingEvent::deleted(
                setting_type.to_string(),
                tenant_id,
                domain_object_id.to_string(),
                None,
            );

            // Publish audit event if configured
            if let Err(e) = self
                .event_publisher
                .publish_audit(event.clone(), gts_type.traits.events.audit, tenant_id)
                .await
            {
                // Log error but don't fail the operation
                eprintln!("Failed to publish audit event: {}", e);
            }

            // Publish notification event if configured
            if let Err(e) = self
                .event_publisher
                .publish_notification(event, gts_type.traits.events.notification, tenant_id)
                .await
            {
                // Log error but don't fail the operation
                eprintln!("Failed to publish notification event: {}", e);
            }
        }

        Ok(())
    }

    /// Lock a setting for compliance mode
    pub async fn lock_setting(
        &self,
        setting_type: &str,
        tenant_id: Uuid,
        domain_object_id: &str,
        read_only: bool,
    ) -> Result<(), SettingsError> {
        // Validate GTS type exists
        self.validate_gts_type_exists(setting_type).await?;

        // Verify setting exists
        self.settings_repo
            .find_by_key(setting_type, tenant_id, domain_object_id)
            .await
            .map_err(|_| SettingsError::NotFound {
                resource: "setting".to_string(),
                id: format!("{}/{}/{}", setting_type, tenant_id, domain_object_id),
            })?;

        // Store lock
        let lock_key = (setting_type.to_string(), tenant_id, domain_object_id.to_string());
        self.locks.write().insert(lock_key, read_only);

        // Publish lock event
        if let Ok(Some(gts_type)) = self.gts_type_repo.find_by_type(setting_type).await {
            let event = SettingEvent::locked(
                setting_type.to_string(),
                tenant_id,
                domain_object_id.to_string(),
                read_only,
                None,
            );

            // Publish audit event
            let _ = self
                .event_publisher
                .publish_audit(event.clone(), gts_type.traits.events.audit, tenant_id)
                .await;

            // Publish notification event
            let _ = self
                .event_publisher
                .publish_notification(event, gts_type.traits.events.notification, tenant_id)
                .await;
        }

        Ok(())
    }

    /// Check if a setting is locked
    pub fn is_locked(&self, setting_type: &str, tenant_id: Uuid, domain_object_id: &str) -> bool {
        let lock_key = (setting_type.to_string(), tenant_id, domain_object_id.to_string());
        self.locks.read().get(&lock_key).copied().unwrap_or(false)
    }

    /// Unlock a setting
    pub fn unlock_setting(&self, setting_type: &str, tenant_id: Uuid, domain_object_id: &str) {
        let lock_key = (setting_type.to_string(), tenant_id, domain_object_id.to_string());
        self.locks.write().remove(&lock_key);
    }

    /// Clean up expired soft-deleted settings based on retention period
    pub async fn enforce_retention(&self) -> Result<usize, SettingsError> {
        // Get all settings
        let all_settings = self
            .settings_repo
            .list_all(u64::MAX, 0)
            .await
            .map_err(|_| SettingsError::Internal)?;

        let mut deleted_count = 0;
        let now = chrono::Utc::now();

        for setting in all_settings {
            // Skip if not soft-deleted
            if setting.deleted_at.is_none() {
                continue;
            }

            // Get GTS type to check retention period
            if let Ok(Some(gts_type)) = self.gts_type_repo.find_by_type(&setting.r#type).await {
                let retention_days = gts_type.traits.options.retention_period as i64;
                let deleted_at = setting.deleted_at.unwrap();
                let retention_deadline = deleted_at + chrono::Duration::days(retention_days);

                // If retention period has expired, hard delete
                if now > retention_deadline {
                    if self
                        .settings_repo
                        .hard_delete(
                            &setting.r#type,
                            setting.tenant_id,
                            &setting.domain_object_id,
                        )
                        .await
                        .is_ok()
                    {
                        deleted_count += 1;
                    }
                }
            }
        }

        Ok(deleted_count)
    }

    // ===== GTS Type Operations =====

    /// Register a new GTS type
    pub async fn register_gts_type(&self, gts_type: GtsType) -> Result<GtsType, SettingsError> {
        // Validate GTS format
        self.validate_gts_format(&gts_type.r#type)?;

        // Check if type already exists
        if self
            .gts_type_repo
            .exists(&gts_type.r#type)
            .await
            .map_err(|_| SettingsError::Internal)?
        {
            return Err(SettingsError::Conflict {
                reason: format!("GTS type already exists: {}", gts_type.r#type),
            });
        }

        self.gts_type_repo
            .create(&gts_type)
            .await
            .map_err(|_| SettingsError::Internal)
    }

    /// Get a GTS type by identifier
    pub async fn get_gts_type(&self, type_id: &str) -> Result<GtsType, SettingsError> {
        self.gts_type_repo
            .find_by_type(type_id)
            .await
            .map_err(|_| SettingsError::Internal)?
            .ok_or_else(|| SettingsError::TypeNotRegistered {
                gts_type: type_id.to_string(),
            })
    }

    /// List all registered GTS types
    pub async fn list_gts_types(&self) -> Result<Vec<GtsType>, SettingsError> {
        self.gts_type_repo
            .list_all()
            .await
            .map_err(|_| SettingsError::Internal)
    }

    /// Update a GTS type
    pub async fn update_gts_type(&self, gts_type: GtsType) -> Result<GtsType, SettingsError> {
        // Validate GTS format
        self.validate_gts_format(&gts_type.r#type)?;

        // Check if type exists
        self.validate_gts_type_exists(&gts_type.r#type).await?;

        self.gts_type_repo
            .update(&gts_type)
            .await
            .map_err(|_| SettingsError::Internal)
    }

    /// Delete a GTS type
    pub async fn delete_gts_type(&self, type_id: &str) -> Result<(), SettingsError> {
        // Check if type exists
        self.validate_gts_type_exists(type_id).await?;

        // TODO: Check if there are active settings using this type
        // Should return Conflict error if settings exist

        self.gts_type_repo
            .delete(type_id)
            .await
            .map_err(|_| SettingsError::Internal)
    }

    // ===== Helper Methods =====

    /// Validate that a GTS type exists
    async fn validate_gts_type_exists(&self, type_id: &str) -> Result<(), SettingsError> {
        if !self
            .gts_type_repo
            .exists(type_id)
            .await
            .map_err(|_| SettingsError::Internal)?
        {
            return Err(SettingsError::TypeNotRegistered {
                gts_type: type_id.to_string(),
            });
        }
        Ok(())
    }

    /// Validate GTS format
    fn validate_gts_format(&self, gts: &str) -> Result<(), SettingsError> {
        // Basic GTS format validation
        // Expected format: gts.a.p.sm.setting.v1.0~vendor.app.feature.v1.0
        if !gts.starts_with("gts.") {
            return Err(SettingsError::InvalidGtsFormat {
                gts: gts.to_string(),
                details: "GTS must start with 'gts.'".to_string(),
            });
        }

        if !gts.contains('~') {
            return Err(SettingsError::InvalidGtsFormat {
                gts: gts.to_string(),
                details: "GTS must contain '~' separator".to_string(),
            });
        }

        // TODO: More comprehensive GTS validation (Phase 2)
        Ok(())
    }
}
