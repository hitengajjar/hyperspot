# Settings Service - Root/Admin Override Capability Delta

## ADDED Requirements

### Requirement: Root/Admin Override for Non-Overwritable Settings
The system SHALL allow root/admin users to override settings regardless of `is_value_overwritable=false` constraint to support emergency operations, compliance updates, and system-wide policy changes.

#### Scenario: Root admin overrides non-overwritable setting
- **WHEN** a root/admin user attempts to upsert a setting at a child tenant
- **AND** the parent tenant has the same setting with `is_value_overwritable=false`
- **THEN** the system SHALL allow the override operation to succeed
- **AND** the child tenant SHALL have the new setting value
- **AND** the parent tenant's setting SHALL remain unchanged

#### Scenario: Root admin sets setting at any tenant level
- **WHEN** a root/admin user creates or updates a setting at any tenant in the hierarchy
- **THEN** the system SHALL allow the operation regardless of inheritance constraints
- **AND** the setting SHALL be stored at the specified tenant level
- **AND** normal inheritance rules SHALL apply for non-admin users querying the setting

#### Scenario: Non-admin user still blocked by non-overwritable constraint
- **WHEN** a non-admin user attempts to override a parent setting with `is_value_overwritable=false`
- **THEN** the system SHALL reject the operation with 403 Forbidden or Conflict error
- **AND** the error SHALL indicate the setting is not overwritable
- **AND** existing behavior SHALL be preserved for non-admin users

#### Scenario: Root admin privilege detection
- **WHEN** a request is processed by the settings service
- **THEN** the system SHALL extract root/admin privilege status from authentication context
- **AND** the system SHALL check for `X-Root-Admin` header or equivalent context flag
- **AND** the privilege status SHALL be propagated through domain service operations

### Requirement: Audit Trail for Root/Admin Overrides
The system SHALL log all root/admin override operations with comprehensive audit information for security and compliance tracking.

#### Scenario: Audit log for override operation
- **WHEN** a root/admin user successfully overrides a non-overwritable setting
- **THEN** the system SHALL emit an audit event with the following information:
  - User or client identifier performing the override
  - Tenant ID where the setting was modified
  - Setting type (GTS identifier)
  - Domain object ID
  - Timestamp of the override operation
  - Indication that this was a root/admin override
- **AND** the audit event SHALL be published to the audit event topic

#### Scenario: Audit event includes override context
- **WHEN** an audit event is generated for a root/admin override
- **THEN** the event SHALL include a flag or field indicating root/admin privilege was used
- **AND** the event SHALL be distinguishable from normal setting updates
- **AND** downstream audit systems SHALL be able to filter for root/admin overrides

## MODIFIED Requirements

### Requirement: Tenant Hierarchy Inheritance
Settings SHALL support tenant hierarchy with inheritance rules controlled by GTS traits, with exceptions for root/admin users who can bypass `is_value_overwritable=false` constraints.

#### Scenario: Basic inheritance
- **WHEN** a parent tenant has a setting with is_value_inheritable=true
- **THEN** child tenants SHALL inherit the setting value
- **AND** child tenants without override SHALL use parent value
- **AND** inheritance SHALL traverse multiple levels

#### Scenario: Override inherited setting
- **WHEN** a child tenant creates a setting that exists in parent
- **THEN** child value SHALL override parent value if is_value_overwritable=true
- **AND** child SHALL use its own value
- **AND** parent value SHALL remain unchanged
- **AND** override SHALL fail if is_value_overwritable=false UNLESS the user has root/admin privileges

#### Scenario: Root admin bypasses overwritable constraint
- **WHEN** a root/admin user attempts to override a parent setting with is_value_overwritable=false
- **THEN** the system SHALL allow the override operation
- **AND** the override SHALL be logged in the audit trail
- **AND** normal users SHALL still be blocked by the constraint

#### Scenario: Inheritance barrier
- **WHEN** a setting has is_barrier_inheritance=true
- **THEN** child tenants SHALL NOT inherit from ancestors beyond this level
- **AND** inheritance SHALL stop at the barrier
- **AND** child tenants SHALL only see settings at or below barrier level

#### Scenario: Non-inheritable setting
- **WHEN** a setting has is_value_inheritable=false
- **THEN** child tenants SHALL NOT inherit the setting
- **AND** each tenant SHALL maintain independent values
- **AND** child tenants SHALL require explicit setting creation

#### Scenario: Multi-level inheritance
- **WHEN** settings exist at GLOBAL, root tenant, and child tenant levels
- **THEN** the system SHALL resolve in order: child > parent > root > GLOBAL
- **AND** most specific setting SHALL take precedence
- **AND** inheritance rules SHALL apply at each level

#### Scenario: Sibling tenant isolation
- **WHEN** sibling tenants exist under same parent
- **THEN** settings SHALL be isolated between siblings
- **AND** one sibling's settings SHALL NOT affect another sibling
- **AND** each sibling SHALL inherit independently from parent

### Requirement: Root/Admin Lock Override
Root/admin users SHALL be able to lock and unlock settings regardless of current lock state, and SHALL be able to modify locked settings.

#### Scenario: Root/admin modifies locked setting
- **GIVEN** a setting is locked for compliance
- **WHEN** a root/admin user attempts to modify the setting
- **THEN** the modification SHALL succeed
- **AND** the setting SHALL remain locked
- **AND** the operation SHALL be audit logged

#### Scenario: Root/admin locks setting
- **GIVEN** a setting exists
- **WHEN** a root/admin user locks the setting
- **THEN** the lock SHALL be applied
- **AND** the operation SHALL be audit logged with user identifier

#### Scenario: Non-admin blocked by lock
- **GIVEN** a setting is locked
- **WHEN** a non-admin user attempts to modify the setting
- **THEN** the operation SHALL fail with Conflict error
- **AND** the setting SHALL remain unchanged
