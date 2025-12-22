# Design: Tenant Hierarchy Integration

## Context

The settings service currently operates in isolation without validating tenant existence or properly traversing the organizational hierarchy managed by the account server. This creates a disconnect between the logical tenant structure and settings inheritance behavior.

**Stakeholders:**
- Settings service consumers expecting proper hierarchy-aware inheritance
- Account server team providing tenant hierarchy APIs
- Platform architects ensuring consistent multi-tenancy patterns

**Constraints:**
- Account server may not be available in all deployment environments (dev, test)
- Must support both mock and real hierarchy clients
- Cannot break existing settings service functionality
- Must maintain performance (avoid N+1 queries)

## Goals / Non-Goals

**Goals:**
- Validate tenant existence before creating/modifying settings
- Traverse actual tenant hierarchy for inheritance resolution
- Support default values in GTS type definitions
- Provide abstraction for hierarchy service communication
- Enable testing without account server dependency

**Non-Goals:**
- Caching tenant hierarchy (future optimization)
- Implementing the account server hierarchy API (separate service)
- Migrating existing settings to new resolution logic (backward compatible)
- Real-time hierarchy change notifications (future enhancement)

## Decisions

### Decision 1: Trait-Based Hierarchy Client
**What:** Define `TenantHierarchyClient` trait with async methods for hierarchy operations

**Why:**
- Enables dependency injection and testing with mocks
- Allows switching between mock and real implementations
- Follows Rust best practices for abstraction
- Supports multiple transport protocols (gRPC, REST)

**Alternatives considered:**
- Direct gRPC client coupling: Rejected due to tight coupling and testing difficulty
- Callback-based API: Rejected due to complexity and poor ergonomics

### Decision 2: Resolution Priority with Default Values
**What:** Implement hierarchical resolution with level-by-level checking:
1. Check requested tenant: explicit setting (tenant_id, domain_object_id)
2. Check requested tenant: generic setting (tenant_id, "generic")
3. For each parent tenant (child to root):
   - Check explicit setting (parent_tenant_id, domain_object_id)
   - Check generic setting (parent_tenant_id, "generic")
   - Stop if is_barrier_inheritance=true
4. Return default value from GTS type
5. Return NotFound error

**Why:**
- Checks both exact and generic at each hierarchy level before moving up
- More efficient than checking all exact settings first, then all generic
- Provides clear, predictable resolution order per tenant level
- Supports gradual rollout (defaults → overrides)
- Enables "factory defaults" pattern
- Backward compatible with existing behavior

**Alternatives considered:**
- Check all exact settings in hierarchy, then all generic: Rejected as less intuitive
- 3-level priority (local → parent → error): Rejected as insufficient
- Database-level defaults: Rejected due to schema complexity

### Decision 3: Lazy Hierarchy Traversal
**What:** Only query hierarchy service when local setting not found

**Why:**
- Minimizes external service calls
- Optimizes common case (setting exists locally)
- Reduces latency for direct lookups
- Limits blast radius of hierarchy service outages

**Alternatives considered:**
- Eager hierarchy loading: Rejected due to performance overhead
- Cached hierarchy paths: Deferred to future optimization

### Decision 4: Mock-First Development
**What:** Implement `MockTenantHierarchyClient` first, real client as Phase 2

**Why:**
- Enables immediate testing without account server
- Allows settings service development to proceed independently
- Provides reference implementation for real client
- Supports local development environments

**Alternatives considered:**
- Real client first: Rejected due to account server API not finalized
- No mock: Rejected due to testing and development friction

### Decision 5: Default Value Storage
**What:** Store default_value as JSONB column in gts_types table

**Why:**
- Keeps default with type definition (single source of truth)
- Enables schema validation of defaults
- Supports complex default structures
- Allows per-type customization

