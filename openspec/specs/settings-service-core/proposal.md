# Settings Service - Complete Implementation

## Why
Implement a comprehensive settings management service as a HyperSpot module using ModKit framework. Settings are identified by GTS (Global Type System) identifiers with configuration metadata defined as GTS traits, providing a clean, type-safe configuration management system with full tenant hierarchy inheritance support.

## What Changes

### Initial Implementation (Phase 1)
- Create HyperSpot module `settings_service` with ModKit capabilities (db, rest)
- Implement DDD-light architecture (contract/, domain/, infra/, api/ layers)
- Create `gts_types` table to store GTS type definitions with traits
- Create `settings` table: `type` (GTS), `tenant_id`, `domain_object_id`, `data`, timestamps
- Implement Setting and GtsType domain models
- Implement repository pattern with SeaORM
- Implement REST API endpoints via api_ingress:
  - GET/PUT/DELETE `/settings/{setting_type}`
  - PUT `/settings/{setting_type}/lock`
  - POST/GET/PUT/DELETE `/gts-types`
- Implement native client contract for inter-module communication
- Support JSON Schema validation for setting values
- Support OData filtering for list endpoints

### Inheritance & Validation Enhancement (Phase 2)
- Add full inheritance resolution logic with tenant hierarchy traversal
- Implement `is_value_overwritable` enforcement when child tenants override parent settings
- Add domain_object_id format validation (UUID, GTS, AppCode, "generic")
- Add GET `/settings/{type}/inherited` endpoint for resolved inherited values
- Add `?resolve_inheritance=true` query parameter
- Update DTOs to include inheritance metadata (`inherited_from`, `is_inherited`, `inheritance_depth`)
- Integrate with tenant service via ClientHub for hierarchy data
- Add comprehensive Python E2E tests for inheritance scenarios

### Additional Features
- Event publishing for configuration changes based on GTS traits
- Soft delete with retention period enforcement
- Hard delete for permanent removal
- Pagination support for list operations
- Multi-format domain_object_id support (UUID, GTS, AppCode, "generic")
- Repository query methods (find_by_domain_object, find_by_tenant, find_by_type)
- GTS format validation with detailed error messages

## Impact

### Affected Specs
- `settings` capability (primary)

### Affected Code
- **New Module**: `modules/settings_service/`
  - Module: ModKit module with db and rest capabilities
  - Database: Create gts_types and settings tables with SeaORM migrations
  - Domain: Implement Setting and GtsType entities with business logic
  - Contract: Native SettingsApi trait for inter-module communication
  - Repository: SeaORM-based repositories with OData filtering
  - API: REST endpoints registered with api_ingress using OperationBuilder
  - Events: Event generation from GTS traits
  - Configuration: Typed configuration with sensible defaults

- **Domain Service**: `src/domain/service.rs`
  - Add inheritance resolution logic
  - Add overwritability enforcement
  - Add domain_object_id validation
  - Add hierarchy service integration

- **Repository Layer**: `src/domain/repository.rs`
  - Add hierarchy query support
  - Add find_by_domain_object, find_by_tenant methods

- **API Layer**: `src/api/rest/`
  - Add inheritance endpoints
  - Add inheritance metadata to DTOs
  - Add query parameter support

- **Testing**: `testing/e2e/modules/settings_service/`
  - Add Python E2E tests for inheritance
  - Add domain_object_id validation tests
  - Add overwritability enforcement tests

### Breaking Changes
None (additive only)

### Dependencies
- Requires tenant service client for hierarchy data (Phase 2)
- SeaORM for database operations
- ModKit framework for module capabilities
- api_ingress for REST endpoint registration
