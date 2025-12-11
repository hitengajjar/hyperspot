# Settings Capability Specification

## Purpose
Provide a GTS-based settings management system per the GTS specification. Settings are identified by Global Type System identifiers (GTS) with configuration defined as GTS traits. Supports runtime registration of GTS types and dynamic trait-based configuration.

**Note**: This spec defines the GTS-based settings service implementation as a HyperSpot module using ModKit framework.

## Module Architecture

### ModKit Module Declaration
The settings_service SHALL be implemented as a HyperSpot module using ModKit with the following characteristics:

```rust
#[modkit::module(
    name = "settings_service",
    deps = ["db"],
    capabilities = [db, rest],
    client = "contract::client::SettingsApi",
    lifecycle(entry = "serve", stop_timeout = "30s", await_ready)
)]
pub struct SettingsServiceModule { ... }
```

### Module Structure (DDD-Light)
The module SHALL follow HyperSpot's DDD-light architecture:

```
modules/settings_service/
├── src/
│   ├── lib.rs              # Public exports
│   ├── module.rs           # Module declaration and ModKit traits
│   ├── config.rs           # Typed configuration
│   ├── contract/           # Public API for other modules
│   │   ├── client.rs       # Native SettingsApi trait
│   │   ├── model.rs        # Transport-agnostic models (no serde)
│   │   └── error.rs        # Domain errors
│   ├── domain/             # Business logic
│   │   ├── service.rs      # Orchestration and rules
│   │   ├── repository.rs   # Repository traits
│   │   └── events.rs       # Domain events
│   ├── infra/              # Infrastructure
│   │   └── storage/        # Database layer
│   │       ├── entity.rs   # SeaORM entities
│   │       ├── mapper.rs   # Entity ↔ Model mappers
│   │       ├── repositories.rs  # SeaORM repository implementations
│   │       ├── odata_mapper.rs  # OData filtering support
│   │       └── migrations/ # Database migrations
│   └── api/                # Transport adapters
│       └── rest/           # HTTP handlers, DTOs, routes
│           ├── dto.rs      # REST DTOs with serde
│           ├── handlers.rs # Thin HTTP handlers
│           ├── routes.rs   # Route registration
│           └── error.rs    # HTTP error mapping
└── Cargo.toml
```

### REST API Registration
The module SHALL NOT implement its own HTTP server. Instead, it SHALL register routes with the `api_ingress` module:
- Routes are registered via `RestfulModule::register_rest()` trait method
- All routes use `OperationBuilder` for OpenAPI documentation
- Handlers are thin and delegate to `domain::service::Service`
- The `api_ingress` module provides the single HTTP server for all modules

### Database Capability
The module SHALL implement `DbModule` trait:
- Declare `capabilities = [db, rest]` in module macro
- Implement `DbModule::migrate()` for schema migrations
- Use SeaORM for database operations with generic `ConnectionTrait`
- Support PostgreSQL, MySQL, and SQLite via ModKit db abstraction

### Native Client Contract
The module SHALL expose a native Rust client for inter-module communication:
- Define `contract::client::SettingsApi` trait with async methods
- NO HTTP/REST in the client trait (direct function calls)
- Register client in `ClientHub` during module initialization
- Other modules access settings via `ctx.client::<dyn SettingsApi>()?`

It allows defining setting values for combination of (tenant_id, domain_type, domain_object_id, setting_type) where 
- tenant_id is the tenant ID of the setting
- domain_type is one of the following:
  - `TENANT`: Tenant-specific settings
  - `STORAGE`: Storage-specific settings
  - `USER`: User-specific settings
  - `AGENT`: Agent-specific settings
  - `APPLICATION`: Application-specific settings
  - `BRAND`: Branding-specific settings
  - `RESOURCE`: GRPM Resource-specific settings
  - `GLOBAL`: Special type for settings not bound to any particular domain type
- domain_object_id can be either of the following:
  - `UUID`: is the unique ID of the domain object within the backend system that owns the domain_type. e.g. storage_id `a29e7adf-290d-4ac1-928f-c792da31da88` is unique storage object in Storage Manager service.
  - `generic` - literal `generic` value is a special case that means the setting is applicable to multiple domain objects at once.

- Please note that when requesting a setting value without a specific `domain_object_id`, the module prioritizes the following:
  - Settings defined for a specific `domain_object_id`
  - Settings defined for the `generic`
  - Default values
