# Implementation Tasks - Settings Service

## 1. Database Schema Changes
- [x] 1.1 Create gts_types table: (type PK, traits JSONB, schema JSONB, created_at, updated_at)
- [x] 1.2 Create settings table with GTS schema: (type, tenant_id, domain_object_id, data, created_at, updated_at, deleted_at)
- [x] 1.3 Set composite primary key on settings: (type, tenant_id, domain_object_id)
- [x] 1.4 Add foreign key: settings.type -> gts_types.type
- [x] 1.5 Add index on settings.type for GTS prefix filtering
- [x] 1.6 Add indexes on tenant_id and domain_object_id for lookups
- [x] 1.7 Create test data with GTS registrations and settings

## 2. Domain Model Implementation
- [x] 2.1 Create Setting entity matching GTS spec:
  - type: String (GTS)
  - tenant_id: UUID
  - domain_object_id: String (UUID | GTS | AppCode | "generic")
  - data: JSON
  - created_at, updated_at, deleted_at: Timestamp
- [x] 2.2 Create GtsType entity for registry:
  - type: String (GTS)
  - traits: JSON (domain_type, events, options, operations, legacy_name, legacy_namespace)
  - schema: JSON (validation schema + `x-gts-traits` extension)
  - created_at, updated_at: Timestamp
- [x] 2.3 Create Setting DTO matching GTS spec
- [x] 2.4 Create GtsType DTO for registration API
- [x] 2.5 Add GTS trait parser for domain_type, events, options, operations, legacy_name, legacy_namespace (including `x-gts-traits` extraction)
- [x] 2.6 Stub domain service integration helper (returns success + TODO log)

## 3. Repository Layer Implementation
- [x] 3.1 Create GTS registry repository with CRUD operations
- [x] 3.2 Create settings repository for GTS schema
- [x] 3.3 Implement queries using composite key (type, tenant_id, domain_object_id)
- [x] 3.4 Add filtering by type (GTS prefix matching)
- [x] 3.5 Add foreign key validation for settings.type
- [x] 3.6 Add find_by_domain_object method
- [x] 3.7 Add find_by_tenant method
- [x] 3.8 Add soft_delete and hard_delete methods
- [x] 3.9 Add list_all with pagination support
- [x] 3.10 Create repository tests for GTS-based operations

## 4. Domain Service Implementation
- [x] 4.1 Create settings service to use GTS trait configuration
- [x] 4.2 Add GTS trait resolution logic (domain_type, events, options)
- [x] 4.3 Implement basic inheritance logic from GTS options traits
- [x] 4.4 Add GTS format validation (validate_gts_format)
- [x] 4.5 Add JSON schema validation
- [x] 4.6 Create service tests for GTS-based operations
- [x] 4.7 Add get_inherited_setting() method with hierarchy traversal
- [x] 4.8 Implement is_value_overwritable enforcement in upsert_setting()
- [x] 4.9 Add domain_object_id format validation (UUID, GTS, AppCode, "generic")
- [ ] 4.10 Add TenantHierarchyService client integration via ClientHub
- [x] 4.11 Add placeholder audit/notification emitters that return success and log TODO
- [x] 4.12 Add placeholder domain service notifier (see Task 2.6) wiring into service lifecycle

## 5. API Layer Implementation - GTS Type Management
- [x] 5.1 Implement GTS Type APIs:
  - [x] 5.1.1 `GET /cti-types` - List all GTS types
  - [x] 5.1.2 `POST /cti-types` - Create new GTS type
  - [x] 5.1.3 `GET /cti-types/{type_id}` - Get GTS type by ID
  - [x] 5.1.4 `PUT /cti-types/{type_id}` - Update GTS type
  - [x] 5.1.5 `DELETE /cti-types/{type_id}` - Delete GTS type

- [x] 6.1 Implement GTS Settings APIs:
  - [x] 6.1.1 `GET /settings` - List all settings with filters
  - [x] 6.1.2 `GET /settings/{setting_type}` - Get setting by GTS
  - [x] 6.1.3 `PUT /settings/{setting_type}` - Update setting value
  - [x] 6.1.4 `DELETE /settings/{setting_type}` - Remove setting value
  - [x] 6.1.5 `PUT /settings/{setting_type}/lock` - Lock setting (compliance)
- [x] 6.2 Add query parameter support: tenant_id, domain_object_id, type, domain_type
- [x] 6.3 Update API responses to match GTS types
- [x] 6.4 Add validation: settings.type must exist in gts_types
- [x] 6.5 Add GET `/settings/{type}/inherited` endpoint
- [x] 6.6 Add `?resolve_inheritance=true` query parameter
- [x] 6.7 Update DTOs to include inheritance metadata
- [ ] 6.8 Implement v1 endpoints (`/api/settings/v1/settings/{gts_id}`, `:batch`, `:lock`) with full query surface (lod, explicit_only, jquery, etc.)
- [ ] 6.9 Align v1 API paths with DNA REST guidelines (/api/settings/v1/...)
- [ ] 6.10 Add `state` query parameter support (ALL, ENABLED, DISABLED) with ENABLED as default
- [ ] 6.11 Add `lod` (Level of Detail) parameter support (BASIC, FULL)
- [ ] 6.12 Implement bidirectional pagination with `before` and `after` cursors
- [ ] 6.13 Add mutual exclusivity validation for `tenant_id` and `subtree_root_id` parameters
- [ ] 6.14 Support multiple settings response format (single object or array)

