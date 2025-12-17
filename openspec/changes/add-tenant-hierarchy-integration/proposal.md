# Change: Add Tenant Hierarchy Integration with Account Server

## Why

The settings service currently lacks proper integration with the account server's tenant hierarchy. This creates several critical issues:

1. **No tenant validation**: `upsert_setting()` doesn't verify if the requested `tenant_id` exists in the account server's tenant hierarchy
2. **Incomplete inheritance**: `get_setting()` doesn't traverse the actual tenant hierarchy when a setting is not found locally
3. **Missing default values**: GTS type registration doesn't support default values, preventing fallback when no setting exists in the hierarchy
4. **Broken hierarchy traversal**: The current `get_inherited_setting()` is a placeholder without real hierarchy service integration

This violates the core principle that settings should honor the organizational tenant hierarchy managed by the account server.

## What Changes

### 1. Add Default Value Support to GTS Types
- Add `default_value` field to `GtsType` model (optional JSON value)
- Update GTS type registration API to accept default values
- Modify schema validation to validate default values against JSON schema

### 2. Integrate with Account Server Tenant Hierarchy
- Add `TenantHierarchyClient` trait for account server integration
- Implement tenant validation in `upsert_setting()` to verify tenant exists
- Enhance `get_setting()` to traverse hierarchy when setting not found locally
- Update `get_inherited_setting()` to use real hierarchy service

### 3. Implement Proper Inheritance Resolution
- Traverse hierarchy level-by-level from child to root
- Check `is_value_inheritable` at each level
- Stop at `is_barrier_inheritance` boundaries
- Fall back to default value if no setting found in hierarchy

### 4. Update Resolution Priority
The new priority order SHALL be:
1. Check requested tenant: explicit setting for (tenant_id, domain_object_id)
2. Check requested tenant: generic setting for (tenant_id, "generic")
3. For each parent tenant in hierarchy (child to root):
   - Check explicit setting for (parent_tenant_id, domain_object_id)
   - Check generic setting for (parent_tenant_id, "generic")
   - Stop if `is_barrier_inheritance=true`
   - Continue to next parent if no match
4. Return default value from GTS type definition
5. Return NotFound error

This ensures both exact and generic domain_object_id are checked at each hierarchy level before moving to the parent.

## Impact

**Affected specs**: `settings`

**Affected code**:
- `modules/settings_service/src/contract/model.rs` - Add `default_value` field
- `modules/settings_service/src/domain/service.rs` - Add hierarchy integration
- `modules/settings_service/src/domain/hierarchy.rs` - **NEW** hierarchy client trait
- `modules/settings_service/src/api/rest/dto.rs` - Add default value to DTOs
- `modules/settings_service/tests/` - Add hierarchy integration tests

**Breaking changes**: None (additive changes only)

**Dependencies**: Requires account server to expose tenant hierarchy API (gRPC or REST)
