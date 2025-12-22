# Settings Capability Specification

## ADDED Requirements

### Requirement: CTI Type Identification
Settings SHALL be identified exclusively by the `type` field containing a full CTI (e.g., `cti.a.p.sm.setting.v1.0~vendor.app.feature.v1.0`) per sm-CTI specification.

#### Scenario: Setting identified by CTI
- **WHEN** a setting value is created with a valid CTI in the type field
- **THEN** the system SHALL store the type as the primary identifier component
- **AND** the setting SHALL be retrievable using GET /settings/{setting_type}
- **AND** the system SHALL parse CTI traits for configuration metadata

#### Scenario: Invalid CTI format
- **WHEN** a request uses an invalid CTI format
- **THEN** the system SHALL return 400 Bad Request
- **AND** the error SHALL indicate the expected CTI format

### Requirement: CTI Trait Configuration
Setting configuration (domain_type, events, options, operations) SHALL be defined in CTI traits, not stored in database.

#### Scenario: Configuration from traits
- **WHEN** a setting type is registered with CTI traits
- **THEN** the system SHALL parse domain_type, events, options, operations from traits
- **AND** configuration SHALL apply to all instances of that type
- **AND** database SHALL store only: type, tenant_id, domain_object_id, data, timestamps

#### Scenario: Event generation from traits
- **WHEN** a setting value is modified
- **THEN** the system SHALL read event config (audit, notification) from CTI traits
- **AND** events SHALL be published per trait configuration (SELF, SUBROOT, NONE)

#### Scenario: Inheritance from traits
- **WHEN** a setting value is queried
- **THEN** the system SHALL apply inheritance rules from CTI options traits
- **AND** child tenants SHALL inherit/override per is_value_inheritable, is_value_overwritable traits

### Requirement: sm-CTI API - List Settings
The system SHALL provide GET /settings endpoint to list all settings with filtering per sm-CTI spec.

#### Scenario: List all settings
- **WHEN** GET /settings is called
- **THEN** the system SHALL return array of Setting objects
- **AND** response SHALL include paging metadata

#### Scenario: Filter by type
- **WHEN** GET /settings?type={cti_prefix} is called
- **THEN** the system SHALL return settings matching CTI prefix

#### Scenario: Filter by domain
- **WHEN** GET /settings?domain_type=TENANT&tenant_id={uuid} is called
- **THEN** the system SHALL return settings for that tenant

### Requirement: sm-CTI API - Get Setting
The system SHALL provide GET /settings/{setting_type} endpoint per sm-CTI spec.

#### Scenario: Get setting by CTI
- **WHEN** GET /settings/{setting_type} is called with valid CTI
- **THEN** the system SHALL return Setting object or array
- **AND** response SHALL include type, tenant_id, domain_object_id, data, timestamps

#### Scenario: Setting not found
- **WHEN** GET /settings/{setting_type} is called for non-existent setting
- **THEN** the system SHALL return 404 Not Found

### Requirement: sm-CTI API - Update Setting
The system SHALL provide PUT /settings/{setting_type} endpoint per sm-CTI spec.

#### Scenario: Update setting value
- **WHEN** PUT /settings/{setting_type} is called with valid Setting body
- **THEN** the system SHALL update or create the setting value
- **AND** response SHALL be 204 No Content
- **AND** events SHALL be generated per CTI traits

#### Scenario: Validation failure
- **WHEN** PUT /settings/{setting_type} is called with invalid data
- **THEN** the system SHALL return 400 Bad Request
- **AND** error SHALL describe validation failure

### Requirement: sm-CTI API - Delete Setting
The system SHALL provide DELETE /settings/{setting_type} endpoint per sm-CTI spec.

#### Scenario: Delete setting value
- **WHEN** DELETE /settings/{setting_type} is called
- **THEN** the system SHALL soft-delete the setting value
- **AND** response SHALL be 204 No Content
- **AND** retention period from CTI traits SHALL apply

### Requirement: sm-CTI API - Lock Setting
The system SHALL provide PUT /settings/{setting_type}/lock endpoint for compliance mode per sm-CTI spec.

