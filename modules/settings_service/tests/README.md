# Settings Service Test Suite

## Overview

The test suite for `settings_service` uses **in-memory mock repositories** (not SQLite or any database) to test business logic in isolation.

## Running Tests with Verbose Output

The test cases have been enhanced with verbose logging to show the state of the SettingsRepository during test execution.

### Run Individual Tests

```bash
# Test basic upsert operation
cargo test -p settings_service test_upsert_setting -- --nocapture

# Test soft delete functionality
cargo test -p settings_service test_delete_setting -- --nocapture

# Test inheritance override behavior
cargo test -p settings_service test_inheritance_override -- --nocapture

# Test multiple settings by type
cargo test -p settings_service test_get_settings_by_type -- --nocapture
```

### Run All Tests

```bash
# Run all tests with verbose output
cargo test -p settings_service -- --nocapture

# Run all tests (quiet mode)
cargo test -p settings_service
```

## Verbose Output Features

The enhanced tests provide:

### 1. **Repository State Snapshots**
Shows complete state of the SettingsRepository at key points:
- Initial state (empty)
- After creating settings
- After updates/deletes
- After inheritance resolution

### 2. **Detailed Setting Information**
For each setting in the repository:
- Composite key (type:tenant_id:domain_object_id)
- Type (GTS identifier)
- Tenant ID
- Domain Object ID
- Data (pretty-printed JSON)
- Created timestamp
- Updated timestamp
- Deleted timestamp (if soft-deleted)

### 3. **Statistics**
- Total settings count
- Active settings count (non-deleted)
- Tenant hierarchy visualization

### 4. **Test Flow Indicators**
- 🧪 Test name
- 📝 Operations being performed
- 🗑️ Delete operations
- 🔍 Inheritance resolution
- ✅ Success indicators
- 🏢 Hierarchy structure

## Example Output

### Soft Delete Test
```
🧪 TEST: test_delete_setting
Tenant ID: 4f0f72e2-1cad-414a-99dd-901cf18a5792

========== SettingsRepository State: After creating setting ==========
Total settings: 1
  Key: gts.a.p.sm.setting.v1.0~test.setting.v1:4f0f72e2-1cad-414a-99dd-901cf18a5792:generic
    Type: gts.a.p.sm.setting.v1.0~test.setting.v1
    Tenant ID: 4f0f72e2-1cad-414a-99dd-901cf18a5792
    Domain Object ID: generic
    Data: {"key": "value"}
    Created: 2025-12-10 06:26:28.544214 UTC
    Updated: 2025-12-10 06:26:28.544214 UTC
    Deleted: None
====================================================

✅ Active settings: 1

🗑️  Deleting setting...

========== SettingsRepository State: After soft delete ==========
Total settings: 1
  Key: gts.a.p.sm.setting.v1.0~test.setting.v1:4f0f72e2-1cad-414a-99dd-901cf18a5792:generic
    Deleted: Some(2025-12-10T06:26:28.544284Z)
====================================================

✅ Active settings: 0
📊 Total settings (including deleted): 1
```

### Inheritance Override Test
```
🧪 TEST: test_inheritance_override

🏢 Creating tenant hierarchy:
  Root:       7fb938a5-b63b-471d-af14-f8c809ba7ec4
  ├─ Child:   e15a4341-b274-4da5-96a2-3effe77e4230
  └─ Grandchild: 47f8a6db-c81b-4ccb-97ed-279a8c0cdc6d

📝 Setting value at ROOT level...
========== SettingsRepository State: After root setting ==========
Total settings: 1
  Data: {"level": "root", "value": 100}
====================================================

📝 OVERRIDING value at CHILD level...
========== SettingsRepository State: After child override ==========
Total settings: 2
  [Shows both root and child settings]
====================================================

✅ Grandchild inherited from:
   Tenant: e15a4341-b274-4da5-96a2-3effe77e4230 (should be child, not root)
   Data: {"level": "child", "value": 200}
```

## Mock Repository Implementation

The tests use custom mock implementations:

- **MockSettingsRepo**: In-memory HashMap-based settings storage
- **MockGtsTypeRepo**: In-memory HashMap-based GTS type registry
- **MockTenancyHierarchy**: In-memory tenant hierarchy for inheritance testing

### Key Methods

```rust
// Print repository state with context
settings_repo.print_state("After upsert");

// Get counts
let total = settings_repo.count();
let active = settings_repo.count_active();
```

## Test Categories

### Basic CRUD Operations
- `test_register_and_get_gts_type` - GTS type registration
- `test_upsert_setting` - Create/update settings
- `test_get_setting` - Retrieve settings
- `test_delete_setting` - Soft delete settings

### Validation
- `test_upsert_setting_without_gts_type` - Error handling
- `test_json_schema_validation` - Schema validation

### Multi-tenant Operations
- `test_get_settings_by_type` - Query across tenants
- `test_list_gts_types` - List all types

### Inheritance Tests
- `test_inheritance_basic` - Basic parent-child inheritance
- `test_inheritance_override` - Child overriding parent values
- `test_inheritance_barrier` - Blocking inheritance
- `test_inheritance_not_overwritable` - Non-overwritable settings
- `test_multi_level_inheritance` - 5-level hierarchy
- `test_sibling_isolation` - Sibling tenant isolation

## Database Integration Tests

For actual database testing (SQLite, PostgreSQL, MySQL), use:

```bash
make test-sqlite    # SQLite integration tests
make test-pg        # PostgreSQL integration tests
make test-mysql     # MySQL integration tests
make test-all       # All database tests
```

These tests use real database connections and the actual repository implementations from `src/infra/storage/repositories.rs`.