- `setting_type` refers to the GTS ID of the setting type, such as `gts.a.p.sm.setting.v1.0~vendor.app.feature.v1.0`.

Its purpose is to 
- Storage for storing settings that support arbitrary data.
- make sure that settings are unique per (tenant_id, domain_type, domain_object_id, setting_type) and to provide a way to define settings at runtime.
- auto-remove settings when **any** of the entity within the combination (tenant_id, domain_type, domain_object_id, setting_type) is deleted.
- raise events for interested parties to be aware of the changes (either setting values or setting types).


## Requirements

### Requirement: GTS Type Identification
Settings SHALL be identified exclusively by the `type` field containing a full GTS (e.g., `gts.a.p.sm.setting.v1.0~vendor.app.feature.v1.0`) per GTS specification.

#### Scenario: Setting identified by GTS
- **WHEN** a setting value is created with a valid GTS in the type field
- **THEN** the system SHALL store the type as the primary identifier component
- **AND** the setting SHALL be retrievable using GET /settings/{setting_type}
- **AND** the system SHALL parse GTS traits for configuration metadata

#### Scenario: Invalid GTS format
- **WHEN** a request uses an invalid GTS format
- **THEN** the system SHALL return 400 Bad Request
- **AND** the error SHALL indicate the expected GTS format

### Requirement: GTS Trait Configuration
Setting configuration (domain_type, events, options, operations) SHALL be defined in GTS traits, and stored in separate table within the database.

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
- **AND** child tenants SHALL inherit/override per is_value_inheritable, is_value_overwritable traits

### Requirement: Module Initialization
The module SHALL implement proper ModKit lifecycle:

#### Scenario: Module initialization
- **WHEN** the module is initialized via `Module::init()`
- **THEN** the system SHALL read typed configuration from `ModuleCtx`
- **AND** the system SHALL verify database capability is available
- **AND** the system SHALL construct SeaORM repositories with database connection
- **AND** the system SHALL build domain `Service` with repository dependencies
- **AND** the system SHALL register native client in `ClientHub`
- **AND** the system SHALL store service instance for REST handler access

#### Scenario: Database migration
- **WHEN** `DbModule::migrate()` is called
- **THEN** the system SHALL run SeaORM migrations via `Migrator::up()`
- **AND** migrations SHALL be idempotent and support rollback

### Requirement: REST API Registration
The module SHALL register HTTP routes with `api_ingress` module:

#### Scenario: Route registration
- **WHEN** `RestfulModule::register_rest()` is called
- **THEN** the system SHALL verify service is initialized
- **AND** the system SHALL call `api::rest::routes::register_routes()`
- **AND** all routes SHALL use `OperationBuilder` for OpenAPI documentation
- **AND** routes SHALL be attached to the provided `axum::Router`
- **AND** service SHALL be injected via `Extension(Arc<Service>)` layer

### Requirement: GTS API - List Settings
The system SHALL provide REST endpoints for settings management:
- GET /settings - List all settings with pagination
- GET /settings/{setting_type} - Get settings by GTS type
- PUT /settings/{setting_type} - Update or create setting value
- DELETE /settings/{setting_type} - Delete setting value
- PUT /settings/{setting_type}/lock - Lock setting for compliance

#### Scenario: OpenAPI documentation
- **WHEN** routes are registered
- **THEN** each endpoint SHALL be documented via `OperationBuilder`
- **AND** operation IDs SHALL follow pattern `settings_service.<resource>.<action>`
- **AND** request/response schemas SHALL be auto-generated from DTOs
- **AND** error responses SHALL include RFC-9457 Problem Details

### Requirement: GTS Type Management API
The system SHALL provide REST endpoints for GTS type registration:
- POST /gts-types - Register new GTS type with traits
- GET /gts-types - List all registered GTS types
- GET /gts-types/{type} - Get specific GTS type definition
- PUT /gts-types/{type} - Update GTS type definition
- DELETE /gts-types/{type} - Delete GTS type registration

#### Scenario: List all settings
- **WHEN** GET /settings is called
- **THEN** handler SHALL extract query parameters via `Query<ListParams>`
- **AND** handler SHALL call `service.list_settings(params)`
- **AND** service SHALL delegate to repository with OData filtering
- **AND** repository SHALL use `paginate_odata` for cursor-based pagination
- **AND** handler SHALL return `Result<Json<Vec<SettingDto>>, Problem>`
- **AND** response SHALL include paging metadata in headers

