# Implementation Tasks

## 1. Contract Model Updates
- [ ] 1.1 Add `default_value: Option<serde_json::Value>` field to `GtsType` struct
- [ ] 1.2 Update `GtsType` Default implementation to include `default_value: None`
- [ ] 1.3 Add validation that default_value matches schema if both are present

## 2. Domain Layer - Hierarchy Client
- [ ] 2.1 Create `src/domain/hierarchy.rs` with `TenantHierarchyClient` trait
- [ ] 2.2 Define `get_parent_tenant(tenant_id: Uuid) -> Result<Option<Uuid>>`
- [ ] 2.3 Define `validate_tenant_exists(tenant_id: Uuid) -> Result<bool>`
- [ ] 2.4 Define `get_tenant_path(tenant_id: Uuid) -> Result<Vec<Uuid>>` (child to root)
- [ ] 2.5 Create `MockTenantHierarchyClient` for testing
- [ ] 2.6 Add hierarchy client to Service constructor

## 3. Domain Service - Tenant Validation
- [ ] 3.1 Add tenant validation in `upsert_setting()` before creating setting
- [ ] 3.2 Return `SettingsError::TenantNotFound` if tenant doesn't exist
- [ ] 3.3 Add tenant validation in `lock_setting()`
- [ ] 3.4 Add integration tests for tenant validation

## 4. Domain Service - Enhanced Inheritance
- [ ] 4.1 Refactor `get_setting()` to call new `resolve_setting()` method
- [ ] 4.2 Implement `resolve_setting()` with full hierarchy traversal:
  - [ ] 4.2.1 Check explicit setting for (tenant_id, domain_object_id)
  - [ ] 4.2.2 Check generic setting for (tenant_id, "generic")
  - [ ] 4.2.3 Get tenant path from hierarchy client
  - [ ] 4.2.4 Traverse parent tenants checking `is_value_inheritable`
  - [ ] 4.2.5 Stop at `is_barrier_inheritance` boundaries
  - [ ] 4.2.6 Return default value from GTS type if no setting found
  - [ ] 4.2.7 Return NotFound if no default value
- [ ] 4.3 Update `get_inherited_setting()` to use real hierarchy client
- [ ] 4.4 Add `get_default_value()` helper method

## 5. API Layer - DTOs
- [ ] 5.1 Add `default_value` field to `GtsTypeDto`
- [ ] 5.2 Add `default_value` field to `UpsertGtsTypeRequest`
- [ ] 5.3 Update mapper to handle default_value conversion
- [ ] 5.4 Update OpenAPI schemas with default_value field

## 6. Storage Layer
- [ ] 6.1 Add `default_value` column to `gts_types` table migration
- [ ] 6.2 Update `GtsTypeEntity` to include default_value field
- [ ] 6.3 Update storage mapper to serialize/deserialize default_value
- [ ] 6.4 Test default_value persistence and retrieval

## 7. Error Handling
- [ ] 7.1 Add `TenantNotFound` variant to `SettingsError`
- [ ] 7.2 Add `HierarchyServiceError` variant for hierarchy client failures
- [ ] 7.3 Map hierarchy errors to appropriate HTTP status codes
- [ ] 7.4 Add error scenarios to API documentation

## 8. Testing
- [ ] 8.1 Add unit tests for tenant validation
- [ ] 8.2 Add unit tests for hierarchy traversal with mock client
- [ ] 8.3 Add unit tests for default value resolution
- [ ] 8.4 Add integration tests for complete resolution flow
- [ ] 8.5 Add tests for barrier inheritance with hierarchy
- [ ] 8.6 Add tests for non-overwritable settings with hierarchy
- [ ] 8.7 Update existing tests to use mock hierarchy client

## 9. Documentation
- [ ] 9.1 Update openspec tasks.md to mark hierarchy integration as complete
- [ ] 9.2 Add code comments explaining resolution priority
- [ ] 9.3 Update README with hierarchy integration details
- [ ] 9.4 Document account server API contract requirements

## 10. Integration Preparation
- [ ] 10.1 Define gRPC/REST contract for account server hierarchy API
- [ ] 10.2 Create placeholder for real hierarchy client implementation
- [ ] 10.3 Add configuration for hierarchy service endpoint
- [ ] 10.4 Add feature flag for hierarchy integration (default: mock)
