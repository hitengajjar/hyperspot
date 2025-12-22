# Settings Service - Inheritance and Validation Enhancements

## ADDED Requirements

### Requirement: Setting Inheritance Resolution
The system SHALL provide automatic inheritance resolution for settings across tenant hierarchies.

#### Scenario: Resolve inherited setting
- **WHEN** GET `/settings/{type}/inherited` is called with tenant_id and domain_object_id
- **THEN** the system SHALL walk up the tenant hierarchy to find the setting
- **AND** the system SHALL respect `is_value_inheritable` flag from GTS traits
- **AND** the system SHALL stop at inheritance barriers (`is_barrier_inheritance = true`)
- **AND** the system SHALL return the nearest ancestor's setting value
- **AND** the response SHALL include `inherited_from` metadata indicating source tenant

#### Scenario: No inherited value found
- **WHEN** inheritance resolution finds no setting in the hierarchy
- **THEN** the system SHALL return 404 Not Found
- **AND** the error SHALL indicate no inherited value is available

#### Scenario: Inheritance with query parameter
- **WHEN** GET `/settings/{type}?resolve_inheritance=true` is called
- **THEN** the system SHALL automatically resolve inherited values for missing settings
- **AND** the response SHALL include inheritance metadata for each setting

### Requirement: Overwritability Enforcement
The system SHALL enforce `is_value_overwritable` flag when child tenants attempt to override parent settings.

#### Scenario: Block non-overwritable setting
- **WHEN** PUT `/settings/{type}` is called for a child tenant
- **AND** a parent tenant has a setting with `is_value_overwritable = false`
- **THEN** the system SHALL return 403 Forbidden
- **AND** the error SHALL indicate the setting cannot be overridden
- **AND** the error SHALL include the parent tenant ID that owns the non-overwritable setting

#### Scenario: Allow overwritable setting
- **WHEN** PUT `/settings/{type}` is called for a child tenant
- **AND** a parent tenant has a setting with `is_value_overwritable = true`
- **THEN** the system SHALL allow the child to create their own setting value
- **AND** the child's setting SHALL take precedence over the parent's

#### Scenario: No parent setting exists
- **WHEN** PUT `/settings/{type}` is called for a child tenant
- **AND** no parent tenant has the setting
- **THEN** the system SHALL allow the setting to be created
- **AND** overwritability checks SHALL be skipped

### Requirement: Domain Object ID Validation
The system SHALL validate domain_object_id format according to supported types.

#### Scenario: Valid UUID format
- **WHEN** a setting is created with domain_object_id as a valid UUID
- **THEN** the system SHALL accept the value
- **AND** the setting SHALL be stored successfully

#### Scenario: Valid GTS format
- **WHEN** a setting is created with domain_object_id as a valid GTS identifier (e.g., "gts.a.p.sm.resource.v1.0~...")
- **THEN** the system SHALL validate using GTS format rules
- **AND** the setting SHALL be stored successfully

#### Scenario: Valid AppCode format
- **WHEN** a setting is created with domain_object_id matching AppCode pattern `[a-z0-9-]+`
- **THEN** the system SHALL accept the value
- **AND** the setting SHALL be stored successfully

#### Scenario: Generic domain object
- **WHEN** a setting is created with domain_object_id = "generic"
- **THEN** the system SHALL accept the special value
- **AND** the setting SHALL apply to all domain objects of that type

#### Scenario: Invalid domain object ID format
- **WHEN** a setting is created with an invalid domain_object_id format
- **THEN** the system SHALL return 400 Bad Request
- **AND** the error SHALL indicate the expected formats (UUID, GTS, AppCode, or "generic")

### Requirement: Tenant Hierarchy Integration
The system SHALL integrate with the tenant service to obtain hierarchy information.

#### Scenario: Query tenant hierarchy
- **WHEN** inheritance resolution is needed
- **THEN** the system SHALL obtain a tenant hierarchy client via ClientHub
- **AND** the system SHALL query for parent tenant ID
- **AND** the system SHALL cache hierarchy data for performance (optional)

#### Scenario: Tenant service unavailable
- **WHEN** the tenant service is unavailable during inheritance resolution
- **THEN** the system SHALL return 503 Service Unavailable
- **AND** the error SHALL indicate the tenant service is required for inheritance

### Requirement: Inheritance Metadata in Responses
The system SHALL include inheritance metadata in API responses when settings are inherited.