#### Scenario: Filter by type
- **WHEN** GET /settings?type={gts_prefix} is called
- **THEN** the system SHALL return settings matching GTS prefix

#### Scenario: Filter by domain
- **WHEN** GET /settings?domain_type=<domain_type_name>&domain_id={uuid} is called
- **THEN** the system SHALL return all settings for that domain_type_name and domain_id_uuid

### Requirement: Get all Settings of a type
The system SHALL provide GET /settings/{setting_type} endpoint per GTS spec.

#### Scenario: Get setting by GTS
- **WHEN** GET /settings/{setting_type} is called with valid GTS
- **THEN** handler SHALL extract path parameter via `Path<String>`
- **AND** handler SHALL call `service.get_settings_by_type(setting_type)`
- **AND** service SHALL validate GTS format and query repository
- **AND** repository SHALL execute SeaORM query with type filter
- **AND** handler SHALL map domain models to REST DTOs
- **AND** handler SHALL return `Result<Json<Vec<SettingDto>>, Problem>`
- **AND** response SHALL include type, tenant_id, domain_object_id, data, timestamps

#### Scenario: Setting not found
- **WHEN** GET /settings/{setting_type} is called for non-existent setting type
- **THEN** the system SHALL return 404 Not Found

### Requirement: Update Setting
The system SHALL provide PUT /settings/{setting_type} endpoint per GTS spec.

#### Scenario: Update setting value
- **WHEN** PUT /settings/{setting_type} is called with valid Setting body
- **THEN** handler SHALL extract path via `Path<String>` and body via `Json<UpdateSettingDto>`
- **AND** handler SHALL convert DTO to domain model
- **AND** handler SHALL call `service.update_setting(type, data)`
- **AND** service SHALL validate against JSON schema
- **AND** service SHALL upsert via repository
- **AND** service SHALL publish events per GTS traits
- **AND** handler SHALL return `Result<StatusCode, Problem>` with 204 No Content
- **AND** domain errors SHALL be mapped via `api::rest::error::map_domain_error()`

#### Scenario: Validation failure
- **WHEN** PUT /settings/{setting_type} is called with invalid data
- **THEN** the system SHALL return 400 Bad Request
- **AND** error SHALL describe validation failure

### Requirement: GTS API - Delete Setting
The system SHALL provide DELETE /settings/{setting_type} endpoint per GTS spec.

#### Scenario: Delete setting value
- **WHEN** DELETE /settings/{setting_type} is called
- **THEN** handler SHALL extract path parameter via `Path<String>`
- **AND** handler SHALL call `service.delete_setting(type)`
- **AND** service SHALL soft-delete via repository (set deleted_at timestamp)
- **AND** service SHALL apply retention period from GTS traits
- **AND** handler SHALL return `Result<StatusCode, Problem>` with 204 No Content

### Requirement: GTS API - Lock Setting
The system SHALL provide PUT /settings/{setting_type}/lock endpoint for compliance mode per GTS spec.

#### Scenario: Lock setting
- **WHEN** PUT /settings/{setting_type}/lock is called with read_only: true
- **THEN** the system SHALL mark setting read-only for tenant and subtree
- **AND** response SHALL be 204 No Content
- **AND** subsequent updates SHALL be rejected with 403 Forbidden

#### Scenario: Lock without compliance enabled
- **WHEN** PUT /settings/{setting_type}/lock is called for setting without enable_compliance trait
- **THEN** the system SHALL return 400 Bad Request

### Requirement: Error Handling
The module SHALL implement proper error handling and mapping:

#### Scenario: Domain errors
- **WHEN** domain operations fail
- **THEN** service SHALL return typed errors from `contract::error::SettingsError`
- **AND** error enum SHALL include variants: NotFound, Conflict, Validation, Internal
- **AND** errors SHALL be transport-agnostic (no HTTP status codes)

#### Scenario: HTTP error mapping
- **WHEN** REST handler receives domain error
- **THEN** it SHALL call `api::rest::error::map_domain_error(err, instance)`
- **AND** mapping SHALL convert to RFC-9457 Problem Details
- **AND** Problem SHALL include status, code, title, detail, instance
- **AND** handler SHALL return `Problem` which implements `IntoResponse`

### Requirement: Configuration Management
The module SHALL support typed configuration:

