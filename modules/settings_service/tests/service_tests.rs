//! Integration tests for Settings Service

use settings_service::contract::*;
use settings_service::domain::repository::{GtsTypeRepository, SettingsRepository};
use settings_service::domain::{NoOpEventPublisher, Service};
use std::sync::Arc;
use uuid::Uuid;

/// Realistic tenant hierarchy for testing
/// Root → Partners (Pax8, Datto, ConnectWise) → Customers
mod common;
use common::TestTenantHierarchy;

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


// Mock repository implementations for testing
pub mod mocks {
    use super::*;
    use async_trait::async_trait;
    use parking_lot::RwLock;
    use std::collections::HashMap;

    #[derive(Clone)]
    pub struct MockSettingsRepo {
        data: Arc<RwLock<HashMap<String, Setting>>>,
    }

    impl MockSettingsRepo {
        pub fn new() -> Self {
            Self {
                data: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        fn make_key(setting_type: &str, tenant_id: Uuid, domain_object_id: &str) -> String {
            format!("{}:{}:{}", setting_type, tenant_id, domain_object_id)
        }

        /// Print verbose information about repository state
        pub fn print_state(&self, context: &str) {
            let data = self.data.read();
            println!("\n========== SettingsRepository State: {} ==========", context);
            println!("Total settings: {}", data.len());
            
            if data.is_empty() {
                println!("  (empty)");
            } else {
                for (key, setting) in data.iter() {
                    println!("\n  Key: {}", key);
                    println!("    Type: {}", setting.r#type);
                    println!("    Tenant ID: {}", setting.tenant_id);
                    println!("    Domain Object ID: {}", setting.domain_object_id);
                    println!("    Data: {}", serde_json::to_string_pretty(&setting.data).unwrap_or_else(|_| "N/A".to_string()));
                    println!("    Created: {}", setting.created_at);
                    println!("    Updated: {}", setting.updated_at);
                    println!("    Deleted: {:?}", setting.deleted_at);
                }
            }
            println!("====================================================\n");
        }

        /// Get count of settings
        pub fn count(&self) -> usize {
            self.data.read().len()
        }

        /// Get count of non-deleted settings
        pub fn count_active(&self) -> usize {
            self.data.read().values().filter(|s| s.deleted_at.is_none()).count()
        }
    }

    #[async_trait]
    impl SettingsRepository for MockSettingsRepo {
        async fn upsert(&self, setting: &Setting) -> anyhow::Result<Setting> {
            let key = Self::make_key(&setting.r#type, setting.tenant_id, &setting.domain_object_id);
            self.data.write().insert(key, setting.clone());
            Ok(setting.clone())
        }

        async fn find_by_key(
            &self,
            setting_type: &str,
            tenant_id: Uuid,
            domain_object_id: &str,
        ) -> anyhow::Result<Option<Setting>> {
            let key = Self::make_key(setting_type, tenant_id, domain_object_id);
            Ok(self.data.read().get(&key).and_then(|s| {
                // Filter out soft-deleted items
                if s.deleted_at.is_none() {
                    Some(s.clone())
                } else {
                    None
                }
            }))
        }

        async fn find_by_type(
            &self,
            setting_type: &str,
            tenant_id: Option<Uuid>,
        ) -> anyhow::Result<Vec<Setting>> {
            let data = self.data.read();
            let results: Vec<Setting> = data
                .values()
                .filter(|s| {
                    s.r#type == setting_type
                        && (tenant_id.is_none() || tenant_id == Some(s.tenant_id))
                })
                .cloned()
                .collect();
            Ok(results)
        }

        async fn soft_delete(
            &self,
            setting_type: &str,
            tenant_id: Uuid,
            domain_object_id: &str,
        ) -> anyhow::Result<()> {
            let key = Self::make_key(setting_type, tenant_id, domain_object_id);
            if let Some(setting) = self.data.write().get_mut(&key) {
                setting.deleted_at = Some(chrono::Utc::now());
            }
            Ok(())
        }

        async fn hard_delete(
            &self,
            setting_type: &str,
            tenant_id: Uuid,
            domain_object_id: &str,
        ) -> anyhow::Result<()> {
            let key = Self::make_key(setting_type, tenant_id, domain_object_id);
            self.data.write().remove(&key);
            Ok(())
        }

        async fn find_by_tenant(&self, tenant_id: Uuid) -> anyhow::Result<Vec<Setting>> {
            let data = self.data.read();
            let results: Vec<Setting> = data
                .values()
                .filter(|s| s.tenant_id == tenant_id)
                .cloned()
                .collect();
            Ok(results)
        }

        async fn find_by_domain_object(&self, domain_object_id: &str) -> anyhow::Result<Vec<Setting>> {
            let data = self.data.read();
            let results: Vec<Setting> = data
                .values()
                .filter(|s| s.domain_object_id == domain_object_id)
                .cloned()
                .collect();
            Ok(results)
        }

        async fn list_all(&self, _limit: u64, _offset: u64) -> anyhow::Result<Vec<Setting>> {
            Ok(self.data.read().values().cloned().collect())
        }
    }

    #[derive(Clone)]
    pub struct MockGtsTypeRepo {
        data: Arc<RwLock<HashMap<String, GtsType>>>,
    }

    impl MockGtsTypeRepo {
        pub fn new() -> Self {
            Self {
                data: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl GtsTypeRepository for MockGtsTypeRepo {
        async fn create(&self, gts_type: &GtsType) -> anyhow::Result<GtsType> {
            self.data
                .write()
                .insert(gts_type.r#type.clone(), gts_type.clone());
            Ok(gts_type.clone())
        }

        async fn find_by_type(&self, type_id: &str) -> anyhow::Result<Option<GtsType>> {
            Ok(self.data.read().get(type_id).cloned())
        }

        async fn list_all(&self) -> anyhow::Result<Vec<GtsType>> {
            Ok(self.data.read().values().cloned().collect())
        }

        async fn update(&self, gts_type: &GtsType) -> anyhow::Result<GtsType> {
            self.data
                .write()
                .insert(gts_type.r#type.clone(), gts_type.clone());
            Ok(gts_type.clone())
        }

        async fn delete(&self, type_id: &str) -> anyhow::Result<()> {
            self.data.write().remove(type_id);
            Ok(())
        }

        async fn exists(&self, type_id: &str) -> anyhow::Result<bool> {
            Ok(self.data.read().contains_key(type_id))
        }
    }
}

fn create_test_service() -> Service {
    let settings_repo = Arc::new(mocks::MockSettingsRepo::new());
    let gts_type_repo = Arc::new(mocks::MockGtsTypeRepo::new());
    let event_publisher = Arc::new(NoOpEventPublisher);
    Service::new(settings_repo, gts_type_repo, event_publisher)
}

fn create_test_service_with_repos() -> (Service, Arc<mocks::MockSettingsRepo>, Arc<mocks::MockGtsTypeRepo>) {
    let settings_repo = Arc::new(mocks::MockSettingsRepo::new());
    let gts_type_repo = Arc::new(mocks::MockGtsTypeRepo::new());
    let event_publisher = Arc::new(NoOpEventPublisher);
    let service = Service::new(settings_repo.clone(), gts_type_repo.clone(), event_publisher);
    (service, settings_repo, gts_type_repo)
}

fn create_test_gts_type() -> GtsType {
    GtsType {
        r#type: "gts.a.p.sm.setting.v1.0~test.setting.v1".to_string(),
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

#[tokio::test]
async fn test_register_and_get_gts_type() {
    let service = create_test_service();
    let gts_type = create_test_gts_type();

    print_test_header(
        "test_register_and_get_gts_type",
        &["Verify that registering a GTS type persists it and get returns the same type."],
    );

    println!("\n📝 Stage 1: Register GTS type");
    println!("   Type: {}", gts_type.r#type);

    // Register GTS type
    let registered = service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    assert_eq!(registered.r#type, gts_type.r#type);

    println!("\n📝 Stage 2: Get GTS type");
    // Get GTS type
    let retrieved = service
        .get_gts_type(&gts_type.r#type)
        .await
        .expect("Failed to get GTS type");

    assert_eq!(retrieved.r#type, gts_type.r#type);
}

#[tokio::test]
async fn test_upsert_setting() {
    let (service, settings_repo, _gts_repo) = create_test_service_with_repos();
    let gts_type = create_test_gts_type();
    let tenants = TestTenantHierarchy::new();
    let tenant_id = tenants.partner1_pax8;

    print_test_header(
        "test_upsert_setting",
        &[
            "Verify that upsert_setting persists the setting and the repository reflects it.",
            "This also prints repository state before and after upsert for debugging.",
        ],
    );
    tenants.print_structure();
    println!("\n📝 Stage 1: Setup");
    println!("   Tenant: {} (Pax8)", tenant_id);
    println!("   Domain Object: generic");
    
    settings_repo.print_state("Initial state");

    // Register GTS type first
    println!("\n📝 Stage 2: Register GTS type");
    println!("   Type: {}", gts_type.r#type);
    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    // Upsert setting
    println!("\n📝 Stage 3: Upsert setting");
    let data = serde_json::json!({"key": "value"});
    print_json("Data", &data);
    
    let setting = service
        .upsert_setting(&gts_type.r#type, tenant_id, "generic", data.clone())
        .await
        .expect("Failed to upsert setting");

    println!("\n📝 Stage 4: Inspect repo state");
    settings_repo.print_state("After upsert");
    
    println!("✅ Active settings count: {}", settings_repo.count_active());
    println!("📊 Total settings count: {}", settings_repo.count());

    assert_eq!(setting.r#type, gts_type.r#type);
    assert_eq!(setting.tenant_id, tenant_id);
    assert_eq!(setting.data, data);
}

#[tokio::test]
async fn test_upsert_setting_without_gts_type() {
    let service = create_test_service();
    let tenants = TestTenantHierarchy::new();
    let tenant_id = tenants.partner3_customer1_sanit;

    print_test_header(
        "test_upsert_setting_without_gts_type",
        &["Verify that upserting a setting without registering its type returns an error."],
    );
    tenants.print_structure();
    println!("\n📝 Stage 1: Attempt upsert without registering type");
    println!("   Tenant: {} (San-iT)", tenant_id);

    // Try to upsert setting without registering GTS type
    let data = serde_json::json!({"key": "value"});
    print_json("Data", &data);
    let result = service
        .upsert_setting("nonexistent.type", tenant_id, "generic", data)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        SettingsError::NotFound { .. } 
        | SettingsError::InvalidGtsFormat { .. } 
        | SettingsError::TypeNotRegistered { .. } => {}
        e => panic!("Expected NotFound, InvalidGtsFormat, or TypeNotRegistered error, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_get_setting() {
    let service = create_test_service();
    let gts_type = create_test_gts_type();
    let tenants = TestTenantHierarchy::new();
    let tenant_id = tenants.partner2_customer2_computergeeks;

    print_test_header(
        "test_get_setting",
        &["Verify that a setting can be retrieved after being upserted."],
    );
    tenants.print_structure();
    println!("\n📝 Stage 1: Setup");
    println!("   Tenant: {} (Computer Geeks)", tenant_id);

    // Register GTS type and create setting
    println!("\n📝 Stage 2: Register GTS type");
    println!("   Type: {}", gts_type.r#type);
    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    let data = serde_json::json!({"key": "value"});
    println!("\n📝 Stage 3: Upsert setting");
    print_json("Data", &data);
    service
        .upsert_setting(&gts_type.r#type, tenant_id, "generic", data.clone())
        .await
        .expect("Failed to upsert setting");

    // Get setting
    println!("\n📝 Stage 4: Get setting");
    let retrieved = service
        .get_setting(&gts_type.r#type, tenant_id, "generic")
        .await
        .expect("Failed to get setting");

    assert_eq!(retrieved.data, data);
}

#[tokio::test]
async fn test_delete_setting() {
    let (service, settings_repo, _gts_repo) = create_test_service_with_repos();
    let gts_type = create_test_gts_type();
    let tenants = TestTenantHierarchy::new();
    let tenant_id = tenants.partner2_datto;

    print_test_header(
        "test_delete_setting",
        &[
            "Verify that delete_setting soft-deletes a setting and it becomes non-retrievable.",
            "This prints repository state before and after deletion for debugging.",
        ],
    );
    tenants.print_structure();
    println!("\n📝 Stage 1: Setup");
    println!("   Tenant: {} (Datto)", tenant_id);

    // Register GTS type and create setting
    println!("\n📝 Stage 2: Register GTS type");
    println!("   Type: {}", gts_type.r#type);
    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    let data = serde_json::json!({"key": "value"});
    println!("\n📝 Stage 3: Upsert setting");
    print_json("Data", &data);
    service
        .upsert_setting(&gts_type.r#type, tenant_id, "generic", data)
        .await
        .expect("Failed to upsert setting");

    println!("\n📝 Stage 4: Repo state after create");
    settings_repo.print_state("After creating setting");
    println!("✅ Active settings: {}", settings_repo.count_active());

    // Delete setting
    println!("\n📝 Stage 5: Delete setting (soft delete)");
    service
        .delete_setting(&gts_type.r#type, tenant_id, "generic")
        .await
        .expect("Failed to delete setting");

    println!("\n📝 Stage 6: Repo state after delete");
    settings_repo.print_state("After soft delete");
    println!("✅ Active settings: {}", settings_repo.count_active());
    println!("📊 Total settings (including deleted): {}", settings_repo.count());

    // Verify setting is deleted (soft delete means it still exists but with deleted_at set)
    // The service's delete_setting uses soft delete, so we can't retrieve it anymore
    let result = service.get_setting(&gts_type.r#type, tenant_id, "generic").await;
    assert!(result.is_err(), "Setting should not be retrievable after deletion");
}

#[tokio::test]
async fn test_json_schema_validation() {
    let service = create_test_service();
    let tenants = TestTenantHierarchy::new();
    let tenant_id = tenants.partner1_customer2_braden;

    print_test_header(
        "test_json_schema_validation",
        &["Verify that schema validation accepts valid data and rejects invalid data."],
    );
    tenants.print_structure();
    println!("\n📝 Stage 1: Setup");
    println!("   Tenant: {} (Braden Business Systems)", tenant_id);

    // Create GTS type with JSON schema
    println!("\n📝 Stage 2: Register GTS type with schema");
    let mut gts_type = create_test_gts_type();
    gts_type.schema = Some(serde_json::json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "number", "minimum": 0 }
        },
        "required": ["name"]
    }));
    print_json("Schema", gts_type.schema.as_ref().unwrap());

    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    // Valid data
    println!("\n📝 Stage 3: Upsert valid data");
    let valid_data = serde_json::json!({"name": "John", "age": 30});
    print_json("Valid data", &valid_data);
    let result = service
        .upsert_setting(&gts_type.r#type, tenant_id, "generic", valid_data)
        .await;
    assert!(result.is_ok());

    // Invalid data (missing required field)
    println!("\n📝 Stage 4: Upsert invalid data (missing required field)");
    let invalid_data = serde_json::json!({"age": 30});
    print_json("Invalid data", &invalid_data);
    let result = service
        .upsert_setting(&gts_type.r#type, tenant_id, "generic2", invalid_data)
        .await;
    assert!(result.is_err());
    match result.unwrap_err() {
        SettingsError::SchemaValidation { .. } => {}
        _ => panic!("Expected SchemaValidation error"),
    }
}

#[tokio::test]
async fn test_list_gts_types() {
    let service = create_test_service();

    // Register multiple GTS types
    for i in 1..=3 {
        let mut gts_type = create_test_gts_type();
        gts_type.r#type = format!("gts.a.p.sm.setting.v1.0~test.setting.v{}", i);
        service
            .register_gts_type(gts_type)
            .await
            .expect("Failed to register GTS type");
    }

    // List all GTS types
    let types = service.list_gts_types().await.expect("Failed to list GTS types");
    assert_eq!(types.len(), 3);
}

#[tokio::test]
async fn test_get_settings_by_type() {
    let (service, settings_repo, _gts_repo) = create_test_service_with_repos();
    let gts_type = create_test_gts_type();
    let tenants = TestTenantHierarchy::new();

    print_test_header(
        "test_get_settings_by_type",
        &["Verify that get_settings_by_type returns all settings of a type across tenants."],
    );
    tenants.print_structure();

    // Register GTS type
    println!("\n📝 Stage 1: Register GTS type");
    println!("   Type: {}", gts_type.r#type);
    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    settings_repo.print_state("Initial state");

    // Create settings for multiple tenants
    println!("\n📝 Stage 2: Create settings across tenants");
    let tenant_ids = [
        tenants.partner1_pax8,
        tenants.partner2_datto,
        tenants.partner3_connectwise,
    ];
    for (i, tenant_id) in tenant_ids.into_iter().enumerate() {
        let data = serde_json::json!({"index": i + 1});
        println!("   Tenant: {}", tenant_id);
        print_json("Data", &data);
        service
            .upsert_setting(&gts_type.r#type, tenant_id, "generic", data)
            .await
            .expect("Failed to upsert setting");
    }

    settings_repo.print_state("After creating 3 settings");

    // Get all settings of this type
    println!("\n📝 Stage 3: Get settings by type");
    let settings = service
        .get_settings_by_type(&gts_type.r#type, None)
        .await
        .expect("Failed to get settings by type");

    println!("✅ Retrieved {} settings by type", settings.len());
    assert_eq!(settings.len(), 3);
}

// ===== Tenancy Hierarchy and Inheritance Tests =====

/// Mock tenancy hierarchy for testing inheritance
mod tenancy_hierarchy {
    use super::*;
    use parking_lot::RwLock;
    use std::collections::HashMap;

    /// Represents a tenant node in the hierarchy
    #[derive(Debug, Clone)]
    pub struct TenantNode {
        #[allow(dead_code)]
        pub id: Uuid,
        #[allow(dead_code)]
        pub name: String,
        pub parent_id: Option<Uuid>,
        pub children: Vec<Uuid>,
    }

    /// Mock tenancy hierarchy manager
    #[derive(Clone)]
    pub struct MockTenancyHierarchy {
        tenants: Arc<RwLock<HashMap<Uuid, TenantNode>>>,
    }

    impl MockTenancyHierarchy {
        pub fn new() -> Self {
            Self {
                tenants: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        /// Add a tenant to the hierarchy
        pub fn add_tenant(&self, id: Uuid, name: String, parent_id: Option<Uuid>) {
            let node = TenantNode {
                id,
                name,
                parent_id,
                children: Vec::new(),
            };

            self.tenants.write().insert(id, node);

            // Update parent's children list
            if let Some(parent_id) = parent_id {
                if let Some(parent) = self.tenants.write().get_mut(&parent_id) {
                    parent.children.push(id);
                }
            }
        }

        /// Get parent tenant ID
        pub fn get_parent(&self, tenant_id: Uuid) -> Option<Uuid> {
            self.tenants.read().get(&tenant_id)?.parent_id
        }

        /// Get all ancestor tenant IDs (from immediate parent to root)
        pub fn get_ancestors(&self, tenant_id: Uuid) -> Vec<Uuid> {
            let mut ancestors = Vec::new();
            let mut current = tenant_id;

            while let Some(parent_id) = self.get_parent(current) {
                ancestors.push(parent_id);
                current = parent_id;
            }

            ancestors
        }

        /// Get all child tenant IDs
        pub fn get_children(&self, tenant_id: Uuid) -> Vec<Uuid> {
            self.tenants
                .read()
                .get(&tenant_id)
                .map(|node| node.children.clone())
                .unwrap_or_default()
        }

        /// Get all descendant tenant IDs (recursive)
        pub fn get_descendants(&self, tenant_id: Uuid) -> Vec<Uuid> {
            let mut descendants = Vec::new();
            let children = self.get_children(tenant_id);

            for child_id in children {
                descendants.push(child_id);
                descendants.extend(self.get_descendants(child_id));
            }

            descendants
        }

        /// Check if tenant is ancestor of another tenant
        pub fn is_ancestor(&self, potential_ancestor: Uuid, tenant_id: Uuid) -> bool {
            self.get_ancestors(tenant_id).contains(&potential_ancestor)
        }
    }
}

use tenancy_hierarchy::MockTenancyHierarchy;

/// Helper to resolve inherited setting value
async fn resolve_inherited_setting(
    service: &Service,
    hierarchy: &MockTenancyHierarchy,
    setting_type: &str,
    tenant_id: Uuid,
    domain_object_id: &str,
) -> Option<Setting> {
    // Try to get setting for current tenant
    if let Ok(setting) = service.get_setting(setting_type, tenant_id, domain_object_id).await {
        return Some(setting);
    }

    // Walk up the hierarchy to find inherited value
    for ancestor_id in hierarchy.get_ancestors(tenant_id) {
        if let Ok(setting) = service.get_setting(setting_type, ancestor_id, domain_object_id).await {
            // Check if this setting is inheritable
            if let Ok(gts_type) = service.get_gts_type(setting_type).await {
                if gts_type.traits.options.is_value_inheritable 
                    && !gts_type.traits.options.is_barrier_inheritance {
                    return Some(setting);
                }
            }
        }
    }

    None
}

#[tokio::test]
async fn test_inheritance_basic() {
    let service = create_test_service();
    let hierarchy = MockTenancyHierarchy::new();

    print_test_header(
        "test_inheritance_basic",
        &[
            "Verify that a child/grandchild tenant inherits a setting from the nearest ancestor that defines it.",
            "This uses a simplified Root -> Child -> Grandchild chain created from the shared test hierarchy.",
        ],
    );

    let tenants = TestTenantHierarchy::new();
    tenants.print_structure();

    // Create hierarchy: Root -> Child -> Grandchild
    let root_id = tenants.root;
    let child_id = tenants.partner1_pax8;
    let grandchild_id = tenants.partner1_customer1_evergreen;

    println!("\n📝 Stage 1: Register tenant hierarchy in mock");
    println!("   Root: {}", root_id);
    println!("   Child: {} (Pax8)", child_id);
    println!("   Grandchild: {} (Evergreen and Lyra)", grandchild_id);

    hierarchy.add_tenant(root_id, "Root Tenant".to_string(), None);
    hierarchy.add_tenant(child_id, "Child Tenant".to_string(), Some(root_id));
    hierarchy.add_tenant(grandchild_id, "Grandchild Tenant".to_string(), Some(child_id));

    // Register GTS type with inheritance enabled
    println!("\n📝 Stage 2: Register inheritable/overwritable GTS type");
    let mut gts_type = create_test_gts_type();
    gts_type.r#type = "gts.a.p.sm.setting.v1.0~test.inheritance.v1".to_string();
    gts_type.traits.options.is_value_inheritable = true;
    gts_type.traits.options.is_value_overwritable = true;
    gts_type.traits.options.is_barrier_inheritance = false;
    println!("   Type: {}", gts_type.r#type);
    println!("   is_value_inheritable: {}", gts_type.traits.options.is_value_inheritable);
    println!("   is_value_overwritable: {}", gts_type.traits.options.is_value_overwritable);
    println!("   is_barrier_inheritance: {}", gts_type.traits.options.is_barrier_inheritance);

    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    // Set value at root level
    let root_data = serde_json::json!({"level": "root", "value": 100});

    println!("\n📝 Stage 3: Upsert root value");
    println!("   Tenant: {}", root_id);
    print_json("Root data", &root_data);

    service
        .upsert_setting(&gts_type.r#type, root_id, "generic", root_data.clone())
        .await
        .expect("Failed to set root setting");

    // Child should inherit from root
    println!("\n📝 Stage 4: Resolve inherited value at child");
    let child_setting = resolve_inherited_setting(
        &service,
        &hierarchy,
        &gts_type.r#type,
        child_id,
        "generic",
    )
    .await;

    assert!(child_setting.is_some());
    let child_setting = child_setting.unwrap();
    println!("   ✅ Child resolved from tenant: {}", child_setting.tenant_id);
    print_json("Child resolved data", &child_setting.data);
    assert_eq!(child_setting.tenant_id, root_id); // Inherited from root

    // Grandchild should also inherit from root
    println!("\n📝 Stage 5: Resolve inherited value at grandchild");
    let grandchild_setting = resolve_inherited_setting(
        &service,
        &hierarchy,
        &gts_type.r#type,
        grandchild_id,
        "generic",
    )
    .await;

    assert!(grandchild_setting.is_some());
    let grandchild_setting = grandchild_setting.unwrap();
    println!("   ✅ Grandchild resolved from tenant: {}", grandchild_setting.tenant_id);
    print_json("Grandchild resolved data", &grandchild_setting.data);
    assert_eq!(grandchild_setting.tenant_id, root_id);
}

#[tokio::test]
async fn test_inheritance_override() {
    let (service, settings_repo, _gts_repo) = create_test_service_with_repos();
    let hierarchy = MockTenancyHierarchy::new();

    print_test_header(
        "test_inheritance_override",
        &[
            "Verify that a child can override an inherited value when overwritable is enabled.",
            "Grandchild should then inherit from child (nearest ancestor), not root.",
        ],
    );

    let tenants = TestTenantHierarchy::new();
    tenants.print_structure();

    // Create hierarchy: Root -> Child -> Grandchild
    let root_id = tenants.root;
    let child_id = tenants.partner2_datto;
    let grandchild_id = tenants.partner2_customer1_bcs;

    println!("\n📝 Stage 1: Register tenant hierarchy in mock");
    println!("   Root: {}", root_id);
    println!("   Child: {} (Datto)", child_id);
    println!("   Grandchild: {} (BCS Manufacturing)", grandchild_id);

    hierarchy.add_tenant(root_id, "Root Tenant".to_string(), None);
    hierarchy.add_tenant(child_id, "Child Tenant".to_string(), Some(root_id));
    hierarchy.add_tenant(grandchild_id, "Grandchild Tenant".to_string(), Some(child_id));

    // Register GTS type with overwritable inheritance
    println!("\n📝 Stage 2: Register inheritable/overwritable GTS type");
    let mut gts_type = create_test_gts_type();
    gts_type.r#type = "gts.a.p.sm.setting.v1.0~test.override.v1".to_string();
    gts_type.traits.options.is_value_inheritable = true;
    gts_type.traits.options.is_value_overwritable = true;
    println!("   Type: {}", gts_type.r#type);
    println!("   is_value_inheritable: {}", gts_type.traits.options.is_value_inheritable);
    println!("   is_value_overwritable: {}", gts_type.traits.options.is_value_overwritable);

    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    settings_repo.print_state("Initial state");

    // Set value at root level
    let root_data = serde_json::json!({"level": "root", "value": 100});
    println!("\n📝 Stage 3: Upsert root value");
    println!("   Tenant: {}", root_id);
    print_json("Root data", &root_data);
    service
        .upsert_setting(&gts_type.r#type, root_id, "generic", root_data)
        .await
        .expect("Failed to set root setting");

    settings_repo.print_state("After root setting");

    // Override at child level
    let child_data = serde_json::json!({"level": "child", "value": 200});
    println!("\n📝 Stage 4: Override at child");
    println!("   Tenant: {}", child_id);
    print_json("Child override data", &child_data);
    service
        .upsert_setting(&gts_type.r#type, child_id, "generic", child_data.clone())
        .await
        .expect("Failed to set child setting");

    settings_repo.print_state("After child override");

    // Child should have its own value
    let child_setting = service
        .get_setting(&gts_type.r#type, child_id, "generic")
        .await
        .expect("Failed to get child setting");

    println!("\n📝 Stage 5: Validate child stored value");
    println!("   Stored tenant: {}", child_setting.tenant_id);
    print_json("Child stored data", &child_setting.data);

    assert_eq!(child_setting.tenant_id, child_id);
    assert_eq!(child_setting.data["level"], "child");
    assert_eq!(child_setting.data["value"], 200);

    // Grandchild should inherit from child (not root)
    println!("\n📝 Stage 6: Resolve inherited value at grandchild");
    let grandchild_setting = resolve_inherited_setting(
        &service,
        &hierarchy,
        &gts_type.r#type,
        grandchild_id,
        "generic",
    )
    .await;

    assert!(grandchild_setting.is_some());
    let grandchild_setting = grandchild_setting.unwrap();

    println!("   ✅ Grandchild resolved from tenant: {} (should be child)", grandchild_setting.tenant_id);
    print_json("Grandchild resolved data", &grandchild_setting.data);
    
    assert_eq!(grandchild_setting.tenant_id, child_id); // Inherited from child
    assert_eq!(grandchild_setting.data["level"], "child");
}

#[tokio::test]
async fn test_inheritance_barrier() {
    let service = create_test_service();
    let hierarchy = MockTenancyHierarchy::new();

    print_test_header(
        "test_inheritance_barrier",
        &[
            "Verify that a setting with is_barrier_inheritance=true is not inherited by children.",
            "This test expects resolve_inherited_setting to return None for the child.",
        ],
    );

    let tenants = TestTenantHierarchy::new();
    tenants.print_structure();

    // Create hierarchy: Root -> Child -> Grandchild
    let root_id = tenants.root;
    let child_id = tenants.partner3_connectwise;
    let grandchild_id = tenants.partner3_customer2_sourcepass;

    println!("\n📝 Stage 1: Register tenant hierarchy in mock");
    println!("   Root: {}", root_id);
    println!("   Child: {} (ConnectWise)", child_id);
    println!("   Grandchild: {} (Sourcepass)", grandchild_id);

    hierarchy.add_tenant(root_id, "Root Tenant".to_string(), None);
    hierarchy.add_tenant(child_id, "Child Tenant".to_string(), Some(root_id));
    hierarchy.add_tenant(grandchild_id, "Grandchild Tenant".to_string(), Some(child_id));

    // Register GTS type with inheritance barrier
    println!("\n📝 Stage 2: Register inheritable type with inheritance barrier enabled");
    let mut gts_type = create_test_gts_type();
    gts_type.r#type = "gts.a.p.sm.setting.v1.0~test.barrier.v1".to_string();
    gts_type.traits.options.is_value_inheritable = true;
    gts_type.traits.options.is_barrier_inheritance = true; // Barrier enabled
    println!("   Type: {}", gts_type.r#type);
    println!("   is_value_inheritable: {}", gts_type.traits.options.is_value_inheritable);
    println!("   is_barrier_inheritance: {}", gts_type.traits.options.is_barrier_inheritance);

    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    // Set value at root level
    let root_data = serde_json::json!({"level": "root", "value": 100});
    println!("\n📝 Stage 3: Upsert root value");
    print_json("Root data", &root_data);
    service
        .upsert_setting(&gts_type.r#type, root_id, "generic", root_data)
        .await
        .expect("Failed to set root setting");

    // Child should NOT inherit due to barrier
    println!("\n📝 Stage 4: Attempt resolve at child (should be blocked)");
    let child_setting = resolve_inherited_setting(
        &service,
        &hierarchy,
        &gts_type.r#type,
        child_id,
        "generic",
    )
    .await;

    if child_setting.is_none() {
        println!("   ✅ No inherited value resolved due to barrier");
    }
    assert!(child_setting.is_none()); // Barrier blocks inheritance
}

#[tokio::test]
async fn test_inheritance_not_overwritable() {
    let service = create_test_service();
    let hierarchy = MockTenancyHierarchy::new();

    print_test_header(
        "test_inheritance_not_overwritable",
        &[
            "Document current behavior when is_value_overwritable=false and a parent value exists.",
            "Depending on implementation/hierarchy integration, child upsert may be blocked or allowed.",
        ],
    );

    let tenants = TestTenantHierarchy::new();
    tenants.print_structure();

    // Create hierarchy: Root -> Child
    let root_id = tenants.root;
    let child_id = tenants.partner1_customer2_braden;

    println!("\n📝 Stage 1: Register tenant hierarchy in mock");
    println!("   Root: {}", root_id);
    println!("   Child: {} (Braden Business Systems)", child_id);

    hierarchy.add_tenant(root_id, "Root Tenant".to_string(), None);
    hierarchy.add_tenant(child_id, "Child Tenant".to_string(), Some(root_id));

    // Register GTS type with non-overwritable inheritance
    println!("\n📝 Stage 2: Register inheritable but NOT overwritable GTS type");
    let mut gts_type = create_test_gts_type();
    gts_type.r#type = "gts.a.p.sm.setting.v1.0~test.locked.v1".to_string();
    gts_type.traits.options.is_value_inheritable = true;
    gts_type.traits.options.is_value_overwritable = false; // Cannot override
    println!("   Type: {}", gts_type.r#type);
    println!("   is_value_inheritable: {}", gts_type.traits.options.is_value_inheritable);
    println!("   is_value_overwritable: {}", gts_type.traits.options.is_value_overwritable);

    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    // Set value at root level
    let root_data = serde_json::json!({"level": "root", "value": 100});
    println!("\n📝 Stage 3: Upsert root value");
    print_json("Root data", &root_data);
    service
        .upsert_setting(&gts_type.r#type, root_id, "generic", root_data)
        .await
        .expect("Failed to set root setting");

    // Child can still create its own setting (implementation dependent)
    // In a full implementation, this might be blocked by business logic
    let child_data = serde_json::json!({"level": "child", "value": 200});
    println!("\n📝 Stage 4: Attempt child upsert (observational)");
    print_json("Child data", &child_data);
    let result = service
        .upsert_setting(&gts_type.r#type, child_id, "generic", child_data)
        .await;

    // This test documents current behavior - in production, you might want to
    // add validation to prevent overriding non-overwritable settings
    // Note: With root/admin override implementation, the check is now in place
    // but requires full hierarchy traversal integration (Phase 3)
    if result.is_err() {
        println!("   ✅ Non-overwritable constraint enforced (with hierarchy check)");
    } else {
        println!("   ⚠️  Setting created (hierarchy traversal not fully integrated)");
    }
}

#[tokio::test]
async fn test_multi_level_inheritance() {
    let _service: Service = create_test_service();
    let hierarchy = MockTenancyHierarchy::new();

    print_test_header(
        "test_multi_level_inheritance",
        &[
            "Verify that MockTenancyHierarchy returns correct ancestors/descendants across multiple levels.",
            "This uses 5 tenant IDs from the shared hierarchy wired into a single 5-level chain.",
        ],
    );

    let tenants = TestTenantHierarchy::new();
    tenants.print_structure();

    // Create 5-level hierarchy
    let level1 = tenants.root;
    let level2 = tenants.partner1_pax8;
    let level3 = tenants.partner1_customer1_evergreen;
    let level4 = tenants.partner1_customer2_braden;
    let level5 = tenants.partner2_datto;

    println!("\n📝 Stage 1: Register 5-level tenant chain in mock");
    println!("   Level 1: {}", level1);
    println!("   Level 2: {}", level2);
    println!("   Level 3: {}", level3);
    println!("   Level 4: {}", level4);
    println!("   Level 5: {}", level5);

    hierarchy.add_tenant(level1, "Level 1".to_string(), None);
    hierarchy.add_tenant(level2, "Level 2".to_string(), Some(level1));
    hierarchy.add_tenant(level3, "Level 3".to_string(), Some(level2));
    hierarchy.add_tenant(level4, "Level 4".to_string(), Some(level3));
    hierarchy.add_tenant(level5, "Level 5".to_string(), Some(level4));

    // Verify hierarchy
    println!("\n📝 Stage 2: Verify ancestors(level5)");
    let ancestors = hierarchy.get_ancestors(level5);
    println!("   Ancestors: {:?}", ancestors);
    assert_eq!(ancestors.len(), 4);
    assert_eq!(ancestors[0], level4);
    assert_eq!(ancestors[1], level3);
    assert_eq!(ancestors[2], level2);
    assert_eq!(ancestors[3], level1);

    // Verify descendants
    println!("\n📝 Stage 3: Verify descendants(level1)");
    let descendants = hierarchy.get_descendants(level1);
    println!("   Descendants: {:?}", descendants);
    assert_eq!(descendants.len(), 4);
    assert!(descendants.contains(&level2));
    assert!(descendants.contains(&level5));

    // Verify ancestry check
    println!("\n📝 Stage 4: Verify is_ancestor checks");
    assert!(hierarchy.is_ancestor(level1, level5));
    assert!(hierarchy.is_ancestor(level3, level5));
    assert!(!hierarchy.is_ancestor(level5, level1));
}

#[tokio::test]
async fn test_sibling_isolation() {
    let service = create_test_service();
    let hierarchy = MockTenancyHierarchy::new();

    print_test_header(
        "test_sibling_isolation",
        &[
            "Verify that sibling tenants do not inherit from each other and cannot read each other's tenant-scoped settings.",
            "This test writes a setting for one child and confirms get on the sibling errors.",
        ],
    );

    let tenants = TestTenantHierarchy::new();
    tenants.print_structure();

    // Create hierarchy with siblings: Root -> Child1, Child2
    let root_id = tenants.root;
    let child1_id = tenants.partner1_pax8;
    let child2_id = tenants.partner2_datto;

    println!("\n📝 Stage 1: Register sibling hierarchy in mock");
    println!("   Root: {}", root_id);
    println!("   Child 1 (Pax8): {}", child1_id);
    println!("   Child 2 (Datto): {}", child2_id);

    hierarchy.add_tenant(root_id, "Root".to_string(), None);
    hierarchy.add_tenant(child1_id, "Child 1".to_string(), Some(root_id));
    hierarchy.add_tenant(child2_id, "Child 2".to_string(), Some(root_id));

    // Register GTS type
    println!("\n📝 Stage 2: Register GTS type");
    let mut gts_type = create_test_gts_type();
    gts_type.r#type = "gts.a.p.sm.setting.v1.0~test.sibling.v1".to_string();
    println!("   Type: {}", gts_type.r#type);

    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    // Set value for child1
    let child1_data = serde_json::json!({"owner": "child1"});
    println!("\n📝 Stage 3: Upsert setting for Child 1 (Pax8)");
    print_json("Child 1 data", &child1_data);
    service
        .upsert_setting(&gts_type.r#type, child1_id, "generic", child1_data)
        .await
        .expect("Failed to set child1 setting");

    // Child2 should NOT see child1's setting
    println!("\n📝 Stage 4: Attempt get for sibling Child 2 (Datto) - should fail");
    let child2_result = service
        .get_setting(&gts_type.r#type, child2_id, "generic")
        .await;

    if let Err(e) = &child2_result {
        println!("   ✅ Expected error: {:?}", e);
    }

    assert!(child2_result.is_err()); // Siblings don't inherit from each other

    // Verify child1 and child2 are not ancestors of each other
    assert!(!hierarchy.is_ancestor(child1_id, child2_id));
    assert!(!hierarchy.is_ancestor(child2_id, child1_id));
}

// ===== Domain Object ID Format Tests =====
// OpenSpec specifies: domain_object_id: String (UUID | GTS | AppCode | "generic")

#[tokio::test]
async fn test_domain_object_id_uuid_format() {
    let (service, settings_repo, _gts_repo) = create_test_service_with_repos();
    let gts_type = create_test_gts_type();
    let tenants = TestTenantHierarchy::new();
    let tenant_id = tenants.partner2_customer1_bcs;
    let storage_uuid = Uuid::new_v4();

    print_test_header(
        "test_domain_object_id_uuid_format",
        &[
            "Verify that a UUID domain_object_id can be used to upsert and retrieve a setting.",
            "This models a resource-scoped object (e.g., storage system UUID).",
        ],
    );
    tenants.print_structure();
    println!("\n📝 Stage 1: Setup");
    println!("   Tenant: {} (BCS Manufacturing)", tenant_id);
    println!("   Domain Object (UUID): {}", storage_uuid);

    println!("\n📝 Stage 2: Register GTS type");
    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    settings_repo.print_state("Initial state");

    // Create setting with UUID domain_object_id (e.g., storage system)
    let storage_data = serde_json::json!({
        "encryption_enabled": true,
        "backup_retention_days": 90
    });

    println!("\n📝 Stage 3: Upsert setting for UUID domain object");
    print_json("Data", &storage_data);
    let setting = service
        .upsert_setting(
            &gts_type.r#type,
            tenant_id,
            &storage_uuid.to_string(),
            storage_data.clone(),
        )
        .await
        .expect("Failed to create setting with UUID domain_object_id");

    settings_repo.print_state("After creating UUID-based setting");

    assert_eq!(setting.domain_object_id, storage_uuid.to_string());
    assert_eq!(setting.data, storage_data);

    // Retrieve the setting
    println!("\n📝 Stage 4: Retrieve setting");
    let retrieved = service
        .get_setting(&gts_type.r#type, tenant_id, &storage_uuid.to_string())
        .await
        .expect("Failed to retrieve setting");

    println!("   ✅ Retrieved setting with UUID domain_object_id");
    assert_eq!(retrieved.domain_object_id, storage_uuid.to_string());
}

#[tokio::test]
async fn test_domain_object_id_gts_format() {
    let (service, settings_repo, _gts_repo) = create_test_service_with_repos();
    let gts_type = create_test_gts_type();
    let tenants = TestTenantHierarchy::new();
    let tenant_id = tenants.partner3_customer1_sanit;
    let agent_gts = "gts.a.p.agent.v1.0~backup.agent.v2.1";

    print_test_header(
        "test_domain_object_id_gts_format",
        &[
            "Verify that a GTS string domain_object_id can be used to upsert and retrieve a setting.",
            "This models a resource identified by a GTS (e.g., agent type).",
        ],
    );
    tenants.print_structure();
    println!("\n📝 Stage 1: Setup");
    println!("   Tenant: {} (San-iT)", tenant_id);
    println!("   Domain Object (GTS): {}", agent_gts);

    println!("\n📝 Stage 2: Register GTS type");
    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    settings_repo.print_state("Initial state");

    // Create setting with GTS domain_object_id (e.g., agent type)
    let agent_data = serde_json::json!({
        "agent_type": "backup",
        "version": "2.1",
        "auto_update": true
    });

    println!("\n📝 Stage 3: Upsert setting for GTS domain object");
    print_json("Data", &agent_data);
    let setting = service
        .upsert_setting(&gts_type.r#type, tenant_id, agent_gts, agent_data.clone())
        .await
        .expect("Failed to create setting with GTS domain_object_id");

    settings_repo.print_state("After creating GTS-based setting");

    assert_eq!(setting.domain_object_id, agent_gts);
    assert_eq!(setting.data, agent_data);

    // Retrieve the setting
    println!("\n📝 Stage 4: Retrieve setting");
    let retrieved = service
        .get_setting(&gts_type.r#type, tenant_id, agent_gts)
        .await
        .expect("Failed to retrieve setting");

    println!("   ✅ Retrieved setting with GTS domain_object_id");
    assert_eq!(retrieved.domain_object_id, agent_gts);
}

#[tokio::test]
async fn test_domain_object_id_appcode_format() {
    let (service, settings_repo, _gts_repo) = create_test_service_with_repos();
    let gts_type = create_test_gts_type();
    let tenants = TestTenantHierarchy::new();
    let tenant_id = tenants.partner2_customer2_computergeeks;
    let app_code = "APP-BACKUP-2024";

    print_test_header(
        "test_domain_object_id_appcode_format",
        &[
            "Verify that an AppCode domain_object_id can be used to upsert and retrieve a setting.",
            "This models a resource identifier with a custom application code.",
        ],
    );
    tenants.print_structure();
    println!("\n📝 Stage 1: Setup");
    println!("   Tenant: {} (Computer Geeks)", tenant_id);
    println!("   Domain Object (AppCode): {}", app_code);

    println!("\n📝 Stage 2: Register GTS type");
    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    settings_repo.print_state("Initial state");

    // Create setting with AppCode domain_object_id
    let app_data = serde_json::json!({
        "application": "Backup Manager",
        "license_key": "ABC-123-XYZ",
        "max_concurrent_jobs": 10
    });

    println!("\n📝 Stage 3: Upsert setting for AppCode domain object");
    print_json("Data", &app_data);
    let setting = service
        .upsert_setting(&gts_type.r#type, tenant_id, app_code, app_data.clone())
        .await
        .expect("Failed to create setting with AppCode domain_object_id");

    settings_repo.print_state("After creating AppCode-based setting");

    assert_eq!(setting.domain_object_id, app_code);
    assert_eq!(setting.data, app_data);

    // Retrieve the setting
    println!("\n📝 Stage 4: Retrieve setting");
    let retrieved = service
        .get_setting(&gts_type.r#type, tenant_id, app_code)
        .await
        .expect("Failed to retrieve setting");

    println!("   ✅ Retrieved setting with AppCode domain_object_id");
    assert_eq!(retrieved.domain_object_id, app_code);
}

#[tokio::test]
async fn test_domain_object_id_all_formats_coexist() {
    let (service, settings_repo, _gts_repo) = create_test_service_with_repos();
    let gts_type = create_test_gts_type();
    let tenants = TestTenantHierarchy::new();
    let tenant_id = tenants.partner1_customer1_evergreen;

    print_test_header(
        "test_domain_object_id_all_formats_coexist",
        &[
            "Verify that UUID/GTS/AppCode/generic domain_object_id formats coexist independently for the same tenant/type.",
            "Each domain_object_id should map to a distinct setting record.",
        ],
    );
    tenants.print_structure();
    println!("\n📝 Stage 1: Setup");
    println!("   Tenant: {} (Evergreen and Lyra)", tenant_id);

    println!("\n📝 Stage 2: Register GTS type");
    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    // Create settings with all 4 formats
    let uuid_id = Uuid::new_v4().to_string();
    let gts_id = "gts.a.p.resource.v1.0~compute.vm.v1";
    let appcode_id = "APP-WEB-2024";
    let generic_id = "generic";

    let test_cases = vec![
        (uuid_id.as_str(), "UUID", serde_json::json!({"format": "uuid"})),
        (gts_id, "GTS", serde_json::json!({"format": "gts"})),
        (appcode_id, "AppCode", serde_json::json!({"format": "appcode"})),
        (generic_id, "Generic", serde_json::json!({"format": "generic"})),
    ];

    println!("\n📝 Stage 3: Create settings with all 4 domain_object_id formats");
    for (domain_id, format_name, data) in &test_cases {
        println!("   - {} format: {}", format_name, domain_id);
        print_json("Data", data);
        service
            .upsert_setting(&gts_type.r#type, tenant_id, domain_id, data.clone())
            .await
            .expect(&format!("Failed to create {} setting", format_name));
    }

    settings_repo.print_state("After creating all 4 format types");

    // Verify all settings are independent and retrievable
    println!("\n📝 Stage 4: Retrieve and verify each setting");
    for (domain_id, format_name, expected_data) in &test_cases {
        let setting = service
            .get_setting(&gts_type.r#type, tenant_id, domain_id)
            .await
            .expect(&format!("Failed to retrieve {} setting", format_name));

        assert_eq!(setting.domain_object_id, *domain_id);
        assert_eq!(setting.data, *expected_data);
        println!("   ✅ {} format verified", format_name);
    }

    println!("\n✅ All 4 domain_object_id formats coexist independently");
    assert_eq!(settings_repo.count_active(), 4);
}

#[tokio::test]
async fn test_find_by_domain_object() {
    let (service, settings_repo, _gts_repo) = create_test_service_with_repos();
    let tenants = TestTenantHierarchy::new();
    let tenant_id = tenants.partner3_connectwise;
    let storage_uuid = Uuid::new_v4().to_string();

    print_test_header(
        "test_find_by_domain_object",
        &[
            "Verify that repository find_by_domain_object returns settings across multiple types for the same domain_object_id.",
            "This writes 3 settings with different types, all keyed by the same storage UUID.",
        ],
    );
    tenants.print_structure();
    println!("\n📝 Stage 1: Setup");
    println!("   Tenant: {} (ConnectWise)", tenant_id);
    println!("   Domain Object (UUID): {}", storage_uuid);

    // Register multiple GTS types
    println!("\n📝 Stage 2: Register multiple types + upsert settings");
    for i in 1..=3 {
        let mut gts_type = create_test_gts_type();
        gts_type.r#type = format!("gts.a.p.sm.setting.v1.0~test.type.v{}", i);
        println!("   Type {}: {}", i, gts_type.r#type);
        service
            .register_gts_type(gts_type.clone())
            .await
            .expect("Failed to register GTS type");

        // Create setting for the same storage UUID with different types
        let data = serde_json::json!({"type_index": i});
        print_json("Data", &data);
        service
            .upsert_setting(&gts_type.r#type, tenant_id, &storage_uuid, data)
            .await
            .expect("Failed to create setting");
    }

    settings_repo.print_state("After creating 3 settings for same storage UUID");

    // Use repository method directly to find all settings for this domain object
    println!("\n📝 Stage 3: Query by domain_object_id via repository");
    let settings = settings_repo
        .find_by_domain_object(&storage_uuid)
        .await
        .expect("Failed to find by domain object");

    println!("\n✅ Found {} settings for storage UUID: {}", settings.len(), storage_uuid);
    assert_eq!(settings.len(), 3);

    // Verify all settings have the same domain_object_id
    for setting in &settings {
        assert_eq!(setting.domain_object_id, storage_uuid);
    }
}

// ===== Hard Delete Tests =====

#[tokio::test]
async fn test_hard_delete_removes_permanently() {
    let (service, settings_repo, _gts_repo) = create_test_service_with_repos();
    let gts_type = create_test_gts_type();
    let tenants = TestTenantHierarchy::new();
    let tenant_id = tenants.partner1_pax8;

    print_test_header(
        "test_hard_delete_removes_permanently",
        &[
            "Verify that hard_delete permanently removes a setting record after it has been soft deleted.",
            "This is a repository-level operation and should remove the row entirely.",
        ],
    );
    tenants.print_structure();
    println!("\n📝 Stage 1: Setup");
    println!("   Tenant: {} (Pax8)", tenant_id);

    // Register GTS type
    println!("\n📝 Stage 2: Register GTS type");
    println!("   Type: {}", gts_type.r#type);
    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    // Create setting
    let data = serde_json::json!({"key": "value"});
    println!("\n📝 Stage 3: Upsert setting");
    print_json("Data", &data);
    service
        .upsert_setting(&gts_type.r#type, tenant_id, "generic", data)
        .await
        .expect("Failed to upsert setting");

    // Soft delete first
    println!("\n📝 Stage 4: Soft delete");
    service
        .delete_setting(&gts_type.r#type, tenant_id, "generic")
        .await
        .expect("Failed to soft delete");

    // Hard delete
    println!("\n📝 Stage 5: Hard delete");
    settings_repo
        .hard_delete(&gts_type.r#type, tenant_id, "generic")
        .await
        .expect("Failed to hard delete");

    // Verify setting is completely gone
    println!("\n📝 Stage 6: Verify repository is empty");
    assert_eq!(settings_repo.count(), 0, "Setting should be completely removed");
}

// ===== Lock Setting Tests =====

#[tokio::test]
async fn test_lock_setting_prevents_deletion() {
    let (service, _settings_repo, _gts_repo) = create_test_service_with_repos();
    let gts_type = create_test_gts_type();
    let tenants = TestTenantHierarchy::new();
    let tenant_id = tenants.partner2_datto;

    print_test_header(
        "test_lock_setting_prevents_deletion",
        &[
            "Verify that a locked setting cannot be deleted, and deletion succeeds after unlocking.",
            "This test is lock-focused and prints lock state transitions.",
        ],
    );
    tenants.print_structure();
    println!("\n📝 Stage 1: Setup");
    println!("   Tenant: {} (Datto)", tenant_id);

    // Register GTS type
    println!("\n📝 Stage 2: Register GTS type");
    println!("   Type: {}", gts_type.r#type);
    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    // Create setting
    let data = serde_json::json!({"key": "value"});
    println!("\n📝 Stage 3: Upsert setting");
    print_json("Data", &data);
    service
        .upsert_setting(&gts_type.r#type, tenant_id, "generic", data)
        .await
        .expect("Failed to upsert setting");

    // Lock the setting
    println!("\n📝 Stage 4: Lock setting (read_only=true)");
    service
        .lock_setting(&gts_type.r#type, tenant_id, "generic", true)
        .await
        .expect("Failed to lock setting");

    // Verify setting is locked
    println!("   Locked?: {}", service.is_locked(&gts_type.r#type, tenant_id, "generic"));
    assert!(service.is_locked(&gts_type.r#type, tenant_id, "generic"));

    // Try to delete - should fail
    println!("\n📝 Stage 5: Attempt delete while locked (should fail)");
    let result = service.delete_setting(&gts_type.r#type, tenant_id, "generic").await;
    assert!(result.is_err(), "Should not be able to delete locked setting");

    // Unlock and try again
    println!("\n📝 Stage 6: Unlock and delete");
    service.unlock_setting(&gts_type.r#type, tenant_id, "generic");
    assert!(!service.is_locked(&gts_type.r#type, tenant_id, "generic"));

    // Now deletion should work
    service.delete_setting(&gts_type.r#type, tenant_id, "generic").await.expect("Should be able to delete after unlock");
}

// ===== Retention Enforcement Tests =====

#[tokio::test]
async fn test_retention_enforcement() {
    let (service, settings_repo, gts_repo) = create_test_service_with_repos();
    let tenants = TestTenantHierarchy::new();
    let tenant_id = tenants.partner3_customer2_sourcepass;

    print_test_header(
        "test_retention_enforcement",
        &[
            "Verify that enforce_retention hard-deletes soft-deleted settings whose deleted_at exceeds retention_period.",
            "This inserts a setting with deleted_at older than retention and expects one row deleted.",
        ],
    );
    tenants.print_structure();
    println!("\n📝 Stage 1: Setup");
    println!("   Tenant: {} (Sourcepass)", tenant_id);

    // Create GTS type with short retention period
    println!("\n📝 Stage 2: Create GTS type with short retention");
    let gts_type = GtsType {
        r#type: "gts.a.p.sm.setting.v1.0~retention.test.v1".to_string(),
        traits: GtsTraits {
            domain_type: DomainType::Tenant,
            events: EventConfig {
                audit: EventTarget::Self_,
                notification: EventTarget::None,
            },
            options: SettingOptions {
                retention_period: 1, // 1 day retention
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
    };

    println!("   Type: {}", gts_type.r#type);
    println!("   retention_period(days): {}", gts_type.traits.options.retention_period);

    gts_repo.create(&gts_type).await.expect("Failed to create GTS type");

    // Create and soft-delete a setting with old deleted_at timestamp
    println!("\n📝 Stage 3: Insert soft-deleted setting older than retention");
    let setting = Setting {
        r#type: gts_type.r#type.clone(),
        tenant_id,
        domain_object_id: "generic".to_string(),
        data: serde_json::json!({"key": "value"}),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        deleted_at: Some(chrono::Utc::now() - chrono::Duration::days(2)), // Deleted 2 days ago
    };

    settings_repo.upsert(&setting).await.expect("Failed to create setting");
    println!("✅ Created setting with old deleted_at timestamp");
    println!("Total settings before enforcement: {}", settings_repo.count());

    // Enforce retention
    println!("\n📝 Stage 4: Enforce retention");
    let deleted_count = service.enforce_retention().await.expect("Failed to enforce retention");
    println!("🗑️  Deleted {} expired settings", deleted_count);
    println!("Total settings after enforcement: {}", settings_repo.count());

    assert_eq!(deleted_count, 1, "Should have deleted 1 expired setting");
    assert_eq!(settings_repo.count(), 0, "All expired settings should be removed");
}

// ===== GTS Validation Tests =====

#[tokio::test]
async fn test_gts_validation_missing_prefix() {
    let (service, _settings_repo, _gts_repo) = create_test_service_with_repos();

    print_test_header(
        "test_gts_validation_missing_prefix",
        &["Verify that register_gts_type rejects a type string that does not start with 'gts.'."],
    );

    println!("\n📝 Stage 1: Build invalid GTS type (missing prefix)");
    let invalid_gts = GtsType {
        r#type: "invalid.type~test.v1".to_string(), // Missing "gts." prefix
        traits: GtsTraits {
            domain_type: DomainType::Tenant,
            events: EventConfig {
                audit: EventTarget::None,
                notification: EventTarget::None,
            },
            options: SettingOptions::default(),
            operation: None,
        },
        schema: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    println!("   Provided type: {}", invalid_gts.r#type);

    println!("\n📝 Stage 2: Attempt register");
    let result = service.register_gts_type(invalid_gts).await;
    
    assert!(result.is_err(), "Should reject GTS without 'gts.' prefix");
    
    println!("\n📝 Stage 3: Verify error details");
    match result {
        Err(SettingsError::InvalidGtsFormat { gts, details }) => {
            println!("✅ Correctly rejected: {}", details);
            assert!(details.contains("gts."), "Error should mention 'gts.' prefix");
            assert_eq!(gts, "invalid.type~test.v1");
        }
        _ => panic!("Expected InvalidGtsFormat error"),
    }
}

#[tokio::test]
async fn test_gts_validation_missing_tilde() {
    let (service, _settings_repo, _gts_repo) = create_test_service_with_repos();

    print_test_header(
        "test_gts_validation_missing_tilde",
        &["Verify that register_gts_type rejects a type string missing the '~' separator."],
    );

    println!("\n📝 Stage 1: Build invalid GTS type (missing '~')");
    let invalid_gts = GtsType {
        r#type: "gts.a.p.sm.setting.v1.0.no.tilde".to_string(), // Missing "~" separator
        traits: GtsTraits {
            domain_type: DomainType::Tenant,
            events: EventConfig {
                audit: EventTarget::None,
                notification: EventTarget::None,
            },
            options: SettingOptions::default(),
            operation: None,
        },
        schema: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    println!("   Provided type: {}", invalid_gts.r#type);

    println!("\n📝 Stage 2: Attempt register");
    let result = service.register_gts_type(invalid_gts).await;
    
    assert!(result.is_err(), "Should reject GTS without '~' separator");
    
    println!("\n📝 Stage 3: Verify error details");
    match result {
        Err(SettingsError::InvalidGtsFormat { gts, details }) => {
            println!("✅ Correctly rejected: {}", details);
            assert!(details.contains("~"), "Error should mention '~' separator");
            assert_eq!(gts, "gts.a.p.sm.setting.v1.0.no.tilde");
        }
        _ => panic!("Expected InvalidGtsFormat error"),
    }
}
