/// Domain events for settings service
///
/// Events are published based on GTS traits configuration:
/// - Audit events: Published to `sm.events.audit` topic
/// - Notification events: Published to `sm.events.notification` topic
/// - Event targets: SELF (tenant only), SUBROOT (tenant + subtree), NONE (no event)

use crate::contract::model::{EventTarget, Setting};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Domain event types for settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum SettingEvent {
    /// Setting was created or updated
    SettingUpserted(SettingUpsertedEvent),
    /// Setting was deleted (soft delete)
    SettingDeleted(SettingDeletedEvent),
    /// Setting was locked for compliance
    SettingLocked(SettingLockedEvent),
}

/// Event data for setting upsert
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingUpsertedEvent {
    /// Setting type (GTS identifier)
    pub setting_type: String,
    /// Tenant ID
    pub tenant_id: Uuid,
    /// Domain object ID
    pub domain_object_id: String,
    /// Setting data (JSON)
    pub data: serde_json::Value,
    /// Whether this was a create or update
    pub is_new: bool,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// User who performed the action (if available)
    pub user_id: Option<Uuid>,
}

/// Event data for setting deletion
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingDeletedEvent {
    /// Setting type (GTS identifier)
    pub setting_type: String,
    /// Tenant ID
    pub tenant_id: Uuid,
    /// Domain object ID
    pub domain_object_id: String,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// User who performed the action (if available)
    pub user_id: Option<Uuid>,
}

/// Event data for setting lock
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingLockedEvent {
    /// Setting type (GTS identifier)
    pub setting_type: String,
    /// Tenant ID
    pub tenant_id: Uuid,
    /// Domain object ID
    pub domain_object_id: String,
    /// Whether setting is now read-only
    pub read_only: bool,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// User who performed the action (if available)
    pub user_id: Option<Uuid>,
}

/// Event publisher trait for publishing domain events
///
/// Implementations should handle:
/// - Publishing to appropriate topics (audit, notification)
/// - Handling event targets (SELF, SUBROOT, NONE)
/// - Error handling and retries
#[async_trait::async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish an audit event
    ///
    /// # Arguments
    /// * `event` - The setting event to publish
    /// * `target` - Event target (SELF, SUBROOT, NONE)
    /// * `tenant_id` - Tenant ID for the event
    async fn publish_audit(
        &self,
        event: SettingEvent,
        target: EventTarget,
        tenant_id: Uuid,
    ) -> anyhow::Result<()>;

    /// Publish a notification event
    ///
    /// # Arguments
    /// * `event` - The setting event to publish
    /// * `target` - Event target (SELF, SUBROOT, NONE)
    /// * `tenant_id` - Tenant ID for the event
    async fn publish_notification(
        &self,
        event: SettingEvent,
        target: EventTarget,
        tenant_id: Uuid,
    ) -> anyhow::Result<()>;
}

/// No-op event publisher for testing or when events are disabled
pub struct NoOpEventPublisher;

#[async_trait::async_trait]
impl EventPublisher for NoOpEventPublisher {
    async fn publish_audit(
        &self,
        _event: SettingEvent,
        _target: EventTarget,
        _tenant_id: Uuid,
    ) -> anyhow::Result<()> {
        // No-op: events are not published
        Ok(())
    }

    async fn publish_notification(
        &self,
        _event: SettingEvent,
        _target: EventTarget,
        _tenant_id: Uuid,
    ) -> anyhow::Result<()> {
        // No-op: events are not published
        Ok(())
    }
}

impl SettingEvent {
    /// Create a new SettingUpserted event
    pub fn upserted(
        setting: &Setting,
        is_new: bool,
        user_id: Option<Uuid>,
    ) -> Self {
        SettingEvent::SettingUpserted(SettingUpsertedEvent {
            setting_type: setting.r#type.clone(),
            tenant_id: setting.tenant_id,
            domain_object_id: setting.domain_object_id.clone(),
            data: setting.data.clone(),
            is_new,
            timestamp: Utc::now(),
            user_id,
        })
    }

    /// Create a new SettingDeleted event
    pub fn deleted(
        setting_type: String,
        tenant_id: Uuid,
        domain_object_id: String,
        user_id: Option<Uuid>,
    ) -> Self {
        SettingEvent::SettingDeleted(SettingDeletedEvent {
            setting_type,
            tenant_id,
            domain_object_id,
            timestamp: Utc::now(),
            user_id,
        })
    }

    /// Create a new SettingLocked event
    pub fn locked(
        setting_type: String,
        tenant_id: Uuid,
        domain_object_id: String,
        read_only: bool,
        user_id: Option<Uuid>,
    ) -> Self {
        SettingEvent::SettingLocked(SettingLockedEvent {
            setting_type,
            tenant_id,
            domain_object_id,
            read_only,
            timestamp: Utc::now(),
            user_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_upserted_event_creation() {
        let setting = Setting {
            r#type: "gts.a.p.sm.setting.v1.0~test.v1".to_string(),
            tenant_id: Uuid::new_v4(),
            domain_object_id: "generic".to_string(),
            data: serde_json::json!({"key": "value"}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let event = SettingEvent::upserted(&setting, true, None);

        match event {
            SettingEvent::SettingUpserted(e) => {
                assert_eq!(e.setting_type, setting.r#type);
                assert_eq!(e.tenant_id, setting.tenant_id);
                assert_eq!(e.domain_object_id, setting.domain_object_id);
                assert_eq!(e.is_new, true);
                assert!(e.user_id.is_none());
            }
            _ => panic!("Expected SettingUpserted event"),
        }
    }

    #[test]
    fn test_setting_deleted_event_creation() {
        let tenant_id = Uuid::new_v4();
        let event = SettingEvent::deleted(
            "gts.a.p.sm.setting.v1.0~test.v1".to_string(),
            tenant_id,
            "generic".to_string(),
            None,
        );

        match event {
            SettingEvent::SettingDeleted(e) => {
                assert_eq!(e.tenant_id, tenant_id);
                assert_eq!(e.domain_object_id, "generic");
                assert!(e.user_id.is_none());
            }
            _ => panic!("Expected SettingDeleted event"),
        }
    }

    #[tokio::test]
    async fn test_noop_event_publisher() {
        let publisher = NoOpEventPublisher;
        let tenant_id = Uuid::new_v4();
        let event = SettingEvent::deleted(
            "gts.a.p.sm.setting.v1.0~test.v1".to_string(),
            tenant_id,
            "generic".to_string(),
            None,
        );

        // Should not error
        let result = publisher
            .publish_audit(event.clone(), EventTarget::Self_, tenant_id)
            .await;
        assert!(result.is_ok());

        let result = publisher
            .publish_notification(event, EventTarget::Self_, tenant_id)
            .await;
        assert!(result.is_ok());
    }
}
