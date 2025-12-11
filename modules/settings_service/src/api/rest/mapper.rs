//! Mapper implementations for converting between DTOs and contract models
//!
//! This module contains all From/Into implementations for bidirectional
//! conversion between REST DTOs and transport-agnostic contract models.

use super::dto::*;
use crate::contract;

// ===== Setting conversions =====

impl From<contract::Setting> for SettingDto {
    fn from(setting: contract::Setting) -> Self {
        Self {
            r#type: setting.r#type,
            tenant_id: setting.tenant_id,
            domain_object_id: setting.domain_object_id,
            data: setting.data,
            created_at: setting.created_at,
            updated_at: setting.updated_at,
        }
    }
}

// ===== GTS Type conversions =====

impl From<contract::GtsType> for GtsTypeDto {
    fn from(gts_type: contract::GtsType) -> Self {
        Self {
            r#type: gts_type.r#type,
            traits: gts_type.traits.into(),
            schema: gts_type.schema,
            created_at: gts_type.created_at,
            updated_at: gts_type.updated_at,
        }
    }
}

impl From<UpsertGtsTypeRequest> for contract::GtsType {
    fn from(req: UpsertGtsTypeRequest) -> Self {
        Self {
            r#type: req.r#type,
            traits: req.traits.into(),
            schema: req.schema,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

// ===== GTS Traits conversions =====

impl From<contract::GtsTraits> for GtsTraitsDto {
    fn from(traits: contract::GtsTraits) -> Self {
        Self {
            domain_type: format!("{:?}", traits.domain_type).to_uppercase(),
            events: traits.events.into(),
            options: traits.options.into(),
            operation: traits.operation,
        }
    }
}

impl From<GtsTraitsDto> for contract::GtsTraits {
    fn from(dto: GtsTraitsDto) -> Self {
        use contract::{DomainType, EventConfig, EventTarget, SettingOptions};
        
        Self {
            domain_type: match dto.domain_type.as_str() {
                "TENANT" => DomainType::Tenant,
                "STORAGE" => DomainType::Storage,
                "USER" => DomainType::User,
                "AGENT" => DomainType::Agent,
                "APPLICATION" => DomainType::Application,
                "BRAND" => DomainType::Brand,
                "RESOURCE" => DomainType::Resource,
                "GLOBAL" => DomainType::Global,
                _ => DomainType::Tenant,
            },
            events: EventConfig {
                audit: match dto.events.audit.as_str() {
                    "SELF" => EventTarget::Self_,
                    "SUBROOT" => EventTarget::Subroot,
                    _ => EventTarget::None,
                },
                notification: match dto.events.notification.as_str() {
                    "SELF" => EventTarget::Self_,
                    "SUBROOT" => EventTarget::Subroot,
                    _ => EventTarget::None,
                },
            },
            options: SettingOptions {
                retention_period: dto.options.retention_period,
                is_value_inheritable: dto.options.is_value_inheritable,
                is_value_overwritable: dto.options.is_value_overwritable,
                is_barrier_inheritance: dto.options.is_barrier_inheritance,
                enable_generic: dto.options.enable_generic,
                is_mfa_required: dto.options.is_mfa_required,
            },
            operation: dto.operation,
        }
    }
}

// ===== Event Config conversions =====

impl From<contract::EventConfig> for EventConfigDto {
    fn from(config: contract::EventConfig) -> Self {
        Self {
            audit: match config.audit {
                contract::EventTarget::Self_ => "SELF",
                contract::EventTarget::Subroot => "SUBROOT",
                contract::EventTarget::None => "NONE",
            }
            .to_string(),
            notification: match config.notification {
                contract::EventTarget::Self_ => "SELF",
                contract::EventTarget::Subroot => "SUBROOT",
                contract::EventTarget::None => "NONE",
            }
            .to_string(),
        }
    }
}

// ===== Setting Options conversions =====

impl From<contract::SettingOptions> for SettingOptionsDto {
    fn from(options: contract::SettingOptions) -> Self {
        Self {
            retention_period: options.retention_period,
            is_value_inheritable: options.is_value_inheritable,
            is_value_overwritable: options.is_value_overwritable,
            is_barrier_inheritance: options.is_barrier_inheritance,
            enable_generic: options.enable_generic,
            is_mfa_required: options.is_mfa_required,
        }
    }
}
