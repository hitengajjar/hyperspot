# Design: Initial Settings Service Implementation

## Context

The Settings Service is implemented as a HyperSpot module using ModKit framework. Settings are identified by CTI (Component Type Identifier) with configuration metadata defined in CTI traits. The module follows HyperSpot's DDD-light architecture and integrates with the platform's module ecosystem.

**Stakeholders**: Development team, API consumers, other HyperSpot modules

## Goals / Non-Goals

### Goals
- Implement settings_service as a HyperSpot module with ModKit capabilities (db, rest)
- Follow DDD-light architecture (contract/, domain/, infra/, api/ layers)
- Store runtime data in database (type, tenant_id, domain_object_id, data)
- Define configuration metadata in CTI traits
- Support dynamic CTI type registration via API
- Provide native client contract for inter-module communication
- Register REST routes with api_ingress module
- Implement repository pattern with SeaORM

### Non-Goals
- Implementing standalone HTTP server (api_ingress handles this)
- Supporting legacy identification schemes
- Storing configuration metadata in database
- Modifying tenant hierarchy or inheritance mechanisms

## Decisions

### Decision 1: HyperSpot Module with ModKit
**What**: Implement settings_service as a HyperSpot module using ModKit framework with db and rest capabilities.

**Why**: 
- Follows HyperSpot's modular architecture
- Automatic integration with api_ingress for REST endpoints
- Built-in database capability with SeaORM support
- Native client registration in ClientHub for inter-module communication
- Lifecycle management (init, shutdown, migrations)
- Observability and error handling patterns

**Module Declaration**:
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

**Alternatives considered**:
1. Standalone service - Rejected: Doesn't integrate with HyperSpot ecosystem
2. Implement own HTTP server - Rejected: api_ingress provides this

**Trade-offs**: Depends on ModKit framework, but gains significant infrastructure benefits.

### Decision 2: DDD-Light Architecture
**What**: Follow HyperSpot's DDD-light structure with clear layer separation.