#### Scenario: Lock setting
- **WHEN** PUT /settings/{setting_type}/lock is called with read_only: true
- **THEN** the system SHALL mark setting read-only for tenant and subtree
- **AND** response SHALL be 204 No Content
- **AND** subsequent updates SHALL be rejected with 403 Forbidden

#### Scenario: Lock without compliance enabled
- **WHEN** PUT /settings/{setting_type}/lock is called for setting without enable_compliance trait
- **THEN** the system SHALL return 400 Bad Request

### Requirement: Setting Schema Validation
Settings SHALL support JSON schema validation for value data.

#### Scenario: Validate against schema
- **WHEN** a setting value is created or updated
- **THEN** the system SHALL validate data field against registered schema
- **AND** invalid data SHALL be rejected with 400 Bad Request

#### Scenario: Schema versioning
- **WHEN** a setting type has multiple schema versions
- **THEN** the system SHALL validate against the registered schema version
- **AND** schema evolution SHALL be supported via CTI versioning

### Requirement: CTI Type Registration API
The system SHALL provide API endpoints to register CTI type definitions with traits at runtime.

#### Scenario: Register new CTI type
- **WHEN** POST /cti-types is called with valid CTI definition
- **THEN** the system SHALL store the type, traits, and schema in registry
- **AND** response SHALL be 201 Created with registered type details
- **AND** subsequent setting operations SHALL use the registered traits

#### Scenario: Register duplicate CTI type
- **WHEN** POST /cti-types is called with existing type
- **THEN** the system SHALL return 409 Conflict
- **AND** existing registration SHALL remain unchanged

#### Scenario: Invalid CTI format
- **WHEN** POST /cti-types is called with invalid CTI format
- **THEN** the system SHALL return 400 Bad Request
- **AND** error SHALL describe validation failure

### Requirement: CTI Type Retrieval API
The system SHALL provide API endpoints to retrieve registered CTI type definitions.

#### Scenario: Get CTI type by identifier
- **WHEN** GET /cti-types/{type} is called
- **THEN** the system SHALL return the CTI definition with traits and schema
- **AND** response SHALL be 200 OK

#### Scenario: List all registered CTI types
- **WHEN** GET /cti-types is called
- **THEN** the system SHALL return array of registered CTI types
- **AND** response SHALL include paging metadata

#### Scenario: CTI type not found
- **WHEN** GET /cti-types/{type} is called for unregistered type
- **THEN** the system SHALL return 404 Not Found

### Requirement: CTI Type Update API
The system SHALL provide API endpoints to update registered CTI type definitions.

#### Scenario: Update CTI type traits
- **WHEN** PUT /cti-types/{type} is called with updated definition
- **THEN** the system SHALL update the traits and schema
- **AND** response SHALL be 200 OK
- **AND** existing settings SHALL use updated traits for future operations

#### Scenario: Update non-existent CTI type
- **WHEN** PUT /cti-types/{type} is called for unregistered type
- **THEN** the system SHALL return 404 Not Found

### Requirement: CTI Type Deletion API
The system SHALL provide API endpoints to delete registered CTI type definitions.

#### Scenario: Delete CTI type
- **WHEN** DELETE /cti-types/{type} is called
- **THEN** the system SHALL soft-delete the CTI type registration
- **AND** response SHALL be 204 No Content
- **AND** existing setting values SHALL remain but new values SHALL be rejected

#### Scenario: Delete CTI type with active settings
- **WHEN** DELETE /cti-types/{type} is called for type with active settings
- **THEN** the system SHALL return 409 Conflict
- **AND** error SHALL indicate active settings exist

## REMOVED Requirements

### Requirement: Namespace Entity
**Reason**: Namespace entity not in sm-CTI specification. Settings identified directly by CTI type per spec. Greenfield implementation for new Rust service.

### Requirement: Namespace-Based APIs
**Reason**: sm-CTI spec defines settings APIs without namespace concept. APIs use CTI type directly in URI path.

### Requirement: Legacy Name Support
**Reason**: Clean implementation for new Rust service. CTI-only identification per sm-CTI spec. No backward compatibility needed.

### Requirement: Namespace API Endpoints
**Reason**: With namespace entity removed, namespace-specific CRUD operations are no longer needed. New service implements only sm-CTI endpoints.
