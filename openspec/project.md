# Project Context

## Purpose
We want to generate a Rust equivalent of the current Go-based settings service. The Rust implementation SHALL expose the v2 HTTP behavior while presenting itself externally as the v1 API.

If the instruction sounds unclear, vague or requires more context. Ask for clarification.

Always open `@/docs/ARCHITECTURE_MANIFEST.md` to have context about how is the project.

Always open `@/docs/TRACING_SETUP.md` to learn about the project's tracing setup.

Always open `@/docs/MODKIT_UNIFIED_SYSTEM.md` to learn about the project's `@/lib/modkit*` unified system.

Always open `@/docs/MODULE_CREATION_PROMPT.md` to learn about how to create a new module.

Always open `@/docs/SECURE-ORM.md` to learn about the project's database interactions with ORM.

Always follow the guidelines specified in `@/guidelines/README.md`.


## Tech Stack
- RUST
- REACT.js


## Project Conventions


### Architecture Patterns
The Settings Service is a stateless HTTP microservice exposing a generic REST API for managing configuration key-value pairs across the platform. Settings are identified by CTI (Component Type Identifier) and can be bound to specific domain objects like tenants, storage systems, agents, users, applications, brands, and resources. Each setting type includes JSON Schema validation rules, default values, and retention policies. The service supports both static registration (defined in configuration files) and dynamic runtime registration via API. Settings follow tenant hierarchy inheritance rules, allowing parent tenant configurations to cascade to child tenants with optional override capabilities. The service emits Audit and Notification events for all configuration changes and provides reporting interfaces for analytics. Authorization is enforced through integration with the Account Server for RBAC scopes and origin tracking.

### Testing Strategy
- **Unit Tests**: Use Rust's `#[cfg(test)]` modules for testing domain logic, repositories, and business rules. Aim for equivalent coverage to the Go implementation.
- **Integration Tests**: Create integration tests for API endpoints, database operations, and event publishing. Use `sqlx` test fixtures for database state setup.
- **API Regression Tests**: Maintain the existing `tests/apitest` pytest suite. Point it to the Rust endpoint to ensure API parity with the Go service.
- **Performance Tests**: Validate that read endpoints handle at least 100 requests per second with sub-second response times.


### Rust Technology Stack
The following Rust libraries and frameworks are recommended for the implementation:

| Concern | Rust Tooling | Notes |
|---------|--------------|-------|
| HTTP server/router | `axum` or `actix-web` | Middleware-friendly with tower layers for authentication and logging |
| Auth middleware | Custom tower layers + `jsonwebtoken` | Token parsing and scope evaluation |
| Configuration | `serde_yaml`, `config` crate | Layered config files with environment variable overrides |
| Database | `sqlx` or `sea-orm` | Async SQL with migration support and transaction handling |
| Background workers | `tokio` tasks | Structured task supervision for workers |
| Metrics | `metrics` crate + Prometheus exporter | HTTP duration histograms and request counters |
| JSON Schema validation | `jsonschema` crate | Validate setting values against JSON Schema v7 |

### Migration Strategy
The Rust implementation follows a phased approach to ensure API parity and minimize risk:

1. **Phase 1 - Core APIs**: Implement CTI-based settings APIs (GET/PUT/DELETE `/settings/{setting_type}`) and CTI type registration APIs (POST/GET/PUT/DELETE `/cti-types`). Focus on database layer, domain logic, and basic event publishing.

2. **Phase 2 - Feature Parity**: Add tenant hierarchy inheritance, JSON Schema validation, compliance mode locking, and full authorization integration with Account Server.

3. **Phase 3 - Background Workers**: Port cyber cache sync, setting watcher, and tenant reconciliation jobs from Go to Rust using `tokio` tasks.

4. **Phase 4 - Production Deployment**: Run parallel deployments with traffic splitting. Monitor telemetry, validate API regression tests, and gradually shift traffic from Go to Rust service.

5. **Phase 5 - Decommission**: Once telemetry confirms stability and feature parity, retire Go binaries and update deployment manifests (Helm charts, make targets) to use Rust builds exclusively.

## Domain Context
- **Problem**: Internal teams and external partners need a centralized configuration management system where settings can be defined once and applied across multiple organizational levels (tenants, sub-tenants) with the ability to inherit parent configurations or override them at lower levels. The system must track who changed what and when for compliance and debugging purposes.

- **Domain Types**: Settings can be scoped to different types of platform entities:
  - **Tenant**: Organization-level configurations (e.g., backup retention policies, feature flags)
  - **Storage**: Storage system-specific settings (e.g., encryption preferences, quota limits)
  - **User**: Per-user preferences and permissions
  - **Agent**: Backup agent configurations and policies
  - **Application**: Application-specific settings and integrations
  - **Brand**: White-label branding and customization settings
  - **Resource**: GRPM resource-specific configurations
  - **GLOBAL**: Platform-wide defaults that apply when no specific scope is set

- **API Shape**: The service exposes REST endpoints like `/settings_service/v1/:setting_cti_id` to create, read, update, and delete settings. Endpoints support query parameters for filtering by tenant, domain type, and domain object. API aliases like `/api/v1/settings` provide backward compatibility. Bulk operations for managing multiple settings in a single request are planned for v2 to improve performance.

- **Data Model**: Each setting type is defined by a JSON Schema that validates the configuration values. Settings include default values that apply when no custom value is set. When a setting is deleted, it's soft-deleted (marked as deleted but retained in the database) for a configurable retention period to support audit trails and recovery. Settings can be defined at the tenant level (applying to all child tenants) or at specific domain object levels (e.g., for a particular storage system or agent).

- **Events & Reporting**: Every configuration change triggers audit events (who, what, when) and notification events to alert interested systems. The service provides reporting endpoints that aggregate settings data for analytics dashboards and telemetry systems. The service also listens for lifecycle events from other systems (e.g., when a tenant is deleted) and automatically cleans up associated settings.

- **Performance & Constraints**: Read endpoints must handle at least 100 requests per second with sub-second response times. The service includes rate limiting and throttling mechanisms to prevent abuse and protect against high-volume workflows that could overwhelm the system.

## Important Constraints
- **Authorization**: Role-Based Access Control (RBAC) enforced through the Account Server. Each API request must include valid authentication tokens. The system checks user roles and scopes before allowing configuration changes. Origin tracking (identifying which application made the change) is planned but not yet implemented.
- **Compliance**: Security team review is mandatory before deploying changes. The service supports "compliance mode" for agent deployments where certain settings become read-only and cannot be modified without special permissions.
- **Validation**: All setting values must conform to JSON Schema v7 specification. Each setting type defines its own schema that validates data structure, required fields, data types, and value ranges before accepting changes.
- **Retention**: Deleted settings are not immediately removed from the database. Instead, they are marked as deleted and retained for a configurable period (e.g., 30 days) to support audit requirements and allow recovery if needed.

## External Dependencies
- **Account Server**: Provides the tenant hierarchy structure (parent-child relationships), validates user authorization scopes and permissions, and will eventually track which application originated each request for audit purposes.
- **Event Manager**: Receives and distributes audit events (configuration change logs) and notification events (alerts to interested systems). Also sends lifecycle events when tenants or domain objects are created, modified, or deleted so the Settings Service can maintain data consistency.
- **Reporting Service**: Consumes aggregated settings data for analytics dashboards, telemetry monitoring, and business intelligence reports.
- **Third-party Libraries**: Uses the jsonschema library for validating setting values against their defined schemas.
