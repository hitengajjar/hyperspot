//! Entity to model mappers
//!
//! Conversions between SeaORM entities and contract models

use crate::contract::{
    GtsTraits, GtsType, DomainType, EventConfig, EventTarget, Setting, SettingOptions,
};
use super::entity;

// ===== Setting Conversions =====

impl From<entity::Model> for Setting {
    fn from(entity: entity::Model) -> Self {
        Self {
            r#type: entity.r#type,
            tenant_id: entity.tenant_id,
            domain_object_id: entity.domain_object_id,
            data: entity.data,
            created_at: entity.created_at.into(),
            updated_at: entity.updated_at.into(),
            deleted_at: entity.deleted_at.map(|dt| dt.into()),
        }
    }
}

impl From<&Setting> for entity::ActiveModel {
    fn from(model: &Setting) -> Self {
        use sea_orm::ActiveValue::*;
        
        Self {
            r#type: Set(model.r#type.clone()),
            tenant_id: Set(model.tenant_id),
            domain_object_id: Set(model.domain_object_id.clone()),
            data: Set(model.data.clone()),
            created_at: Set(model.created_at.into()),
            updated_at: Set(model.updated_at.into()),
            deleted_at: Set(model.deleted_at.map(|dt| dt.into())),
        }
    }
}

// ===== GTS Type Conversions =====

impl TryFrom<entity::gts_type::Model> for GtsType {
    type Error = anyhow::Error;

    fn try_from(entity: entity::gts_type::Model) -> Result<Self, Self::Error> {
        let traits: GtsTraitsJson = serde_json::from_value(entity.traits)?;
        
        Ok(Self {
            r#type: entity.r#type,
            traits: traits.into(),
            schema: entity.schema,
            created_at: entity.created_at.into(),
            updated_at: entity.updated_at.into(),
        })
    }
}

impl From<&GtsType> for entity::gts_type::ActiveModel {
    fn from(model: &GtsType) -> Self {
        use sea_orm::ActiveValue::*;
        
        let traits_json: GtsTraitsJson = (&model.traits).into();
        
        Self {
            r#type: Set(model.r#type.clone()),
            traits: Set(serde_json::to_value(traits_json).expect("Failed to serialize traits")),
            schema: Set(model.schema.clone()),
            created_at: Set(model.created_at.into()),
            updated_at: Set(model.updated_at.into()),
        }
    }
}

// ===== JSON Serialization Helpers =====

/// JSON representation of GTS traits for database storage
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct GtsTraitsJson {
    domain_type: String,
    events: EventConfigJson,
    options: SettingOptionsJson,
    operation: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct EventConfigJson {
    audit: String,
    notification: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SettingOptionsJson {
    retention_period: u32,
    is_value_inheritable: bool,
    is_value_overwritable: bool,
    is_barrier_inheritance: bool,
    enable_generic: bool,
    is_mfa_required: bool,
}

impl From<GtsTraitsJson> for GtsTraits {
    fn from(json: GtsTraitsJson) -> Self {
        Self {
            domain_type: match json.domain_type.as_str() {
                "TENANT" => DomainType::Tenant,
                "STORAGE" => DomainType::Storage,
                "USER" => DomainType::User,
                "AGENT" => DomainType::Agent,
                "APPLICATION" => DomainType::Application,
                "BRAND" => DomainType::Brand,
                "RESOURCE" => DomainType::Resource,
                "GLOBAL" => DomainType::Global,
                _ => DomainType::Tenant, // Default
            },
            events: EventConfig {
                audit: parse_event_target(&json.events.audit),
                notification: parse_event_target(&json.events.notification),
            },
            options: SettingOptions {
                retention_period: json.options.retention_period,
                is_value_inheritable: json.options.is_value_inheritable,
                is_value_overwritable: json.options.is_value_overwritable,
                is_barrier_inheritance: json.options.is_barrier_inheritance,
                enable_generic: json.options.enable_generic,
                is_mfa_required: json.options.is_mfa_required,
            },
            operation: json.operation,
        }
    }
}

impl From<&GtsTraits> for GtsTraitsJson {
    fn from(traits: &GtsTraits) -> Self {
        Self {
            domain_type: match traits.domain_type {
                DomainType::Tenant => "TENANT",
                DomainType::Storage => "STORAGE",
                DomainType::User => "USER",
                DomainType::Agent => "AGENT",
                DomainType::Application => "APPLICATION",
                DomainType::Brand => "BRAND",
                DomainType::Resource => "RESOURCE",
                DomainType::Global => "GLOBAL",
            }
            .to_string(),
            events: EventConfigJson {
                audit: format_event_target(traits.events.audit),
                notification: format_event_target(traits.events.notification),
            },
            options: SettingOptionsJson {
                retention_period: traits.options.retention_period,
                is_value_inheritable: traits.options.is_value_inheritable,
                is_value_overwritable: traits.options.is_value_overwritable,
                is_barrier_inheritance: traits.options.is_barrier_inheritance,
                enable_generic: traits.options.enable_generic,
                is_mfa_required: traits.options.is_mfa_required,
            },
            operation: traits.operation.clone(),
        }
    }
}

fn parse_event_target(s: &str) -> EventTarget {
    match s {
        "SELF" => EventTarget::Self_,
        "SUBROOT" => EventTarget::Subroot,
        "NONE" => EventTarget::None,
        _ => EventTarget::None, // Default
    }
}

fn format_event_target(target: EventTarget) -> String {
    match target {
        EventTarget::Self_ => "SELF",
        EventTarget::Subroot => "SUBROOT",
        EventTarget::None => "NONE",
    }
    .to_string()
}