## 7. Configuration and Validation
- [x] 7.1 Create GTS type definition format with traits (JSON)
- [x] 7.2 Implement GTS trait parser: domain_type, events, options, operations
- [x] 7.3 Add GTS format validation (gts.a.p.sm.setting.v1.0~vendor.app.feature.v1.0)
- [x] 7.4 Create static config files using GTS types
- [x] 7.5 Add schema validation for setting data per GTS type
- [ ] 7.6 Support `x-gts-traits` schema extension in config loader
- [ ] 7.7 Implement typed authentication context (tenant UUID/ID, user/client IDs, role capabilities, namespace hints)
- [ ] 7.8 Implement RBAC middleware for role enforcement (Admin, Cloud User, self-manage)
- [ ] 7.9 Implement scope resolution compatible with Account Server contracts
- [ ] 7.10 Support `allow_no_auth` trait for unauthenticated GET requests
- [ ] 7.11 Inject auth context into request extensions (no custom HTTP headers)
- [ ] 7.12 Ensure handlers read tenant/user/client metadata from auth context

## 8. Event System Implementation
- [ ] 8.1 Update audit event generation to read from GTS traits
- [ ] 8.2 Update notification event generation to read from GTS traits
- [ ] 8.3 Implement event target resolution (SELF, SUBROOT, NONE) from traits
- [ ] 8.4 Create event structures using GTS type field
- [ ] 8.5 Replace placeholder emitters with real Event Manager integration (after Phase 3)
- [ ] 8.6 Add reporting export placeholder functions (Reporting System) that return success + TODO log
- [ ] 8.7 Document steps required to wire real reporting/analytics integration once available

## 9. Testing - Unit and Integration
- [x] 9.1 Create unit tests for GTS trait parsing
- [x] 9.2 Create unit tests for GtsType entity and repository
- [x] 9.3 Create unit tests for GTS Setting entity
- [x] 9.4 Create integration tests for GTS registration APIs
- [x] 9.5 Create integration tests for GTS settings APIs
- [x] 9.6 Add validation tests: settings.type must exist in registry
- [x] 9.7 Add API tests for all endpoints (GTS registration + settings)
- [x] 9.8 Add tests for domain_object_id formats (UUID, GTS, AppCode, generic)
- [x] 9.9 Add tests for tenant hierarchy inheritance
- [x] 9.10 Add tests for find_by_domain_object repository method
- [x] 9.11 Add tests for soft delete and retention

## 10. Testing - Python E2E Tests
- [ ] 10.1 Convert test_inheritance_basic to Python E2E
- [ ] 10.2 Convert test_inheritance_override to Python E2E
- [ ] 10.3 Convert test_inheritance_barrier to Python E2E
- [ ] 10.4 Convert test_inheritance_not_overwritable to Python E2E
- [ ] 10.5 Convert test_multi_level_inheritance to Python E2E
- [ ] 10.6 Convert test_sibling_isolation to Python E2E
- [ ] 10.7 Add domain_object_id format validation E2E tests
- [ ] 10.8 Add overwritability enforcement E2E tests
- [ ] 10.9 Add helper functions for hierarchy setup in E2E tests

## 11. Documentation
- [x] 11.1 Document GTS API endpoints with examples
- [x] 11.2 Document GTS type definition format with trait examples
- [x] 11.3 Update architecture documentation
- [x] 11.4 Document API specification and usage patterns
- [x] 11.5 Add OpenSpec requirements and scenarios
- [ ] 11.6 Add sequence diagrams for inheritance resolution flow
- [ ] 11.7 Document hierarchy service integration
- [ ] 11.8 Document Reporting System streaming contracts and header expectations
- [ ] 11.9 Document placeholder integration points (domain + events) with TODO guidance
- [ ] 11.10 Document reporting placeholder behavior and migration plan to real Reporting System/reports service
- [ ] 11.11 Generate OpenAPI 3.0 specification aligned with DNA guidelines and ModKit patterns
- [ ] 11.12 Document query parameter validation rules and mutual exclusivity constraints
- [ ] 11.13 Document OAuth2 security schemes and allow_no_auth mechanism

## 12. Code Quality and Optimization
- [x] 12.1 Add inline documentation for complex logic
- [x] 12.2 Ensure error messages are clear and actionable
- [x] 12.3 Run `cargo clippy` and address warnings
- [x] 12.4 Run `cargo fmt` for consistent formatting
- [ ] 12.5 Add logging for inheritance resolution steps
- [ ] 12.6 Review code for edge cases (circular references, orphaned tenants)
- [ ] 12.7 Performance testing for deep hierarchies (5+ levels)
- [ ] 12.8 Add caching for resolved inherited values (optional optimization)

## Status Summary
- **Phase 1 (Initial Implementation)**: ✅ Complete
- **Phase 2 (Inheritance & Validation)**: ✅ Complete (Core features implemented; ClientHub-based TenantHierarchy integration pending)
- **Phase 3 (Events & Advanced Features)**: ⏳ Pending
