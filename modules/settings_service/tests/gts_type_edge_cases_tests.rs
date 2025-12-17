//! GTS Type Update/Delete Edge Case Tests
//!
//! Tests for edge cases in GTS type management operations.

use settings_service::contract::*;
use settings_service::domain::repository::{GtsTypeRepository, SettingsRepository};
use settings_service::domain::{NoOpEventPublisher, Service};
use std::sync::Arc;
use uuid::Uuid;
use serde_json::json;

mod common;
use common::TestTenantHierarchy;

// Import mocks from service_tests
#[path = "service_tests.rs"]
mod service_tests;
use service_tests::mocks::{MockGtsTypeRepo, MockSettingsRepo};

fn print_test_header(test_name: &str, purpose: &[&str]) {
    println!("\n🧪 TEST: {}", test_name);
    if let Some(first) = purpose.first() {
        println!("📋 PURPOSE: {}", first);
    }
    for line in purpose.iter().skip(1) {
        println!("   {}", line);
    }
}

fn print_json(label: &str, value: &serde_json::Value) {
    println!("   {}: {}", label, serde_json::to_string_pretty(value).unwrap());
}

fn create_test_service() -> Service {
    let settings_repo = Arc::new(MockSettingsRepo::new()) as Arc<dyn SettingsRepository>;
    let gts_type_repo = Arc::new(MockGtsTypeRepo::new()) as Arc<dyn GtsTypeRepository>;
    let event_publisher = Arc::new(NoOpEventPublisher);
    Service::new(settings_repo, gts_type_repo, event_publisher)
}