**Alternatives considered:**
- Separate defaults table: Rejected as over-engineering
- Code-level defaults: Rejected as inflexible
- NULL as default: Rejected as ambiguous

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Settings Service                          │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              Domain Service                            │ │
│  │                                                        │ │
│  │  resolve_setting(tenant_id, domain_object_id)         │ │
│  │    1. Check requested tenant:                         │ │
│  │       - Explicit (tenant_id, domain_object_id)        │ │
│  │       - Generic (tenant_id, "generic")                │ │
│  │    2. Get tenant path from hierarchy client           │ │
│  │    3. For each parent tenant (child→root):            │ │
│  │       - Check explicit (parent_id, domain_object_id)  │ │
│  │       - Check generic (parent_id, "generic")          │ │
│  │       - Stop if is_barrier_inheritance=true           │ │
│  │    4. Return default value from GTS type              │ │
│  │    5. Return NotFound                                 │ │
│  └────────────────────────────────────────────────────────┘ │
│                           │                                  │
│                           ▼                                  │
│  ┌────────────────────────────────────────────────────────┐ │
│  │       TenantHierarchyClient (trait)                    │ │
│  │  - get_parent_tenant(tenant_id) -> Option<Uuid>       │ │
│  │  - validate_tenant_exists(tenant_id) -> bool          │ │
│  │  - get_tenant_path(tenant_id) -> Vec<Uuid>            │ │
│  └────────────────────────────────────────────────────────┘ │
│              │                              │                │
└──────────────┼──────────────────────────────┼────────────────┘
               │                              │
               ▼                              ▼
    ┌──────────────────────┐      ┌──────────────────────┐
    │ MockHierarchyClient  │      │  RealHierarchyClient │
    │  (in-memory map)     │      │   (gRPC/REST)        │
    └──────────────────────┘      └──────────────────────┘
                                            │
                                            ▼
                                  ┌──────────────────────┐
                                  │   Account Server     │
                                  │  Tenant Hierarchy    │
                                  │       API            │
                                  └──────────────────────┘
```

## Risks / Trade-offs

### Risk 1: Account Server Dependency
**Risk:** Settings service becomes unavailable if account server is down

**Mitigation:**
- Use circuit breaker pattern for hierarchy calls
- Cache tenant validation results (short TTL)
- Provide degraded mode (skip validation) via feature flag
- Monitor hierarchy service health

### Risk 2: Performance Impact
**Risk:** Hierarchy traversal adds latency to get_setting() calls

**Mitigation:**
- Lazy traversal (only when needed)
- Batch hierarchy queries where possible
- Add metrics to measure impact
- Consider caching tenant paths (future)

### Risk 3: Inconsistent Hierarchy State
**Risk:** Tenant hierarchy changes while settings are being resolved

**Mitigation:**
- Accept eventual consistency model
- Document that hierarchy changes may take time to reflect
- Provide admin API to trigger hierarchy refresh (future)
- Use versioned hierarchy snapshots (future enhancement)

### Risk 4: Migration Complexity
**Risk:** Existing settings may not align with new resolution logic

**Mitigation:**
- New logic is additive (doesn't break existing settings)
- Default values are optional
- Provide migration guide for teams
- Support gradual rollout per GTS type

## Migration Plan

### Phase 1: Foundation (Current Change)
1. Add default_value to GtsType model and database
2. Implement TenantHierarchyClient trait
3. Create MockTenantHierarchyClient
4. Update resolve_setting() with new logic
5. Add comprehensive tests with mock client

### Phase 2: Real Integration (Future)
1. Define account server hierarchy API contract
2. Implement RealTenantHierarchyClient (gRPC or REST)
3. Add configuration for hierarchy service endpoint
4. Add feature flag to switch between mock and real
5. Deploy with mock enabled, gradually enable real client

### Phase 3: Optimization (Future)
1. Add tenant path caching
2. Implement batch hierarchy queries
3. Add circuit breaker for hierarchy calls
4. Monitor and tune performance

### Rollback Plan
- Feature flag allows instant rollback to mock client
- Database migration is additive (can be rolled back)
- Default values are optional (no data migration needed)
- Existing settings continue to work without defaults

## Open Questions

1. **Account Server API Contract:** What protocol (gRPC vs REST) will account server use?
   - **Decision needed by:** Before Phase 2 implementation
   - **Impact:** Determines RealHierarchyClient implementation

2. **Caching Strategy:** Should we cache tenant paths? For how long?
   - **Decision needed by:** Performance testing in staging
   - **Impact:** Affects consistency vs performance trade-off

3. **Hierarchy Change Notifications:** How to handle real-time hierarchy updates?
   - **Decision needed by:** After initial deployment
   - **Impact:** Determines if we need event-driven updates

4. **Default Value Versioning:** Should default values be versioned with GTS types?
   - **Decision needed by:** Before production use
   - **Impact:** Affects migration and rollback strategies

5. **Multi-Region Hierarchy:** How to handle cross-region tenant hierarchies?
   - **Decision needed by:** Multi-region deployment planning
   - **Impact:** May require distributed hierarchy service
