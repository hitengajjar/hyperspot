## ADDED Requirements

### Requirement: Default Value Support in GTS Types
GTS type definitions SHALL support optional default values that are returned when no setting exists in the hierarchy.

#### Scenario: Register GTS type with default value
- **WHEN** a GTS type is registered with a default_value field
- **THEN** the system SHALL store the default value
- **AND** the default value SHALL be validated against the JSON schema if present
- **AND** the default value SHALL be returned when no setting exists

#### Scenario: Default value validation
- **WHEN** a GTS type with both schema and default_value is registered
- **THEN** the system SHALL validate that default_value conforms to the schema
- **AND** registration SHALL fail if default_value is invalid
- **AND** error SHALL indicate schema validation failure

#### Scenario: Retrieve default value
- **WHEN** get_setting() is called and no setting exists in hierarchy
- **THEN** the system SHALL return the default value from GTS type
- **AND** response SHALL indicate the value is a default (not inherited)
- **AND** if no default value exists, return NotFound error

### Requirement: Tenant Validation via Account Server
The system SHALL validate tenant existence with the account server before creating or modifying settings.

#### Scenario: Validate tenant on upsert
- **WHEN** upsert_setting() is called with a tenant_id
- **THEN** the system SHALL query account server to verify tenant exists
- **AND** if tenant does not exist, return TenantNotFound error
- **AND** if tenant exists, proceed with setting creation
- **AND** validation SHALL occur before any database operations

#### Scenario: Validate tenant on lock
- **WHEN** lock_setting() is called with a tenant_id
- **THEN** the system SHALL verify tenant exists in account server
- **AND** if tenant does not exist, return TenantNotFound error
- **AND** lock operation SHALL only proceed for valid tenants

#### Scenario: Handle hierarchy service unavailable
- **WHEN** tenant validation is attempted but hierarchy service is unavailable
- **THEN** the system SHALL return HierarchyServiceError
- **AND** error SHALL include details about service unavailability
- **AND** operation SHALL NOT proceed with unvalidated tenant

### Requirement: Hierarchy-Aware Setting Resolution
The system SHALL traverse the actual tenant hierarchy from account server when resolving settings.

#### Scenario: Resolve with hierarchy traversal
- **WHEN** get_setting() is called and setting not found locally
- **THEN** the system SHALL query account server for parent tenant
- **AND** check parent tenant for the setting if is_value_inheritable=true
- **AND** continue traversing up to root tenant
- **AND** stop at is_barrier_inheritance boundaries
- **AND** return default value if no setting found in hierarchy
- **AND** return NotFound if no default value exists

#### Scenario: Resolution priority order
- **WHEN** resolving a setting value
- **THEN** the system SHALL check in order:
  1. Explicit setting for (requested_tenant_id, domain_object_id)
  2. Generic setting for (requested_tenant_id, "generic")
  3. For each parent tenant in hierarchy (from child to root):
     - Check explicit setting for (parent_tenant_id, domain_object_id)
     - Check generic setting for (parent_tenant_id, "generic")
     - Stop if is_barrier_inheritance=true
     - Continue to next parent if no match
  4. Default value from GTS type
  5. Return NotFound error
- **AND** first match SHALL be returned immediately
- **AND** both exact and generic domain_object_id SHALL be checked at each hierarchy level

#### Scenario: Respect barrier inheritance in hierarchy
- **WHEN** traversing hierarchy and encountering is_barrier_inheritance=true
- **THEN** the system SHALL stop traversal at that level
- **AND** SHALL NOT check ancestors beyond the barrier
- **AND** SHALL return default value if no setting found before barrier
- **AND** barrier SHALL apply even if parent has inheritable settings

#### Scenario: Respect non-overwritable settings
- **WHEN** a parent has a setting with is_value_overwritable=false
- **THEN** child tenant SHALL NOT be able to create override
- **AND** upsert_setting() SHALL return SettingNotOverwritable error
- **AND** parent setting SHALL remain unchanged
- **AND** error SHALL indicate which parent tenant has the restriction

### Requirement: Tenant Hierarchy Client Integration
The system SHALL provide an abstraction for communicating with the account server's tenant hierarchy service.