#### Scenario: Configuration loading
- **WHEN** module initializes
- **THEN** it SHALL read config via `ctx.module_config::<Config>()?`
- **AND** config struct SHALL derive `serde::Deserialize` with `deny_unknown_fields`
- **AND** config SHALL provide sensible defaults via `Default` trait
- **AND** config SHALL support feature flags and module-specific settings

### Requirement: Setting Schema Validation
Settings SHALL support JSON schema validation for value data.

#### Scenario: Validate against schema
- **WHEN** a setting value is created or updated
- **THEN** service SHALL retrieve schema from GTS type definition
- **AND** service SHALL validate data field against registered schema using `jsonschema` crate
- **AND** invalid data SHALL be rejected with `SettingsError::Validation`
- **AND** handler SHALL map to 400 Bad Request with Problem Details

#### Scenario: Schema versioning
- **WHEN** a setting type has multiple schema versions
- **THEN** the system SHALL validate against the registered schema version
- **AND** schema evolution SHALL be supported via GTS versioning

### Requirement: Native Client Contract
The module SHALL expose a native Rust client for inter-module communication:

#### Scenario: Client trait definition
- **WHEN** another module needs to access settings
- **THEN** it SHALL obtain client via `ctx.client::<dyn SettingsApi>()?`
- **AND** client trait SHALL define async methods mirroring domain operations
- **AND** methods SHALL accept/return `contract::model` types (no serde)
- **AND** methods SHALL return `Result<T, contract::error::SettingsError>`
- **AND** NO HTTP/REST calls SHALL be made (direct function calls)

#### Scenario: Client registration
- **WHEN** module initializes
- **THEN** it SHALL create local adapter wrapping `Arc<Service>`
- **AND** it SHALL call `expose_settings_service_client(ctx, &api)?`
- **AND** client SHALL be available to other modules via `ClientHub`

### Requirement: Repository Pattern
The module SHALL implement repository pattern with SeaORM:

#### Scenario: Repository traits
- **WHEN** domain service needs data access
- **THEN** it SHALL depend on repository traits in `domain::repository`
- **AND** traits SHALL define async methods for CRUD operations
- **AND** traits SHALL be generic over `ConnectionTrait + Send + Sync`
- **AND** methods SHALL return `anyhow::Result<T>` with context

#### Scenario: SeaORM implementation
- **WHEN** repository is instantiated
- **THEN** it SHALL be implemented in `infra::storage::repositories`
- **AND** it SHALL hold generic connection `C: ConnectionTrait`
- **AND** it SHALL use SeaORM entities for database operations
- **AND** it SHALL map entities to/from contract models via `mapper.rs`
- **AND** it SHALL use `odata_mapper.rs` for filterable list queries

### Requirement: OData Filtering Support
The module SHALL support OData-style filtering for list endpoints:

#### Scenario: Filterable DTOs
- **WHEN** defining REST DTOs for list responses
- **THEN** primary resource DTO SHALL derive `ODataFilterable`
- **AND** filterable fields SHALL be annotated with `#[odata(filter(kind = "..."))]`
- **AND** this SHALL auto-generate `<DtoName>FilterField` enum

#### Scenario: OData mapper implementation
- **WHEN** repository needs to filter results
- **THEN** `infra::storage::odata_mapper` SHALL implement `FieldToColumn` trait
- **AND** it SHALL map `FilterField` enum to SeaORM `Column` enum
- **AND** it SHALL implement `ODataFieldMapping` for cursor extraction
- **AND** repository SHALL use `paginate_odata::<FilterField, Mapper, ...>()` helper

### Requirement: GTS Type Registration API
The system SHALL provide API endpoints to register GTS type definitions with traits at runtime.

#### Scenario: Register new GTS type
- **WHEN** POST /gts-types is called with valid GTS definition
- **THEN** handler SHALL extract body via `Json<RegisterTypeDto>`
- **AND** handler SHALL call `service.register_gts_type(definition)`
- **AND** service SHALL validate GTS format and traits
- **AND** service SHALL store via repository in gts_types table
- **AND** handler SHALL return `Result<(StatusCode, Json<TypeDto>), Problem>` with 201 Created
- **AND** subsequent setting operations SHALL use the registered traits

#### Scenario: Register duplicate GTS type
- **WHEN** POST /gts-types is called with existing type
- **THEN** the system SHALL return 409 Conflict
- **AND** existing registration SHALL remain unchanged