**Why**: 
- **contract/**: Transport-agnostic models and client trait (no serde)
- **domain/**: Business logic, service orchestration, repository traits
- **infra/**: SeaORM entities, repositories, database migrations
- **api/**: REST DTOs (with serde), handlers, routes, error mapping

Benefits:
- Clear separation of concerns
- Domain logic independent of transport
- Easy to test each layer
- Native client for inter-module calls (no HTTP overhead)

**Alternatives considered**:
1. Flat structure - Rejected: Harder to maintain and test
2. Full DDD with aggregates - Rejected: Over-engineering for this use case

### Decision 3: Repository Pattern with SeaORM
**What**: Implement repository pattern with traits in domain/ and SeaORM implementations in infra/.

**Why**:
- Repository traits define data access interface
- Generic over `ConnectionTrait + Send + Sync`
- SeaORM provides type-safe database operations
- Easy to mock for testing
- Supports PostgreSQL, MySQL, SQLite

**Database Schema**:
```sql
CREATE TABLE settings (
  type TEXT NOT NULL,
  tenant_id UUID NOT NULL,
  domain_object_id TEXT NOT NULL,
  data JSONB NOT NULL,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW(),
  deleted_at TIMESTAMP,
  PRIMARY KEY (type, tenant_id, domain_object_id)
);

CREATE TABLE cti_registry (
  type TEXT PRIMARY KEY,
  traits JSONB NOT NULL,
  schema JSONB,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);
```

**Alternatives considered**:
1. Direct SQL queries - Rejected: Less type-safe, harder to maintain
2. Active Record pattern - Rejected: Doesn't fit HyperSpot patterns

### Decision 4: REST API via api_ingress
**What**: Register REST routes with api_ingress module using OperationBuilder for OpenAPI documentation.

**Why**:
- api_ingress provides single HTTP server for all modules
- Automatic OpenAPI documentation generation
- Consistent error handling with RFC-9457 Problem Details
- Middleware for auth, tracing, rate limiting

**Route Registration**:
```rust
impl RestfulModule for SettingsServiceModule {
    async fn register_rest(
        &self,
        router: Router,
        openapi: &dyn OpenApiRegistry,
    ) -> Result<Router> {
        let service = self.service.get()?.clone();
        api::rest::routes::register_routes(router, openapi, service)
    }
}
```

**Handler Pattern**:
- Extract params via `Path<T>`, `Query<T>`, `Json<T>`
- Call domain service methods
- Map domain errors to HTTP via `api::rest::error::map_domain_error()`
- Return `Result<Json<T>, Problem>` or `Result<StatusCode, Problem>`

**Alternatives considered**:
1. Standalone HTTP server - Rejected: Doesn't integrate with HyperSpot
2. Manual route registration - Rejected: OperationBuilder provides OpenAPI docs

## Risks / Trade-offs

### Risk 1: CTI Trait Parsing Complexity
**Impact**: Medium - Parsing and validating CTI traits adds complexity
**Mitigation**: 
- Use well-tested YAML/JSON parsing libraries for Rust
- Implement comprehensive validation for trait structure
- Provide clear error messages for invalid trait definitions
- Create trait validation tests

### Risk 2: Performance with Composite Keys
**Impact**: Low - Composite primary key may affect query performance
**Mitigation**:
- Add appropriate indexes on type, tenant_id, domain_object_id
- Benchmark query performance during development
- Monitor query execution plans
- Use CTI prefix matching efficiently

### Risk 3: Configuration Management
**Impact**: Medium - CTI trait definitions must be managed carefully
**Mitigation**:
- Provide sensible defaults for common configurations
- Create configuration templates and examples
- Validate trait definitions at registration time
- Document all available trait options
- Implement validation to catch errors early

## Implementation Plan

### Phase 1: Module Scaffolding
1. Create module directory structure (contract/, domain/, infra/, api/)
2. Implement ModKit module declaration
3. Create typed configuration struct
4. Implement Module::init() lifecycle method
5. Implement DbModule::migrate() for schema migrations

### Phase 2: Database Layer
1. Create SeaORM entities for settings and cti_registry tables
2. Implement database migrations
3. Create entity ↔ model mappers
4. Implement repository traits in domain/
5. Implement SeaORM repositories in infra/storage/
6. Add OData mapper for filterable queries

### Phase 3: Domain Logic
1. Implement Setting and CTIType domain models in contract/
2. Create domain service with business logic
3. Implement CTI trait parsing and validation
4. Add JSON Schema validation for setting values
5. Implement event generation from CTI traits
6. Add unit tests for domain logic

### Phase 4: REST API
1. Create REST DTOs with serde derives
2. Implement thin HTTP handlers
3. Create route registration function with OperationBuilder
4. Implement error mapping to RFC-9457 Problem Details
5. Register routes in RestfulModule::register_rest()
6. Add integration tests for API endpoints

### Phase 5: Native Client Contract
1. Define SettingsApi trait in contract/client.rs
2. Create local adapter wrapping domain service
3. Register client in ClientHub during init
4. Add client usage examples
5. Test inter-module communication

### Phase 6: Testing & Documentation
1. Run full test suite (unit, integration, E2E)
2. Performance benchmarking
3. Document API endpoints and OpenAPI schema
4. Document module architecture and patterns
5. Create deployment and configuration guide

## Open Questions

1. **Q**: Which YAML/JSON library should we use for CTI trait parsing in Rust?
   **A**: TBD - Evaluate serde_yaml or serde_json based on performance

2. **Q**: Should we support JSON schema validation for setting data?
   **A**: Yes - per sm-CTI spec, validate data field against registered schema

3. **Q**: How should we handle CTI versioning in the type field?
   **A**: Store full CTI including version, support querying by prefix for version ranges
