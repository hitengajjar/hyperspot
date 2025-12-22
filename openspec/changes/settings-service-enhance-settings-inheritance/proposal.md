# Change: Enhance Settings Service with Full Inheritance Implementation and Test Coverage

## Why
The settings_service currently has basic GTS type management and setting storage, but lacks:
1. **Full inheritance resolution logic** - The domain service doesn't implement tenant hierarchy traversal for inherited settings
2. **Overwritability enforcement** - The `is_value_overwritable` flag is not enforced when child tenants try to override parent settings
3. **Domain object ID validation** - Missing validation for UUID, GTS, AppCode, and "generic" formats
4. **Comprehensive E2E tests** - Python E2E tests need to cover inheritance scenarios
5. **Production-ready hierarchy integration** - Need integration with tenant service for real hierarchy data

The test infrastructure was added (mock hierarchy, inheritance test cases) but the production implementation needs to be completed.

## What Changes
- **Domain Service Enhancement**:
  - Add `get_inherited_setting()` method that walks up tenant hierarchy
  - Implement `is_value_overwritable` validation in `upsert_setting()`
  - Add domain_object_id format validation (UUID, GTS, AppCode, "generic")
  - Add hierarchy service integration via ClientHub

- **Repository Layer**:
  - Add methods to support inheritance queries
  - Optimize queries for hierarchy traversal

- **API Layer**:
  - Add GET `/settings/{type}/inherited` endpoint for resolved inherited values
  - Add query parameter `?resolve_inheritance=true` to existing endpoints
  - Update DTOs to include inheritance metadata

- **Test Coverage**:
  - Convert Rust inheritance tests to Python E2E tests
  - Add domain_object_id format validation tests
  - Add overwritability enforcement tests
  - Add hierarchy integration tests

- **Documentation**:
  - Update spec with inheritance resolution requirements
  - Document hierarchy service integration
  - Add API examples for inheritance queries

## Impact
- **Affected specs**: `settings` (primary)
- **Affected code**:
  - `modules/settings_service/src/domain/service.rs` - Add inheritance logic
  - `modules/settings_service/src/domain/repository.rs` - Add hierarchy queries
  - `modules/settings_service/src/api/rest/handlers.rs` - Add inheritance endpoints
  - `modules/settings_service/src/api/rest/dto.rs` - Add inheritance metadata
  - `testing/e2e/modules/settings_service/` - Add Python inheritance tests
- **Breaking changes**: None (additive only)
- **Dependencies**: Requires tenant service client for hierarchy data
