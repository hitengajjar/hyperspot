//! Integration tests for Settings Service

use settings_service::contract::*;
use settings_service::domain::repository::{GtsTypeRepository, SettingsRepository};
use settings_service::domain::{NoOpEventPublisher, Service};
use std::sync::Arc;
use uuid::Uuid;

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

    // Register GTS type
    let registered = service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    assert_eq!(registered.r#type, gts_type.r#type);

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
    let tenant_id = Uuid::new_v4();

    println!("\n🧪 TEST: test_upsert_setting");
    println!("Tenant ID: {}", tenant_id);
    
    settings_repo.print_state("Initial state");

    // Register GTS type first
    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    // Upsert setting
    let data = serde_json::json!({"key": "value"});
    println!("\n📝 Upserting setting with data: {}", serde_json::to_string_pretty(&data).unwrap());
    
    let setting = service
        .upsert_setting(&gts_type.r#type, tenant_id, "generic", data.clone())
        .await
        .expect("Failed to upsert setting");

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
    let tenant_id = Uuid::new_v4();

    // Try to upsert setting without registering GTS type
    let data = serde_json::json!({"key": "value"});
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
    let tenant_id = Uuid::new_v4();

    // Register GTS type and create setting
    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    let data = serde_json::json!({"key": "value"});
    service
        .upsert_setting(&gts_type.r#type, tenant_id, "generic", data.clone())
        .await
        .expect("Failed to upsert setting");

    // Get setting
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
    let tenant_id = Uuid::new_v4();

    println!("\n🧪 TEST: test_delete_setting");
    println!("Tenant ID: {}", tenant_id);

    // Register GTS type and create setting
    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    let data = serde_json::json!({"key": "value"});
    service
        .upsert_setting(&gts_type.r#type, tenant_id, "generic", data)
        .await
        .expect("Failed to upsert setting");

    settings_repo.print_state("After creating setting");
    println!("✅ Active settings: {}", settings_repo.count_active());

    // Delete setting
    println!("\n🗑️  Deleting setting...");
    service
        .delete_setting(&gts_type.r#type, tenant_id, "generic")
        .await
        .expect("Failed to delete setting");

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
    let tenant_id = Uuid::new_v4();

    // Create GTS type with JSON schema
    let mut gts_type = create_test_gts_type();
    gts_type.schema = Some(serde_json::json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "number", "minimum": 0 }
        },
        "required": ["name"]
    }));

    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    // Valid data
    let valid_data = serde_json::json!({"name": "John", "age": 30});
    let result = service
        .upsert_setting(&gts_type.r#type, tenant_id, "generic", valid_data)
        .await;
    assert!(result.is_ok());

    // Invalid data (missing required field)
    let invalid_data = serde_json::json!({"age": 30});
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

    println!("\n🧪 TEST: test_get_settings_by_type");

    // Register GTS type
    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    settings_repo.print_state("Initial state");

    // Create settings for multiple tenants
    println!("\n📝 Creating settings for 3 different tenants...");
    for i in 1..=3 {
        let tenant_id = Uuid::new_v4();
        let data = serde_json::json!({"index": i});
        println!("  Creating setting {} for tenant {}", i, tenant_id);
        service
            .upsert_setting(&gts_type.r#type, tenant_id, "generic", data)
            .await
            .expect("Failed to upsert setting");
    }

    settings_repo.print_state("After creating 3 settings");

    // Get all settings of this type
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
        pub id: Uuid,
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

    // Create hierarchy: Root -> Child -> Grandchild
    let root_id = Uuid::new_v4();
    let child_id = Uuid::new_v4();
    let grandchild_id = Uuid::new_v4();

    hierarchy.add_tenant(root_id, "Root Tenant".to_string(), None);
    hierarchy.add_tenant(child_id, "Child Tenant".to_string(), Some(root_id));
    hierarchy.add_tenant(grandchild_id, "Grandchild Tenant".to_string(), Some(child_id));

    // Register GTS type with inheritance enabled
    let mut gts_type = create_test_gts_type();
    gts_type.r#type = "gts.a.p.sm.setting.v1.0~test.inheritance.v1".to_string();
    gts_type.traits.options.is_value_inheritable = true;
    gts_type.traits.options.is_value_overwritable = true;
    gts_type.traits.options.is_barrier_inheritance = false;

    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    // Set value at root level
    let root_data = serde_json::json!({"level": "root", "value": 100});
    service
        .upsert_setting(&gts_type.r#type, root_id, "generic", root_data.clone())
        .await
        .expect("Failed to set root setting");

    // Child should inherit from root
    let child_setting = resolve_inherited_setting(
        &service,
        &hierarchy,
        &gts_type.r#type,
        child_id,
        "generic",
    )
    .await;

    assert!(child_setting.is_some());
    assert_eq!(child_setting.unwrap().tenant_id, root_id); // Inherited from root

    // Grandchild should also inherit from root
    let grandchild_setting = resolve_inherited_setting(
        &service,
        &hierarchy,
        &gts_type.r#type,
        grandchild_id,
        "generic",
    )
    .await;

    assert!(grandchild_setting.is_some());
    assert_eq!(grandchild_setting.unwrap().tenant_id, root_id);
}

#[tokio::test]
async fn test_inheritance_override() {
    let (service, settings_repo, _gts_repo) = create_test_service_with_repos();
    let hierarchy = MockTenancyHierarchy::new();

    println!("\n🧪 TEST: test_inheritance_override");

    // Create hierarchy: Root -> Child -> Grandchild
    let root_id = Uuid::new_v4();
    let child_id = Uuid::new_v4();
    let grandchild_id = Uuid::new_v4();

    println!("\n🏢 Creating tenant hierarchy:");
    println!("  Root:       {}", root_id);
    println!("  ├─ Child:   {}", child_id);
    println!("  └─ Grandchild: {}", grandchild_id);

    hierarchy.add_tenant(root_id, "Root Tenant".to_string(), None);
    hierarchy.add_tenant(child_id, "Child Tenant".to_string(), Some(root_id));
    hierarchy.add_tenant(grandchild_id, "Grandchild Tenant".to_string(), Some(child_id));

    // Register GTS type with overwritable inheritance
    let mut gts_type = create_test_gts_type();
    gts_type.r#type = "gts.a.p.sm.setting.v1.0~test.override.v1".to_string();
    gts_type.traits.options.is_value_inheritable = true;
    gts_type.traits.options.is_value_overwritable = true;

    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    settings_repo.print_state("Initial state");

    // Set value at root level
    println!("\n📝 Setting value at ROOT level...");
    let root_data = serde_json::json!({"level": "root", "value": 100});
    service
        .upsert_setting(&gts_type.r#type, root_id, "generic", root_data)
        .await
        .expect("Failed to set root setting");

    settings_repo.print_state("After root setting");

    // Override at child level
    println!("\n📝 OVERRIDING value at CHILD level...");
    let child_data = serde_json::json!({"level": "child", "value": 200});
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

    println!("\n✅ Child setting retrieved:");
    println!("   Tenant: {}", child_setting.tenant_id);
    println!("   Data: {}", serde_json::to_string_pretty(&child_setting.data).unwrap());

    assert_eq!(child_setting.tenant_id, child_id);
    assert_eq!(child_setting.data["level"], "child");
    assert_eq!(child_setting.data["value"], 200);

    // Grandchild should inherit from child (not root)
    println!("\n🔍 Resolving inherited setting for GRANDCHILD...");
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
    
    println!("✅ Grandchild inherited from:");
    println!("   Tenant: {} (should be child, not root)", grandchild_setting.tenant_id);
    println!("   Data: {}", serde_json::to_string_pretty(&grandchild_setting.data).unwrap());
    
    assert_eq!(grandchild_setting.tenant_id, child_id); // Inherited from child
    assert_eq!(grandchild_setting.data["level"], "child");
}

#[tokio::test]
async fn test_inheritance_barrier() {
    let service = create_test_service();
    let hierarchy = MockTenancyHierarchy::new();

    // Create hierarchy: Root -> Child -> Grandchild
    let root_id = Uuid::new_v4();
    let child_id = Uuid::new_v4();
    let grandchild_id = Uuid::new_v4();

    hierarchy.add_tenant(root_id, "Root Tenant".to_string(), None);
    hierarchy.add_tenant(child_id, "Child Tenant".to_string(), Some(root_id));
    hierarchy.add_tenant(grandchild_id, "Grandchild Tenant".to_string(), Some(child_id));

    // Register GTS type with inheritance barrier
    let mut gts_type = create_test_gts_type();
    gts_type.r#type = "gts.a.p.sm.setting.v1.0~test.barrier.v1".to_string();
    gts_type.traits.options.is_value_inheritable = true;
    gts_type.traits.options.is_barrier_inheritance = true; // Barrier enabled

    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    // Set value at root level
    let root_data = serde_json::json!({"level": "root", "value": 100});
    service
        .upsert_setting(&gts_type.r#type, root_id, "generic", root_data)
        .await
        .expect("Failed to set root setting");

    // Child should NOT inherit due to barrier
    let child_setting = resolve_inherited_setting(
        &service,
        &hierarchy,
        &gts_type.r#type,
        child_id,
        "generic",
    )
    .await;

    assert!(child_setting.is_none()); // Barrier blocks inheritance
}

#[tokio::test]
async fn test_inheritance_not_overwritable() {
    let service = create_test_service();
    let hierarchy = MockTenancyHierarchy::new();

    // Create hierarchy: Root -> Child
    let root_id = Uuid::new_v4();
    let child_id = Uuid::new_v4();

    hierarchy.add_tenant(root_id, "Root Tenant".to_string(), None);
    hierarchy.add_tenant(child_id, "Child Tenant".to_string(), Some(root_id));

    // Register GTS type with non-overwritable inheritance
    let mut gts_type = create_test_gts_type();
    gts_type.r#type = "gts.a.p.sm.setting.v1.0~test.locked.v1".to_string();
    gts_type.traits.options.is_value_inheritable = true;
    gts_type.traits.options.is_value_overwritable = false; // Cannot override

    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    // Set value at root level
    let root_data = serde_json::json!({"level": "root", "value": 100});
    service
        .upsert_setting(&gts_type.r#type, root_id, "generic", root_data)
        .await
        .expect("Failed to set root setting");

    // Child can still create its own setting (implementation dependent)
    // In a full implementation, this might be blocked by business logic
    let child_data = serde_json::json!({"level": "child", "value": 200});
    let result = service
        .upsert_setting(&gts_type.r#type, child_id, "generic", child_data)
        .await;

    // This test documents current behavior - in production, you might want to
    // add validation to prevent overriding non-overwritable settings
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multi_level_inheritance() {
    let _service: Service = create_test_service();
    let hierarchy = MockTenancyHierarchy::new();

    // Create 5-level hierarchy
    let level1 = Uuid::new_v4();
    let level2 = Uuid::new_v4();
    let level3 = Uuid::new_v4();
    let level4 = Uuid::new_v4();
    let level5 = Uuid::new_v4();

    hierarchy.add_tenant(level1, "Level 1".to_string(), None);
    hierarchy.add_tenant(level2, "Level 2".to_string(), Some(level1));
    hierarchy.add_tenant(level3, "Level 3".to_string(), Some(level2));
    hierarchy.add_tenant(level4, "Level 4".to_string(), Some(level3));
    hierarchy.add_tenant(level5, "Level 5".to_string(), Some(level4));

    // Verify hierarchy
    let ancestors = hierarchy.get_ancestors(level5);
    assert_eq!(ancestors.len(), 4);
    assert_eq!(ancestors[0], level4);
    assert_eq!(ancestors[1], level3);
    assert_eq!(ancestors[2], level2);
    assert_eq!(ancestors[3], level1);

    // Verify descendants
    let descendants = hierarchy.get_descendants(level1);
    assert_eq!(descendants.len(), 4);
    assert!(descendants.contains(&level2));
    assert!(descendants.contains(&level5));

    // Verify ancestry check
    assert!(hierarchy.is_ancestor(level1, level5));
    assert!(hierarchy.is_ancestor(level3, level5));
    assert!(!hierarchy.is_ancestor(level5, level1));
}

#[tokio::test]
async fn test_sibling_isolation() {
    let service = create_test_service();
    let hierarchy = MockTenancyHierarchy::new();

    // Create hierarchy with siblings: Root -> Child1, Child2
    let root_id = Uuid::new_v4();
    let child1_id = Uuid::new_v4();
    let child2_id = Uuid::new_v4();

    hierarchy.add_tenant(root_id, "Root".to_string(), None);
    hierarchy.add_tenant(child1_id, "Child 1".to_string(), Some(root_id));
    hierarchy.add_tenant(child2_id, "Child 2".to_string(), Some(root_id));

    // Register GTS type
    let mut gts_type = create_test_gts_type();
    gts_type.r#type = "gts.a.p.sm.setting.v1.0~test.sibling.v1".to_string();

    service
        .register_gts_type(gts_type.clone())
        .await
        .expect("Failed to register GTS type");

    // Set value for child1
    let child1_data = serde_json::json!({"owner": "child1"});
    service
        .upsert_setting(&gts_type.r#type, child1_id, "generic", child1_data)
        .await
        .expect("Failed to set child1 setting");

    // Child2 should NOT see child1's setting
    let child2_result = service
        .get_setting(&gts_type.r#type, child2_id, "generic")
        .await;

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
    let tenant_id = Uuid::new_v4();
    let storage_uuid = Uuid::new_v4();

    println!("\n🧪 TEST: domain_object_id with UUID format");
    println!("Tenant ID: {}", tenant_id);
    println!("Storage UUID: {}", storage_uuid);

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

    println!("\n📝 Creating setting for storage UUID: {}", storage_uuid);
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
    let retrieved = service
        .get_setting(&gts_type.r#type, tenant_id, &storage_uuid.to_string())
        .await
        .expect("Failed to retrieve setting");

    println!("✅ Retrieved setting with UUID domain_object_id");
    assert_eq!(retrieved.domain_object_id, storage_uuid.to_string());
}

#[tokio::test]
async fn test_domain_object_id_gts_format() {
    let (service, settings_repo, _gts_repo) = create_test_service_with_repos();
    let gts_type = create_test_gts_type();
    let tenant_id = Uuid::new_v4();
    let agent_gts = "gts.a.p.agent.v1.0~backup.agent.v2.1";

    println!("\n🧪 TEST: domain_object_id with GTS format");
    println!("Tenant ID: {}", tenant_id);
    println!("Agent GTS: {}", agent_gts);

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

    println!("\n📝 Creating setting for agent GTS: {}", agent_gts);
    let setting = service
        .upsert_setting(&gts_type.r#type, tenant_id, agent_gts, agent_data.clone())
        .await
        .expect("Failed to create setting with GTS domain_object_id");

    settings_repo.print_state("After creating GTS-based setting");

    assert_eq!(setting.domain_object_id, agent_gts);
    assert_eq!(setting.data, agent_data);

    // Retrieve the setting
    let retrieved = service
        .get_setting(&gts_type.r#type, tenant_id, agent_gts)
        .await
        .expect("Failed to retrieve setting");

    println!("✅ Retrieved setting with GTS domain_object_id");
    assert_eq!(retrieved.domain_object_id, agent_gts);
}

#[tokio::test]
async fn test_domain_object_id_appcode_format() {
    let (service, settings_repo, _gts_repo) = create_test_service_with_repos();
    let gts_type = create_test_gts_type();
    let tenant_id = Uuid::new_v4();
    let app_code = "APP-BACKUP-2024";

    println!("\n🧪 TEST: domain_object_id with AppCode format");
    println!("Tenant ID: {}", tenant_id);
    println!("App Code: {}", app_code);

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

    println!("\n📝 Creating setting for AppCode: {}", app_code);
    let setting = service
        .upsert_setting(&gts_type.r#type, tenant_id, app_code, app_data.clone())
        .await
        .expect("Failed to create setting with AppCode domain_object_id");

    settings_repo.print_state("After creating AppCode-based setting");

    assert_eq!(setting.domain_object_id, app_code);
    assert_eq!(setting.data, app_data);

    // Retrieve the setting
    let retrieved = service
        .get_setting(&gts_type.r#type, tenant_id, app_code)
        .await
        .expect("Failed to retrieve setting");

    println!("✅ Retrieved setting with AppCode domain_object_id");
    assert_eq!(retrieved.domain_object_id, app_code);
}

#[tokio::test]
async fn test_domain_object_id_all_formats_coexist() {
    let (service, settings_repo, _gts_repo) = create_test_service_with_repos();
    let gts_type = create_test_gts_type();
    let tenant_id = Uuid::new_v4();

    println!("\n🧪 TEST: All domain_object_id formats coexisting");

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

    println!("\n📝 Creating settings with all 4 domain_object_id formats...");
    for (domain_id, format_name, data) in &test_cases {
        println!("  - {} format: {}", format_name, domain_id);
        service
            .upsert_setting(&gts_type.r#type, tenant_id, domain_id, data.clone())
            .await
            .expect(&format!("Failed to create {} setting", format_name));
    }

    settings_repo.print_state("After creating all 4 format types");

    // Verify all settings are independent and retrievable
    for (domain_id, format_name, expected_data) in &test_cases {
        let setting = service
            .get_setting(&gts_type.r#type, tenant_id, domain_id)
            .await
            .expect(&format!("Failed to retrieve {} setting", format_name));

        assert_eq!(setting.domain_object_id, *domain_id);
        assert_eq!(setting.data, *expected_data);
        println!("✅ {} format verified", format_name);
    }

    println!("\n✅ All 4 domain_object_id formats coexist independently");
    assert_eq!(settings_repo.count_active(), 4);
}

#[tokio::test]
async fn test_find_by_domain_object() {
    let (service, settings_repo, _gts_repo) = create_test_service_with_repos();
    let tenant_id = Uuid::new_v4();
    let storage_uuid = Uuid::new_v4().to_string();

    println!("\n🧪 TEST: find_by_domain_object repository method");
    println!("Storage UUID: {}", storage_uuid);

    // Register multiple GTS types
    for i in 1..=3 {
        let mut gts_type = create_test_gts_type();
        gts_type.r#type = format!("gts.a.p.sm.setting.v1.0~test.type.v{}", i);
        service
            .register_gts_type(gts_type.clone())
            .await
            .expect("Failed to register GTS type");

        // Create setting for the same storage UUID with different types
        let data = serde_json::json!({"type_index": i});
        service
            .upsert_setting(&gts_type.r#type, tenant_id, &storage_uuid, data)
            .await
            .expect("Failed to create setting");
    }

    settings_repo.print_state("After creating 3 settings for same storage UUID");

    // Use repository method directly to find all settings for this domain object
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
    let tenant_id = Uuid::new_v4();

    println!("\n🧪 TEST: test_hard_delete_removes_permanently");

    // Register GTS type
    service.register_gts_type(gts_type.clone()).await.expect("Failed to register GTS type");

    // Create setting
    let data = serde_json::json!({"key": "value"});
    service.upsert_setting(&gts_type.r#type, tenant_id, "generic", data).await.expect("Failed to upsert setting");

    // Soft delete first
    service.delete_setting(&gts_type.r#type, tenant_id, "generic").await.expect("Failed to soft delete");

    // Hard delete
    settings_repo.hard_delete(&gts_type.r#type, tenant_id, "generic").await.expect("Failed to hard delete");

    // Verify setting is completely gone
    assert_eq!(settings_repo.count(), 0, "Setting should be completely removed");
}

// ===== Lock Setting Tests =====

#[tokio::test]
async fn test_lock_setting_prevents_deletion() {
    let (service, _settings_repo, _gts_repo) = create_test_service_with_repos();
    let gts_type = create_test_gts_type();
    let tenant_id = Uuid::new_v4();

    println!("\n🧪 TEST: test_lock_setting_prevents_deletion");

    // Register GTS type
    service.register_gts_type(gts_type.clone()).await.expect("Failed to register GTS type");

    // Create setting
    let data = serde_json::json!({"key": "value"});
    service.upsert_setting(&gts_type.r#type, tenant_id, "generic", data).await.expect("Failed to upsert setting");

    // Lock the setting
    service.lock_setting(&gts_type.r#type, tenant_id, "generic", true).await.expect("Failed to lock setting");

    // Verify setting is locked
    assert!(service.is_locked(&gts_type.r#type, tenant_id, "generic"));

    // Try to delete - should fail
    let result = service.delete_setting(&gts_type.r#type, tenant_id, "generic").await;
    assert!(result.is_err(), "Should not be able to delete locked setting");

    // Unlock and try again
    service.unlock_setting(&gts_type.r#type, tenant_id, "generic");
    assert!(!service.is_locked(&gts_type.r#type, tenant_id, "generic"));

    // Now deletion should work
    service.delete_setting(&gts_type.r#type, tenant_id, "generic").await.expect("Should be able to delete after unlock");
}

// ===== Retention Enforcement Tests =====

#[tokio::test]
async fn test_retention_enforcement() {
    let (service, settings_repo, gts_repo) = create_test_service_with_repos();
    let tenant_id = Uuid::new_v4();

    println!("\n🧪 TEST: test_retention_enforcement");

    // Create GTS type with short retention period
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

    gts_repo.create(&gts_type).await.expect("Failed to create GTS type");

    // Create and soft-delete a setting with old deleted_at timestamp
    let mut setting = Setting {
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
    
    println!("\n🧪 TEST: test_gts_validation_missing_prefix");

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

    let result = service.register_gts_type(invalid_gts).await;
    
    assert!(result.is_err(), "Should reject GTS without 'gts.' prefix");
    
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
    
    println!("\n🧪 TEST: test_gts_validation_missing_tilde");

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

    let result = service.register_gts_type(invalid_gts).await;
    
    assert!(result.is_err(), "Should reject GTS without '~' separator");
    
    match result {
        Err(SettingsError::InvalidGtsFormat { gts, details }) => {
            println!("✅ Correctly rejected: {}", details);
            assert!(details.contains("~"), "Error should mention '~' separator");
            assert_eq!(gts, "gts.a.p.sm.setting.v1.0.no.tilde");
        }
        _ => panic!("Expected InvalidGtsFormat error"),
    }
}