#### Scenario: Include inheritance source
- **WHEN** a setting is resolved via inheritance
- **THEN** the response SHALL include `inherited_from` field with source tenant ID
- **AND** the response SHALL include `is_inherited` boolean flag
- **AND** the response SHALL include `inheritance_depth` indicating levels traversed

#### Scenario: Direct setting (not inherited)
- **WHEN** a setting belongs directly to the requested tenant
- **THEN** the response SHALL set `is_inherited = false`
- **AND** the response SHALL set `inherited_from = null`
- **AND** the response SHALL set `inheritance_depth = 0`

### Requirement: Python E2E Tests for Inheritance
The system SHALL provide comprehensive E2E tests for inheritance scenarios.

#### Scenario: Basic inheritance test
- **WHEN** E2E test creates a 3-level hierarchy (root → child → grandchild)
- **AND** a setting is defined at root level
- **THEN** the test SHALL verify child inherits from root
- **AND** the test SHALL verify grandchild inherits from root

#### Scenario: Override inheritance test
- **WHEN** E2E test creates a hierarchy with override at child level
- **THEN** the test SHALL verify child has its own value
- **AND** the test SHALL verify grandchild inherits from child (not root)

#### Scenario: Inheritance barrier test
- **WHEN** E2E test creates a setting with `is_barrier_inheritance = true`
- **THEN** the test SHALL verify child does NOT inherit
- **AND** the test SHALL verify grandchild does NOT inherit

#### Scenario: Sibling isolation test
- **WHEN** E2E test creates siblings under same parent
- **AND** one sibling has a setting
- **THEN** the test SHALL verify other sibling cannot see the setting

### Requirement: Python E2E Tests for Validation
The system SHALL provide E2E tests for domain_object_id validation.

#### Scenario: UUID validation test
- **WHEN** E2E test creates setting with valid UUID
- **THEN** the test SHALL verify 201 Created response
- **AND** the test SHALL verify setting is retrievable

#### Scenario: GTS validation test
- **WHEN** E2E test creates setting with valid GTS identifier
- **THEN** the test SHALL verify 201 Created response

#### Scenario: AppCode validation test
- **WHEN** E2E test creates setting with valid AppCode
- **THEN** the test SHALL verify 201 Created response

#### Scenario: Invalid format test
- **WHEN** E2E test attempts to create setting with invalid domain_object_id
- **THEN** the test SHALL verify 400 Bad Request response
- **AND** the test SHALL verify error message describes valid formats

## MODIFIED Requirements

### Requirement: GTS Trait Configuration
Setting configuration (domain_type, events, options, operations) SHALL be defined in GTS traits, and stored in separate table within the database. The system SHALL enforce inheritance and overwritability rules based on these traits.

#### Scenario: Configuration from traits
- **WHEN** a setting type is registered with GTS traits
- **THEN** the system SHALL parse domain_type, events, options, operations from traits
- **AND** configuration SHALL apply to all instances of that type
- **AND** database SHALL store instances in a table with schema: type, tenant_id, domain_object_id, data, timestamps
- **AND** database SHALL store the declared type and its traits in another table with schema: type, user_id, domain_type, events, options, operations, where user_id is user's ID who created the type

#### Scenario: Event generation from traits
- **WHEN** a setting value is modified
- **THEN** the system SHALL read event config (audit, notification) from GTS traits
- **AND** events SHALL be published per trait configuration (SELF, SUBROOT, NONE) where SUBROOT means one event for each tenant in the subtree, SELF means event for tenant where change happened, NONE means no event
- **AND** audit event will be published to the eventing topic `sm.events.audit`
- **AND** notification event will be published to the eventing topic `sm.events.notification`

#### Scenario: Inheritance from traits
- **WHEN** a setting value is queried
- **THEN** the system SHALL apply inheritance rules from GTS options traits
- **AND** child tenants SHALL inherit per `is_value_inheritable` flag
- **AND** child tenants SHALL be blocked from overriding per `is_value_overwritable` flag
- **AND** inheritance SHALL be blocked by `is_barrier_inheritance` flag
- **AND** the system SHALL walk up the tenant hierarchy to resolve inherited values

#### Scenario: Overwritability enforcement
- **WHEN** a child tenant attempts to create a setting
- **AND** a parent has the same setting with `is_value_overwritable = false`
- **THEN** the system SHALL reject the request with 403 Forbidden
- **AND** the error SHALL indicate which parent tenant blocks the override