#### Scenario: Invalid GTS format
- **WHEN** POST /gts-types is called with invalid GTS format
- **THEN** the system SHALL return 400 Bad Request
- **AND** error SHALL describe validation failure

### Requirement: GTS Type Retrieval API
The system SHALL provide API endpoints to retrieve registered GTS type definitions.

#### Scenario: Get GTS type by identifier
- **WHEN** GET /gts-types/{type} is called
- **THEN** handler SHALL extract path via `Path<String>`
- **AND** handler SHALL call `service.get_gts_type(type)`
- **AND** service SHALL query repository for type definition
- **AND** handler SHALL map to DTO and return `Result<Json<TypeDto>, Problem>`
- **AND** response SHALL include GTS definition with traits and schema

#### Scenario: List all registered GTS types
- **WHEN** GET /gts-types is called
- **THEN** the system SHALL return array of registered GTS types
- **AND** response SHALL include paging metadata

#### Scenario: GTS type not found
- **WHEN** GET /gts-types/{type} is called for unregistered type
- **THEN** the system SHALL return 404 Not Found

### Requirement: GTS Type Update API
The system SHALL provide API endpoints to update registered GTS type definitions.

#### Scenario: Update GTS type traits
- **WHEN** PUT /gts-types/{type} is called with updated definition
- **THEN** handler SHALL extract path and body via `Path<String>` and `Json<UpdateTypeDto>`
- **AND** handler SHALL call `service.update_gts_type(type, definition)`
- **AND** service SHALL validate and update via repository
- **AND** handler SHALL return `Result<StatusCode, Problem>` with 200 OK
- **AND** existing settings SHALL use updated traits for future operations

#### Scenario: Update non-existent GTS type
- **WHEN** PUT /gts-types/{type} is called for unregistered type
- **THEN** the system SHALL return 404 Not Found

### Requirement: GTS Type Deletion API
The system SHALL provide API endpoints to delete registered GTS type definitions.

#### Scenario: Delete GTS type
- **WHEN** DELETE /gts-types/{type} is called
- **THEN** handler SHALL extract path via `Path<String>`
- **AND** handler SHALL call `service.delete_gts_type(type)`
- **AND** service SHALL soft-delete via repository
- **AND** handler SHALL return `Result<StatusCode, Problem>` with 204 No Content
- **AND** existing setting values SHALL remain but new values SHALL be rejected

#### Scenario: Delete GTS type with active settings
- **WHEN** DELETE /gts-types/{type} is called for type with active settings
- **THEN** the system SHALL return 409 Conflict
- **AND** error SHALL indicate active settings exist

### Requirement: Domain Object ID Formats
Settings SHALL support multiple domain_object_id formats per OpenSpec: UUID, GTS, AppCode, and "generic".

#### Scenario: UUID domain_object_id
- **WHEN** a setting is created with UUID domain_object_id (e.g., storage system UUID)
- **THEN** the system SHALL store and retrieve the setting using the UUID
- **AND** the setting SHALL be isolated from other domain objects
- **AND** UUID format SHALL be validated as valid UUID v4

#### Scenario: GTS domain_object_id
- **WHEN** a setting is created with GTS domain_object_id (e.g., agent type identifier)
- **THEN** the system SHALL store and retrieve the setting using the GTS identifier
- **AND** the GTS format SHALL be validated (starts with "gts." and contains "~")
- **AND** the setting SHALL be scoped to that GTS type instance

#### Scenario: AppCode domain_object_id
- **WHEN** a setting is created with AppCode domain_object_id (e.g., "APP-BACKUP-2024")
- **THEN** the system SHALL store and retrieve the setting using the AppCode
- **AND** the setting SHALL be scoped to that application
- **AND** AppCode format SHALL accept alphanumeric with hyphens

#### Scenario: Generic domain_object_id
- **WHEN** a setting is created with "generic" as domain_object_id
- **THEN** the system SHALL treat it as a tenant-level or type-level default
- **AND** the setting SHALL apply to multiple domain objects
- **AND** specific domain_object_id settings SHALL take precedence over generic

#### Scenario: All formats coexist
- **WHEN** settings with all 4 domain_object_id formats exist for same tenant and type
- **THEN** each setting SHALL be independently stored and retrievable
- **AND** settings SHALL not interfere with each other
- **AND** queries SHALL return correct setting based on domain_object_id