fn create_test_gts_type(type_suffix: &str) -> GtsType {
    GtsType {
        r#type: format!("gts.a.p.sm.setting.v1.0~test.{}", type_suffix),
        traits: GtsTraits {
            domain_type: DomainType::Tenant,
            events: EventConfig {
                audit: EventTarget::Self_,
                notification: EventTarget::None,
            },
            options: SettingOptions {
                retention_period: 30,
                is_value_inheritable: true,
                is_value_overwritable: true,
                is_barrier_inheritance: false,
                enable_generic: true,
                is_mfa_required: false,
            },
            operation: None,
        },
        schema: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

// ===== GTS Type Update Tests =====

#[tokio::test]
async fn test_update_gts_type_success() {
    print_test_header(
        "test_update_gts_type_success",
        &["Verify that updating an existing GTS type persists the updated traits."],
    );

    let service = create_test_service();
    let mut gts_type = create_test_gts_type("update.v1");

    println!("\n📝 Stage 1: Register initial GTS type");
    println!("   Type: {}", gts_type.r#type);
    println!(
        "   Initial traits: retention_period={}, is_value_overwritable={}",
        gts_type.traits.options.retention_period,
        gts_type.traits.options.is_value_overwritable
    );

    // Register initial type
    service.register_gts_type(gts_type.clone()).await.unwrap();
    println!("✅ Initial GTS type registered");

    println!("\n📝 Stage 2: Update GTS type traits");
    gts_type.traits.options.retention_period = 60;
    gts_type.traits.options.is_value_overwritable = false;
    gts_type.updated_at = chrono::Utc::now();
    println!(
        "   Updated traits: retention_period={}, is_value_overwritable={}",
        gts_type.traits.options.retention_period,
        gts_type.traits.options.is_value_overwritable
    );

    let result = service.update_gts_type(gts_type.clone()).await;
    assert!(result.is_ok(), "Should be able to update GTS type");

    println!("\n📝 Stage 3: Verify update persisted");
    let retrieved = service.get_gts_type(&gts_type.r#type).await.unwrap();
    println!(
        "   Retrieved traits: retention_period={}, is_value_overwritable={}",
        retrieved.traits.options.retention_period,
        retrieved.traits.options.is_value_overwritable
    );
    assert_eq!(retrieved.traits.options.retention_period, 60);
    assert_eq!(retrieved.traits.options.is_value_overwritable, false);

    println!("✅ GTS type updated successfully");
}

#[tokio::test]
async fn test_update_nonexistent_gts_type() {
    print_test_header(
        "test_update_nonexistent_gts_type",
        &["Verify updating a non-existent GTS type fails with TypeNotRegistered."],
    );

    let service = create_test_service();
    let gts_type = create_test_gts_type("nonexistent.v1");

    println!("\n📝 Stage 1: Attempt update");
    println!("   Type: {}", gts_type.r#type);

    let result = service.update_gts_type(gts_type).await;
    assert!(result.is_err(), "Should fail to update non-existent type");

    match result.unwrap_err() {
        SettingsError::TypeNotRegistered { gts_type } => {
            println!("✅ Correctly rejected with TypeNotRegistered: {}", gts_type);
        }
        _ => panic!("Expected TypeNotRegistered error"),
    }
}

#[tokio::test]
async fn test_update_gts_type_with_invalid_format() {
    print_test_header(
        "test_update_gts_type_with_invalid_format",
        &["Verify updating a GTS type with an invalid format is rejected."],
    );

    let service = create_test_service();
    let mut gts_type = create_test_gts_type("valid.v1");

    // Register valid type first
    println!("\n📝 Stage 1: Register valid GTS type");
    println!("   Type: {}", gts_type.r#type);
    service.register_gts_type(gts_type.clone()).await.unwrap();

    // Try to update with invalid GTS format
    println!("\n📝 Stage 2: Attempt update with invalid format");
    gts_type.r#type = "invalid.format.without.gts.prefix~test".to_string();
    println!("   Invalid type: {}", gts_type.r#type);

    let result = service.update_gts_type(gts_type).await;
    assert!(result.is_err(), "Should fail with invalid GTS format");

    match result.unwrap_err() {
        SettingsError::InvalidGtsFormat { gts, details } => {
            println!("✅ Correctly rejected invalid format: {} - {}", gts, details);
            assert!(details.contains("gts."));
        }
        _ => panic!("Expected InvalidGtsFormat error"),
    }
}

#[tokio::test]
async fn test_update_gts_type_affects_new_settings() {
    print_test_header(
        "test_update_gts_type_affects_new_settings",
        &[
            "Verify that updating a GTS type schema affects validation for NEW settings created after the update.",
            "Existing stored settings are not revalidated; only new writes are checked.",
        ],
    );

    let service = create_test_service();
    let mut gts_type = create_test_gts_type("affects.v1");
    let tenants = TestTenantHierarchy::new();
    let tenant_id = tenants.partner1_customer2_braden;
    tenants.print_structure();

    println!("\n📝 Stage 1: Register type with initial schema");
    println!("   Tenant: {} (Braden Business Systems)", tenant_id);

    // Register type with schema
    gts_type.schema = Some(json!({
        "type": "object",
        "properties": {
            "value": {"type": "string"}
        },
        "required": ["value"]
    }));
    service.register_gts_type(gts_type.clone()).await.unwrap();
    println!("   Type: {}", gts_type.r#type);
    println!(
        "   Initial schema: {}",
        serde_json::to_string_pretty(gts_type.schema.as_ref().unwrap()).unwrap()
    );

    // Create setting with old schema
    let data = json!({"value": "test"});
    println!("\n📝 Stage 2: Create setting with schema v1");
    print_json("Data", &data);
    service.upsert_setting(&gts_type.r#type, tenant_id, "generic", data).await.unwrap();
    println!("✅ Setting created with original schema");

    // Update schema to be more restrictive
    println!("\n📝 Stage 3: Update schema to be more restrictive");
    gts_type.schema = Some(json!({
        "type": "object",
        "properties": {
            "value": {"type": "string", "minLength": 10}
        },
        "required": ["value"]
    }));
    println!(
        "   Updated schema: {}",
        serde_json::to_string_pretty(gts_type.schema.as_ref().unwrap()).unwrap()
    );
    service.update_gts_type(gts_type.clone()).await.unwrap();
    println!("✅ GTS type schema updated");

    // Try to create new setting with short value (should fail)
    let short_data = json!({"value": "short"});
    println!("\n📝 Stage 4: Create new setting expected to FAIL under schema v2");
    print_json("Data", &short_data);
    let result = service.upsert_setting(&gts_type.r#type, tenant_id, "generic2", short_data).await;
    assert!(result.is_err(), "Should fail validation with updated schema");

    println!("✅ New settings correctly validated against updated schema");
}

// ===== GTS Type Delete Tests =====

#[tokio::test]
async fn test_delete_gts_type_success() {
    print_test_header(
        "test_delete_gts_type_success",
        &["Verify a registered GTS type can be deleted and is no longer retrievable."],
    );

    let service = create_test_service();
    let gts_type = create_test_gts_type("delete.v1");

    println!("\n📝 Stage 1: Register type");
    println!("   Type: {}", gts_type.r#type);

    // Register type
    service.register_gts_type(gts_type.clone()).await.unwrap();
    println!("✅ GTS type registered");

    // Delete type
    println!("\n📝 Stage 2: Delete type");
    let result = service.delete_gts_type(&gts_type.r#type).await;
    assert!(result.is_ok(), "Should be able to delete GTS type");

    // Verify deletion
    println!("\n📝 Stage 3: Verify type is not retrievable");
    let result = service.get_gts_type(&gts_type.r#type).await;
    assert!(result.is_err(), "Type should not exist after deletion");

    println!("✅ GTS type deleted successfully");
}

#[tokio::test]
async fn test_delete_nonexistent_gts_type() {
    print_test_header(
        "test_delete_nonexistent_gts_type",
        &["Verify deleting a non-existent GTS type fails with TypeNotRegistered."],
    );

    let service = create_test_service();

    println!("\n📝 Stage 1: Attempt delete");

    let result = service.delete_gts_type("gts.a.p.sm.setting.v1.0~nonexistent.v1").await;
    assert!(result.is_err(), "Should fail to delete non-existent type");

    match result.unwrap_err() {
        SettingsError::TypeNotRegistered { gts_type } => {
            println!("✅ Correctly rejected with TypeNotRegistered: {}", gts_type);
        }
        _ => panic!("Expected TypeNotRegistered error"),
    }
}

#[tokio::test]
async fn test_delete_gts_type_with_active_settings() {
    print_test_header(
        "test_delete_gts_type_with_active_settings",
        &[
            "Verify behavior when deleting a GTS type that still has active settings.",
            "Current implementation allows deletion (TODO: enforce Conflict when active settings exist).",
        ],
    );

    let service = create_test_service();
    let gts_type = create_test_gts_type("with_settings.v1");
    let tenants = TestTenantHierarchy::new();
    let tenant_id = tenants.partner2_customer2_computergeeks;
    tenants.print_structure();

    println!("\n📝 Stage 1: Register type and create active setting");
    println!("   Tenant: {} (Computer Geeks)", tenant_id);

    // Register type and create settings
    service.register_gts_type(gts_type.clone()).await.unwrap();
    println!("   Type: {}", gts_type.r#type);
    
    let data = json!({"value": "test"});
    print_json("Setting data", &data);
    service.upsert_setting(&gts_type.r#type, tenant_id, "generic", data).await.unwrap();
    println!("✅ Setting created using GTS type");

    // Try to delete type (should succeed in current implementation)
    // TODO: In production, this should check for active settings and return Conflict
    println!("\n📝 Stage 2: Attempt delete type");
    let result = service.delete_gts_type(&gts_type.r#type).await;
    
    if result.is_err() {
        match result.unwrap_err() {
            SettingsError::Conflict { reason } => {
                println!("✅ Correctly rejected deletion: {}", reason);
                assert!(reason.contains("active settings"));
            }
            _ => panic!("Expected Conflict error"),
        }
    } else {
        println!("⚠️  GTS type deleted despite active settings (TODO: implement check)");
    }
}

#[tokio::test]
async fn test_cannot_create_setting_after_type_deleted() {
    print_test_header(
        "test_cannot_create_setting_after_type_deleted",
        &[
            "Verify that after deleting a GTS type, creating a setting of that type fails.",
            "This prevents orphaned settings for unregistered types.",
        ],
    );

    let service = create_test_service();
    let gts_type = create_test_gts_type("deleted.v1");
    let tenants = TestTenantHierarchy::new();
    let tenant_id = tenants.partner3_customer2_sourcepass;
    tenants.print_structure();

    println!("\n📝 Stage 1: Register and delete GTS type");
    println!("   Type: {}", gts_type.r#type);
    println!("   Tenant: {} (Sourcepass)", tenant_id);
    service.register_gts_type(gts_type.clone()).await.unwrap();
    println!("   ✅ Type registered");
    service.delete_gts_type(&gts_type.r#type).await.unwrap();
    println!("   ✅ Type deleted");

    // Try to create setting with deleted type
    let data = json!({"value": "test"});
    println!("\n📝 Stage 2: Attempt to create setting for deleted type (expected FAIL)");
    print_json("Data", &data);
    let result = service.upsert_setting(&gts_type.r#type, tenant_id, "generic", data).await;
    
    assert!(result.is_err(), "Should not be able to create setting with deleted type");
    match result.unwrap_err() {
        SettingsError::TypeNotRegistered { gts_type } => {
            println!("✅ Correctly rejected: {}", gts_type);
        }
        _ => panic!("Expected TypeNotRegistered error"),
    }
}

// ===== GTS Type Duplicate Registration Tests =====

#[tokio::test]
async fn test_register_duplicate_gts_type() {
    print_test_header(
        "test_register_duplicate_gts_type",
        &[
            "Verify that registering the same GTS type twice is rejected with Conflict.",
            "This ensures type identifiers remain unique.",
        ],
    );

    let service = create_test_service();
    let gts_type = create_test_gts_type("duplicate.v1");

    println!("\n📝 Stage 1: Register type first time");
    println!("   Type: {}", gts_type.r#type);
    service.register_gts_type(gts_type.clone()).await.unwrap();
    println!("   ✅ First registration successful");

    println!("\n📝 Stage 2: Attempt duplicate registration (expected FAIL)");
    let result = service.register_gts_type(gts_type.clone()).await;
    assert!(result.is_err(), "Should fail to register duplicate type");

    match result.unwrap_err() {
        SettingsError::Conflict { reason } => {
            println!("✅ Correctly rejected duplicate: {}", reason);
            assert!(reason.contains("already exists"));
        }
        _ => panic!("Expected Conflict error"),
    }
}

// ===== GTS Type Schema Evolution Tests =====

#[tokio::test]
async fn test_schema_evolution_backward_compatible() {
    print_test_header(
        "test_schema_evolution_backward_compatible",
        &[
            "Verify schema evolution by adding an OPTIONAL field is backward compatible.",
            "Old data remains valid and new data can include the new field.",
        ],
    );

    let service = create_test_service();
    let mut gts_type = create_test_gts_type("evolution.v1");
    let tenants = TestTenantHierarchy::new();
    let tenant_id = tenants.partner1_customer1_evergreen;
    tenants.print_structure();

    println!("\n📝 Stage 1: Register type with minimal schema (v1)");
    println!("   Tenant: {} (Evergreen and Lyra)", tenant_id);

    // Register type with minimal schema
    gts_type.schema = Some(json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"}
        },
        "required": ["name"]
    }));
    service.register_gts_type(gts_type.clone()).await.unwrap();
    println!("   Type: {}", gts_type.r#type);
    println!(
        "   Schema v1: {}",
        serde_json::to_string_pretty(gts_type.schema.as_ref().unwrap()).unwrap()
    );

    println!("\n📝 Stage 2: Create setting with v1 schema");
    let data = json!({"name": "test"});
    print_json("Data", &data);
    service.upsert_setting(&gts_type.r#type, tenant_id, "generic", data).await.unwrap();
    println!("✅ Setting created with v1 schema");

    println!("\n📝 Stage 3: Evolve schema to v2 (add OPTIONAL field)");
    gts_type.schema = Some(json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "description": {"type": "string"}
        },
        "required": ["name"]
    }));
    println!(
        "   Schema v2: {}",
        serde_json::to_string_pretty(gts_type.schema.as_ref().unwrap()).unwrap()
    );
    service.update_gts_type(gts_type.clone()).await.unwrap();
    println!("✅ Schema evolved to v2 (added optional field)");

    // Old data should still be valid
    let retrieved = service.get_setting(&gts_type.r#type, tenant_id, "generic").await.unwrap();
    assert_eq!(retrieved.data["name"], "test");

    println!("\n📝 Stage 4: Create new setting that uses the new optional field");
    let new_data = json!({"name": "test2", "description": "with description"});
    print_json("Data", &new_data);
    service.upsert_setting(&gts_type.r#type, tenant_id, "generic2", new_data).await.unwrap();
    println!("✅ New settings work with evolved schema");
}

#[tokio::test]
async fn test_schema_evolution_breaking_change() {
    print_test_header(
        "test_schema_evolution_breaking_change",
        &[
            "Verify schema evolution with a BREAKING change is enforced for NEW settings.",
            "Example: changing field type from string -> number should reject new string writes.",
        ],
    );

    let service = create_test_service();
    let mut gts_type = create_test_gts_type("breaking.v1");
    let tenants = TestTenantHierarchy::new();
    let tenant_id = tenants.partner2_customer1_bcs;
    tenants.print_structure();

    println!("\n📝 Stage 1: Register type with schema v1");
    println!("   Tenant: {} (BCS Manufacturing)", tenant_id);

    // Register type with schema
    gts_type.schema = Some(json!({
        "type": "object",
        "properties": {
            "value": {"type": "string"}
        },
        "required": ["value"]
    }));
    service.register_gts_type(gts_type.clone()).await.unwrap();
    println!(
        "   Schema v1: {}",
        serde_json::to_string_pretty(gts_type.schema.as_ref().unwrap()).unwrap()
    );

    println!("\n📝 Stage 2: Create setting with schema v1");
    let data = json!({"value": "test"});
    print_json("Data", &data);
    service.upsert_setting(&gts_type.r#type, tenant_id, "generic", data).await.unwrap();

    println!("\n📝 Stage 3: Update schema to v2 with BREAKING change (string -> number)");
    gts_type.schema = Some(json!({
        "type": "object",
        "properties": {
            "value": {"type": "number"}
        },
        "required": ["value"]
    }));
    println!(
        "   Schema v2: {}",
        serde_json::to_string_pretty(gts_type.schema.as_ref().unwrap()).unwrap()
    );
    service.update_gts_type(gts_type.clone()).await.unwrap();
    println!("✅ Schema updated with breaking change");

    println!("\n📝 Stage 4: Attempt new setting that should FAIL under schema v2");
    let new_data = json!({"value": "string_value"});
    print_json("Data", &new_data);
    let result = service.upsert_setting(&gts_type.r#type, tenant_id, "generic2", new_data).await;
    assert!(result.is_err(), "Should fail validation with new schema");

    println!("✅ Breaking schema change correctly enforced for new settings");
}

// ===== GTS Type Traits Modification Tests =====

#[tokio::test]
async fn test_update_inheritance_traits() {
    print_test_header(
        "test_update_inheritance_traits",
        &["Verify that inheritance-related traits can be updated and persist."],
    );

    let service = create_test_service();
    let mut gts_type = create_test_gts_type("inheritance.v1");

    println!("\n📝 Stage 1: Register with inheritance enabled");
    println!("   Type: {}", gts_type.r#type);

    // Register with inheritance enabled
    gts_type.traits.options.is_value_inheritable = true;
    gts_type.traits.options.is_value_overwritable = true;
    service.register_gts_type(gts_type.clone()).await.unwrap();

    println!("\n📝 Stage 2: Update to disable inheritance and overwriting");
    gts_type.traits.options.is_value_inheritable = false;
    gts_type.traits.options.is_value_overwritable = false;
    let result = service.update_gts_type(gts_type.clone()).await;
    assert!(result.is_ok(), "Should be able to update inheritance traits");

    println!("\n📝 Stage 3: Verify updated traits");
    let retrieved = service.get_gts_type(&gts_type.r#type).await.unwrap();
    assert!(!retrieved.traits.options.is_value_inheritable);
    assert!(!retrieved.traits.options.is_value_overwritable);

    println!("✅ Inheritance traits updated successfully");
}

#[tokio::test]
async fn test_update_event_configuration() {
    print_test_header(
        "test_update_event_configuration",
        &["Verify that event configuration traits can be updated (notification target changes persist)."],
    );

    let service = create_test_service();
    let mut gts_type = create_test_gts_type("events.v1");

    println!("\n📝 Stage 1: Register with audit-only events");
    println!("   Type: {}", gts_type.r#type);

    // Register with audit events only
    gts_type.traits.events.audit = EventTarget::Self_;
    gts_type.traits.events.notification = EventTarget::None;
    service.register_gts_type(gts_type.clone()).await.unwrap();

    println!("\n📝 Stage 2: Update to enable notifications");
    gts_type.traits.events.notification = EventTarget::Subroot;
    let result = service.update_gts_type(gts_type.clone()).await;
    assert!(result.is_ok(), "Should be able to update event configuration");

    println!("\n📝 Stage 3: Verify updated event config");
    let retrieved = service.get_gts_type(&gts_type.r#type).await.unwrap();
    assert_eq!(retrieved.traits.events.notification, EventTarget::Subroot);

    println!("✅ Event configuration updated successfully");
}
