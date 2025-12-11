//! JSON Schema validation for setting values

use crate::contract::SettingsError;
use jsonschema::Validator;
use serde_json::Value;

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
