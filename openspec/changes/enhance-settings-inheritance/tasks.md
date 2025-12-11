# Implementation Tasks

## 1. Domain Service - Inheritance Resolution
- [ ] 1.1 Add `get_inherited_setting()` method to `domain/service.rs`
- [ ] 1.2 Implement tenant hierarchy traversal logic
- [ ] 1.3 Add `TenantHierarchyService` client integration via ClientHub
- [ ] 1.4 Implement inheritance barrier checking
- [ ] 1.5 Add caching for resolved inherited values (optional optimization)

## 2. Domain Service - Overwritability Enforcement
- [ ] 2.1 Add validation in `upsert_setting()` to check `is_value_overwritable`
- [ ] 2.2 Query parent settings to determine if override is allowed
- [ ] 2.3 Return appropriate error when override is blocked
- [ ] 2.4 Add unit tests for overwritability enforcement

## 3. Domain Service - Domain Object ID Validation
- [ ] 3.1 Create `validate_domain_object_id()` function
- [ ] 3.2 Add UUID format validation
- [ ] 3.3 Add GTS format validation (reuse existing `validate_gts_format`)
- [ ] 3.4 Add AppCode format validation (pattern: `[a-z0-9-]+`)
- [ ] 3.5 Allow "generic" as special case
- [ ] 3.6 Integrate validation into `upsert_setting()`
- [ ] 3.7 Add unit tests for all format validations

## 4. Repository Layer - Hierarchy Support
- [ ] 4.1 Add `find_setting_chain()` method to get settings up hierarchy
- [ ] 4.2 Optimize queries for hierarchy traversal
- [ ] 4.3 Add indexes for tenant hierarchy queries (migration)

## 5. REST API - Inheritance Endpoints
- [ ] 5.1 Add GET `/settings/{type}/inherited` endpoint
- [ ] 5.2 Add `?resolve_inheritance=true` query parameter to GET `/settings/{type}`
- [ ] 5.3 Update DTOs to include `inherited_from` metadata field
- [ ] 5.4 Add OpenAPI documentation for new endpoints
- [ ] 5.5 Update handlers to call inheritance resolution

## 6. Python E2E Tests - Inheritance
- [ ] 6.1 Convert `test_inheritance_basic` to Python
- [ ] 6.2 Convert `test_inheritance_override` to Python
- [ ] 6.3 Convert `test_inheritance_barrier` to Python
- [ ] 6.4 Convert `test_inheritance_not_overwritable` to Python
- [ ] 6.5 Convert `test_multi_level_inheritance` to Python
- [ ] 6.6 Convert `test_sibling_isolation` to Python
- [ ] 6.7 Add helper functions for hierarchy setup in E2E tests

## 7. Python E2E Tests - Domain Object ID Validation
- [ ] 7.1 Add test for UUID format validation
- [ ] 7.2 Add test for GTS format validation
- [ ] 7.3 Add test for AppCode format validation
- [ ] 7.4 Add test for "generic" special case
- [ ] 7.5 Add test for invalid formats (negative cases)

## 8. Python E2E Tests - Overwritability
- [ ] 8.1 Add test for blocked override scenario
- [ ] 8.2 Add test for allowed override scenario
- [ ] 8.3 Add test for error response validation

## 9. Documentation
- [ ] 9.1 Update spec.md with inheritance resolution requirements
- [ ] 9.2 Document hierarchy service integration
- [ ] 9.3 Add API examples for inheritance queries
- [ ] 9.4 Update README with inheritance feature description
- [ ] 9.5 Add sequence diagrams for inheritance resolution flow

## 10. Integration and Testing
- [ ] 10.1 Run all Rust unit tests
- [ ] 10.2 Run all Rust integration tests
- [ ] 10.3 Run all Python E2E tests
- [ ] 10.4 Verify OpenAPI spec generation
- [ ] 10.5 Manual testing of inheritance scenarios
- [ ] 10.6 Performance testing for deep hierarchies (5+ levels)

## 11. Code Quality
- [ ] 11.1 Add inline documentation for complex inheritance logic
- [ ] 11.2 Ensure error messages are clear and actionable
- [ ] 11.3 Add logging for inheritance resolution steps
- [ ] 11.4 Review code for edge cases (circular references, orphaned tenants)
- [ ] 11.5 Run `cargo clippy` and address warnings
- [ ] 11.6 Run `cargo fmt` for consistent formatting