#### Scenario: Get parent tenant
- **WHEN** hierarchy client get_parent_tenant() is called
- **THEN** the system SHALL query account server for parent tenant ID
- **AND** return Some(parent_id) if parent exists
- **AND** return None if tenant is root
- **AND** return error if tenant does not exist

#### Scenario: Get tenant path
- **WHEN** hierarchy client get_tenant_path() is called
- **THEN** the system SHALL return ordered list from child to root
- **AND** list SHALL include the requested tenant as first element
- **AND** list SHALL end with root tenant
- **AND** list SHALL be empty if tenant does not exist

#### Scenario: Validate tenant exists
- **WHEN** hierarchy client validate_tenant_exists() is called
- **THEN** the system SHALL query account server
- **AND** return true if tenant exists
- **AND** return false if tenant does not exist
- **AND** cache result for performance (optional)

## MODIFIED Requirements

### Requirement: Tenant Hierarchy Inheritance
Settings SHALL support tenant hierarchy with inheritance rules controlled by GTS traits, using the actual hierarchy from the account server.

#### Scenario: Basic inheritance
- **WHEN** a parent tenant has a setting with is_value_inheritable=true
- **THEN** child tenants SHALL inherit the setting value
- **AND** child tenants without override SHALL use parent value
- **AND** inheritance SHALL traverse multiple levels using account server hierarchy
- **AND** system SHALL query account server for parent relationships

#### Scenario: Override inherited setting
- **WHEN** a child tenant creates a setting that exists in parent
- **THEN** child value SHALL override parent value if is_value_overwritable=true
- **AND** child SHALL use its own value
- **AND** parent value SHALL remain unchanged
- **AND** override SHALL fail if is_value_overwritable=false
- **AND** system SHALL check parent settings via hierarchy traversal

#### Scenario: Inheritance barrier
- **WHEN** a setting has is_barrier_inheritance=true
- **THEN** child tenants SHALL NOT inherit from ancestors beyond this level
- **AND** inheritance SHALL stop at the barrier
- **AND** child tenants SHALL only see settings at or below barrier level
- **AND** system SHALL respect barriers during hierarchy traversal

#### Scenario: Non-inheritable setting
- **WHEN** a setting has is_value_inheritable=false
- **THEN** child tenants SHALL NOT inherit the setting
- **AND** each tenant SHALL maintain independent values
- **AND** child tenants SHALL require explicit setting creation
- **AND** hierarchy traversal SHALL skip non-inheritable settings

#### Scenario: Multi-level inheritance
- **WHEN** settings exist at GLOBAL, root tenant, and child tenant levels
- **THEN** the system SHALL resolve in order: child > parent > root > GLOBAL > default
- **AND** most specific setting SHALL take precedence
- **AND** inheritance rules SHALL apply at each level
- **AND** system SHALL use account server to determine hierarchy levels

#### Scenario: Sibling tenant isolation
- **WHEN** sibling tenants exist under same parent
- **THEN** settings SHALL be isolated between siblings
- **AND** one sibling's settings SHALL NOT affect another sibling
- **AND** each sibling SHALL inherit independently from parent
- **AND** system SHALL use account server to identify sibling relationships

### Requirement: GTS Type Registration API
The system SHALL provide REST endpoints for registering and managing GTS type definitions with default values.

#### Scenario: Create GTS type with default value
- **WHEN** POST /cti-types is called with default_value in request body
- **THEN** the system SHALL validate the GTS identifier format
- **AND** SHALL validate default_value against schema if provided
- **AND** SHALL store the GTS type with default value
- **AND** SHALL return 201 Created with the registered type
- **AND** response SHALL include the default_value field

#### Scenario: Update GTS type default value
- **WHEN** PUT /cti-types/{type_id} is called with updated default_value
- **THEN** the system SHALL validate new default_value against schema
- **AND** SHALL update the stored default value
- **AND** SHALL return 200 OK with updated type
- **AND** existing settings SHALL NOT be affected by default value change

#### Scenario: Retrieve GTS type with default value
- **WHEN** GET /cti-types/{type_id} is called
- **THEN** the system SHALL return the GTS type definition
- **AND** response SHALL include default_value if set
- **AND** response SHALL include all traits and schema
- **AND** default_value SHALL be null if not set
