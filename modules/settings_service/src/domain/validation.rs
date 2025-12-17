//! JSON Schema validation for setting values

use crate::contract::SettingsError;
use jsonschema::Validator;
use serde_json::Value;
use uuid::Uuid;

/// Validate domain_object_id format
/// 
/// Accepts:
/// - "generic" - literal string
/// - UUID format (e.g., "550e8400-e29b-41d4-a716-446655440000")
/// - GTS format (e.g., "gts.a.p.sm.storage.v1.0~vendor.app.v1.0")
/// - AppCode format (alphanumeric with _, ., - but must start with alphanumeric)
pub fn validate_domain_object_id(domain_object_id: &str) -> Result<(), SettingsError> {
    // Empty string is invalid
    if domain_object_id.is_empty() {
        return Err(SettingsError::Validation {
            message: "domain_object_id cannot be empty".to_string(),
        });
    }

    // Check for "generic" literal
    if domain_object_id == "generic" {
        return Ok(());
    }

    // Check for UUID format
    if Uuid::parse_str(domain_object_id).is_ok() {
        return Ok(());
    }

    // Check for GTS format (contains "gts." prefix and "~")
    if domain_object_id.starts_with("gts.") && domain_object_id.contains('~') {
        return Ok(());
    }

    // Check for AppCode format: must start with alphanumeric, can contain alphanumeric, _, ., -
    let first_char = domain_object_id.chars().next().unwrap();
    if !first_char.is_alphanumeric() {
        return Err(SettingsError::Validation {
            message: format!(
                "domain_object_id '{}' must start with alphanumeric character",
                domain_object_id
            ),
        });
    }

    // Check if all characters are valid (alphanumeric, _, ., -)
    let is_valid_appcode = domain_object_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '-');

    if !is_valid_appcode {
        return Err(SettingsError::Validation {
            message: format!(
                "domain_object_id '{}' contains invalid characters. Only alphanumeric, '_', '.', and '-' are allowed",
                domain_object_id
            ),
        });
    }

    // Check if it's not only special characters
    let has_alphanumeric = domain_object_id.chars().any(|c| c.is_alphanumeric());
    if !has_alphanumeric {
        return Err(SettingsError::Validation {
            message: format!(
                "domain_object_id '{}' must contain at least one alphanumeric character",
                domain_object_id
            ),
        });
    }

    Ok(())
}

/// Validate a setting value against a JSON Schema
pub fn validate_against_schema(data: &Value, schema: &Value) -> Result<(), SettingsError> {
    // Compile the schema
    let validator = Validator::new(schema)
        .map_err(|e| SettingsError::Validation {
            message: format!("Invalid JSON Schema: {}", e),
        })?;

    // Validate the data
    if let Err(error) = validator.validate(data) {
        return Err(SettingsError::SchemaValidation {
            errors: vec![error.to_string()],
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_domain_object_id_generic() {
        assert!(validate_domain_object_id("generic").is_ok());
    }

    #[test]
    fn test_validate_domain_object_id_uuid() {
        assert!(validate_domain_object_id("550e8400-e29b-41d4-a716-446655440000").is_ok());
        let uuid = Uuid::new_v4().to_string();
        assert!(validate_domain_object_id(&uuid).is_ok());
    }

    #[test]
    fn test_validate_domain_object_id_gts() {
        assert!(validate_domain_object_id("gts.a.p.sm.storage.v1.0~vendor.app.v1.0").is_ok());
        assert!(validate_domain_object_id("gts.a.p.sm.setting.v1.0~backup.schedule.v1.0").is_ok());
        assert!(validate_domain_object_id("gts.x.y.z~test.v2.0").is_ok());
    }

    #[test]
    fn test_validate_domain_object_id_appcode() {
        assert!(validate_domain_object_id("app.backup.v1").is_ok());
        assert!(validate_domain_object_id("my_app_123").is_ok());
        assert!(validate_domain_object_id("app-code-v2").is_ok());
        assert!(validate_domain_object_id("BackupAgent").is_ok());
        assert!(validate_domain_object_id("agent_v1.2.3").is_ok());
    }

    #[test]
    fn test_validate_domain_object_id_invalid() {
        assert!(validate_domain_object_id("_invalid").is_err());
        assert!(validate_domain_object_id("-invalid").is_err());
        assert!(validate_domain_object_id(".invalid").is_err());
        assert!(validate_domain_object_id("app@code").is_err());
        assert!(validate_domain_object_id("app code").is_err());
        assert!(validate_domain_object_id("app#code").is_err());
        assert!(validate_domain_object_id("").is_err());
        assert!(validate_domain_object_id("___").is_err());
    }

    #[test]
    fn test_valid_schema_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "number", "minimum": 0 }
            },
            "required": ["name"]
        });

        let data = json!({
            "name": "John",
            "age": 30
        });

        assert!(validate_against_schema(&data, &schema).is_ok());
    }

    #[test]
    fn test_invalid_schema_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "number", "minimum": 0 }
            },
            "required": ["name"]
        });

        let data = json!({
            "age": -5  // Missing required "name" and invalid age
        });

        let result = validate_against_schema(&data, &schema);
        assert!(result.is_err());
        
        if let Err(SettingsError::SchemaValidation { errors }) = result {
            assert!(!errors.is_empty());
        } else {
            panic!("Expected SchemaValidation error");
        }
    }

    #[test]
    fn test_type_mismatch() {
        let schema = json!({
            "type": "object",
            "properties": {
                "count": { "type": "integer" }
            }
        });

        let data = json!({
            "count": "not a number"
        });

        let result = validate_against_schema(&data, &schema);
        assert!(result.is_err());
    }
}