### Requirement: Tenant Hierarchy Inheritance
Settings SHALL support tenant hierarchy with inheritance rules controlled by GTS traits.

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
- **AND** override SHALL fail if is_value_overwritable=false

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

### Requirement: Repository Query Methods
The system SHALL provide repository methods for querying settings by various criteria.

#### Scenario: Find by domain object
- **WHEN** repository.find_by_domain_object(domain_object_id) is called
- **THEN** the system SHALL return all settings for that domain object
- **AND** results SHALL include all GTS types for that domain object
- **AND** results SHALL span multiple tenants if applicable
- **AND** soft-deleted settings SHALL be excluded

#### Scenario: Find by tenant
- **WHEN** repository.find_by_tenant(tenant_id) is called
- **THEN** the system SHALL return all settings for that tenant
- **AND** results SHALL include all GTS types and domain objects
- **AND** results SHALL be scoped to single tenant only
- **AND** soft-deleted settings SHALL be excluded

#### Scenario: Find by type with tenant filter
- **WHEN** repository.find_by_type(type, Some(tenant_id)) is called
- **THEN** the system SHALL return settings matching both type and tenant
- **AND** results SHALL be filtered by tenant_id
- **AND** soft-deleted settings SHALL be excluded

#### Scenario: Find by type without tenant filter
- **WHEN** repository.find_by_type(type, None) is called
- **THEN** the system SHALL return all settings of that type across all tenants
- **AND** results SHALL include all tenants
- **AND** soft-deleted settings SHALL be excluded

### Requirement: Soft and Hard Delete
The system SHALL support both soft delete (with retention) and hard delete (permanent removal).

#### Scenario: Soft delete with retention
- **WHEN** a setting is soft-deleted
- **THEN** the system SHALL set deleted_at timestamp
- **AND** the setting SHALL be excluded from normal queries
- **AND** the setting SHALL remain in database for retention period
- **AND** retention period SHALL be read from GTS traits

#### Scenario: Hard delete
- **WHEN** repository.hard_delete() is called
- **THEN** the system SHALL permanently remove the setting from database
- **AND** the setting SHALL NOT be recoverable
- **AND** the operation SHALL be irreversible

#### Scenario: Retention period enforcement
- **WHEN** retention period expires for soft-deleted setting
- **THEN** the system SHALL automatically hard-delete the setting
- **AND** cleanup SHALL run periodically
- **AND** expired settings SHALL be permanently removed

### Requirement: Pagination Support
The system SHALL support pagination for list operations.

#### Scenario: List all with pagination
- **WHEN** repository.list_all(limit, offset) is called
- **THEN** the system SHALL return settings limited by limit parameter
- **AND** results SHALL start from offset position
- **AND** pagination SHALL support cursor-based navigation
- **AND** total count SHALL be available in response metadata

#### Scenario: OData cursor pagination
- **WHEN** list endpoint uses OData filtering
- **THEN** the system SHALL use paginate_odata helper
- **AND** cursor SHALL be based on filterable fields
- **AND** pagination metadata SHALL be in response headers
- **AND** next/previous links SHALL be provided

### Requirement: GTS Format Validation
The system SHALL validate GTS format according to specification.

#### Scenario: Valid GTS format
- **WHEN** GTS type is registered with format "gts.a.p.sm.setting.v1.0~vendor.app.feature.v1.0"
- **THEN** validation SHALL pass
- **AND** GTS SHALL be stored as-is
- **AND** GTS SHALL be usable for setting operations

#### Scenario: Missing gts prefix
- **WHEN** GTS type is registered without "gts." prefix
- **THEN** validation SHALL fail with InvalidGtsFormat error
- **AND** error SHALL indicate "GTS must start with 'gts.'"
- **AND** registration SHALL be rejected

#### Scenario: Missing tilde separator
- **WHEN** GTS type is registered without "~" separator
- **THEN** validation SHALL fail with InvalidGtsFormat error
- **AND** error SHALL indicate "GTS must contain '~' separator"
- **AND** registration SHALL be rejected

#### Scenario: Comprehensive validation
- **WHEN** comprehensive GTS validation is implemented (Phase 2)
- **THEN** the system SHALL validate version format
- **AND** the system SHALL validate namespace structure
- **AND** the system SHALL validate component identifiers
- **AND** invalid formats SHALL be rejected with detailed error messages
