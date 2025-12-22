# Implementation Tasks

## 1. Database Schema Changes
- [ ] 1.1 Create cti_registry table: (type PK, traits JSONB, schema JSONB, created_at, updated_at)
- [ ] 1.2 Create settings table with sm-CTI schema: (type, tenant_id, domain_object_id, data, created_at, updated_at)
- [ ] 1.3 Set composite primary key on settings: (type, tenant_id, domain_object_id)
- [ ] 1.4 Add foreign key: settings.type -> cti_registry.type
- [ ] 1.5 Add index on settings.type for CTI prefix filtering
- [ ] 1.6 Add indexes on tenant_id and domain_object_id for lookups
- [ ] 1.7 Create test data with CTI registrations and settings

## 2. Domain Model Updates
- [ ] 2.1 Create new Setting entity matching sm-CTI spec:
  - type: String (CTI)
  - tenant_id: UUID
  - domain_object_id: String (UUID | CTI | AppCode | "generic")
  - data: JSON
  - created_at, updated_at: Timestamp
- [ ] 2.2 Create CTIType entity for registry:
  - type: String (CTI)
  - traits: JSON (domain_type, events, options, operations)
  - schema: JSON (validation schema)
  - created_at, updated_at: Timestamp
- [ ] 2.3 Create Setting DTO matching sm-CTI spec
- [ ] 2.4 Create CTIType DTO for registration API
- [ ] 2.5 Add CTI trait parser for domain_type, events, options, operations

## 3. Repository Layer Changes
- [ ] 3.1 Create CTI registry repository with CRUD operations
- [ ] 3.2 Create settings repository for sm-CTI schema
- [ ] 3.3 Implement queries using composite key (type, tenant_id, domain_object_id)
- [ ] 3.4 Add filtering by type (CTI prefix matching)
- [ ] 3.5 Add foreign key validation for settings.type
- [ ] 3.6 Create repository tests for CTI-based operations

## 4. Domain Manager Updates
- [ ] 4.1 Create settings manager to use CTI trait configuration
- [ ] 4.2 Add CTI trait resolution logic (domain_type, events, options)
- [ ] 4.3 Implement inheritance logic from CTI options traits
- [ ] 4.4 Create manager tests for CTI-based operations

## 5. API Layer Changes - Implement APIs
- [ ] 5.1 Implement CTI Registration APIs:
  - [ ] 5.1.1 `POST /cti-types` - Register new CTI type
  - [ ] 5.1.2 `GET /cti-types` - List registered CTI types
  - [ ] 5.1.3 `GET /cti-types/{type}` - Get CTI type by identifier
  - [ ] 5.1.4 `PUT /cti-types/{type}` - Update CTI type
  - [ ] 5.1.5 `DELETE /cti-types/{type}` - Delete CTI type
- [ ] 5.2 Implement sm-CTI Settings APIs:
  - [ ] 5.2.1 `GET /settings` - List all settings with filters
  - [ ] 5.2.2 `GET /settings/{setting_type}` - Get setting by CTI
  - [ ] 5.2.3 `PUT /settings/{setting_type}` - Update setting value
  - [ ] 5.2.4 `DELETE /settings/{setting_type}` - Remove setting value
  - [ ] 5.2.5 `PUT /settings/{setting_type}/lock` - Lock setting (compliance)
- [ ] 5.3 Add query parameter support: tenant_id, domain_object_id, type, domain_type, etc.
- [ ] 5.4 Update API responses to match sm-CTI types
- [ ] 5.5 Add validation: settings.type must exist in cti_registry

## 6. Configuration Updates
- [ ] 6.1 Create CTI type definition format with traits (YAML/JSON)
- [ ] 6.2 Implement CTI trait parser: domain_type, events, options, operations
- [ ] 6.3 Add CTI format validation (cti.a.p.sm.setting.v1.0~vendor.app.feature.v1.0)
- [ ] 6.4 Create static config files using CTI types
- [ ] 6.5 Add schema validation for setting data per CTI type

## 7. Event System Updates
- [ ] 7.1 Update audit event generation to read from CTI traits
- [ ] 7.2 Update notification event generation to read from CTI traits
- [ ] 7.3 Implement event target resolution (SELF, SUBROOT, NONE) from traits
- [ ] 7.4 Create event structures using CTI type field

## 8. Testing
- [ ] 8.1 Create unit tests for CTI trait parsing
- [ ] 8.2 Create unit tests for CTIType entity and repository
- [ ] 8.3 Create unit tests for sm-CTI Setting entity
- [ ] 8.4 Create integration tests for CTI registration APIs
- [ ] 8.5 Create integration tests for sm-CTI settings APIs
- [ ] 8.6 Add validation tests: settings.type must exist in registry
- [ ] 8.7 Add API tests for all endpoints (CTI registration + settings)

## 9. Documentation
- [ ] 9.1 Document sm-CTI API endpoints with examples
- [ ] 9.2 Document CTI type definition format with trait examples
- [ ] 9.3 Update architecture documentation
- [ ] 9.4 Document API specification and usage patterns
