# Change: Initial Settings Service Implementation

## Why
Implement a settings management service as a HyperSpot module using ModKit. Settings are identified by CTI (Component Type Identifier) with configuration metadata defined as CTI traits, providing a clean, type-safe configuration management system.

## What Changes
- Create HyperSpot module `settings_service` with ModKit capabilities (db, rest)
- Implement DDD-light architecture (contract/, domain/, infra/, api/ layers)
- Create `cti_registry` table to store CTI type definitions with traits
- Create `settings` table: `type` (CTI), `tenant_id`, `domain_object_id`, `data`, timestamps
- Implement Setting and CTIType domain models
- Implement repository pattern with SeaORM
- Implement REST API endpoints via api_ingress:
  - GET/PUT/DELETE `/settings/{setting_type}`
  - PUT `/settings/{setting_type}/lock`
  - POST/GET/PUT/DELETE `/cti-types`
- Implement native client contract for inter-module communication
- Support JSON Schema validation for setting values
- Support OData filtering for list endpoints
- Implement event publishing for configuration changes

## Impact
- Affected specs: `settings` capability
- New HyperSpot module implementation:
  - Module: ModKit module with db and rest capabilities
  - Database: Create cti_registry and settings tables with SeaORM migrations
  - Domain: Implement Setting and CTIType entities with business logic
  - Contract: Native SettingsApi trait for inter-module communication
  - Repository: SeaORM-based repositories with OData filtering
  - API: REST endpoints registered with api_ingress using OperationBuilder
  - Events: Event generation from CTI traits
  - Configuration: Typed configuration with sensible defaults
