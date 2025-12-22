# Settings Service - Root/Admin Override Capability

## Why
Currently, the settings service enforces `is_value_overwritable=false` constraints strictly for all users, preventing any tenant from overriding parent settings when this flag is set. However, there are legitimate operational scenarios where root/admin users need to override these constraints for emergency fixes, compliance updates, or system-wide policy changes. Without this capability, administrators cannot perform critical operations that require bypassing inheritance restrictions.

## What Changes

### Add Root/Admin Override Capability
- Extend authentication context to include root/admin privilege detection
- Modify `upsert_setting()` to check for root/admin privileges before enforcing `is_value_overwritable=false`
- Root/admin users SHALL be able to override settings regardless of `is_value_overwritable=false` constraint
- Root/admin users SHALL be able to set settings at any tenant level in the hierarchy
- Add audit logging for all root/admin override operations
- Add integration tests for root/admin override scenarios

### Authentication Context Enhancement
- Read `X-Root-Admin` header or equivalent authentication context flag
- Propagate root/admin status through domain service operations
- Ensure proper authorization checks before allowing override

### Audit Trail
- All root/admin override operations SHALL be logged with:
  - User/client identifier performing the override
  - Tenant being modified
  - Setting type and domain_object_id
  - Timestamp of override
  - Reason/context (if provided)

## Impact

### Affected Specs
- `settings` capability (primary)

### Affected Code
- **Domain Service**: `modules/settings_service/src/domain/service.rs`
  - Modify `upsert_setting()` to check root/admin privileges
  - Add `is_root_admin` parameter or extract from authentication context
  - Skip `is_value_overwritable` enforcement when root/admin flag is true

- **Contract Layer**: `modules/settings_service/src/contract/`
  - Extend authentication context model to include root/admin flag
  - Update client trait methods to accept/propagate auth context

- **API Layer**: `modules/settings_service/src/api/rest/`
  - Extract `X-Root-Admin` header from request context
  - Pass root/admin status to domain service operations

- **Testing**: `modules/settings_service/tests/service_tests.rs`
  - Add test: `test_root_admin_can_override_non_overwritable_setting`
  - Add test: `test_root_admin_override_is_audited`
  - Add test: `test_non_admin_cannot_override_non_overwritable_setting` (existing behavior)

### Breaking Changes
None - this is an additive enhancement that preserves existing behavior for non-admin users.

### Dependencies
- Requires authentication context to include root/admin privilege information
- May require coordination with account-server for role/privilege validation
