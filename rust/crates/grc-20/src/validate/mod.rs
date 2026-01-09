//! Semantic validation for GRC-20 edits.
//!
//! This module provides validation beyond structural encoding checks.
//! Structural validation happens during decode; semantic validation
//! requires additional context (schema, entity state).

use std::collections::HashMap;

use crate::error::ValidationError;
use crate::model::{DataType, Edit, Id, Op, PropertyValue, Value};

/// Schema context for semantic validation.
#[derive(Debug, Clone, Default)]
pub struct SchemaContext {
    /// Known property data types.
    properties: HashMap<Id, DataType>,
}

impl SchemaContext {
    /// Creates a new empty schema context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a property with its data type.
    pub fn add_property(&mut self, id: Id, data_type: DataType) {
        self.properties.insert(id, data_type);
    }

    /// Gets the data type for a property, if known.
    pub fn get_property_type(&self, id: &Id) -> Option<DataType> {
        self.properties.get(id).copied()
    }
}

/// Validates an edit against a schema context.
///
/// This performs semantic validation that requires context:
/// - Value types match property data types
/// - DataType declarations are consistent with existing schema
///
/// Note: Entity lifecycle (DEAD/ALIVE) validation requires state context
/// and is not performed here.
pub fn validate_edit(edit: &Edit, schema: &SchemaContext) -> Result<(), ValidationError> {
    // Build a local schema from CreateProperty ops in this edit
    let mut local_schema = schema.clone();

    for op in &edit.ops {
        match op {
            Op::CreateProperty(cp) => {
                // Check consistency with existing schema
                if let Some(existing) = schema.get_property_type(&cp.id) {
                    if existing != cp.data_type {
                        return Err(ValidationError::DataTypeInconsistent {
                            property: cp.id,
                            schema: existing,
                            declared: cp.data_type,
                        });
                    }
                }
                local_schema.add_property(cp.id, cp.data_type);
            }
            Op::CreateEntity(ce) => {
                validate_property_values(&ce.values, &local_schema)?;
            }
            Op::UpdateEntity(ue) => {
                validate_property_values(&ue.set_properties, &local_schema)?;
                validate_property_values(&ue.add_values, &local_schema)?;
                validate_property_values(&ue.remove_values, &local_schema)?;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Validates that property values match their declared types.
fn validate_property_values(
    values: &[PropertyValue],
    schema: &SchemaContext,
) -> Result<(), ValidationError> {
    for pv in values {
        if let Some(expected_type) = schema.get_property_type(&pv.property) {
            let actual_type = pv.value.data_type();
            if expected_type != actual_type {
                return Err(ValidationError::TypeMismatch {
                    property: pv.property,
                    expected: expected_type,
                });
            }
        }
        // Note: If property is not in schema, we allow it (might be defined elsewhere)
    }
    Ok(())
}

/// Validates a single value (independent of property context).
///
/// This checks value-level constraints like:
/// - NaN not allowed in floats
/// - Point bounds
/// - Decimal normalization
/// - Position string format
pub fn validate_value(value: &Value) -> Option<&'static str> {
    value.validate()
}

/// Validates a position string according to spec rules.
///
/// Position strings must:
/// - Only contain characters 0-9, A-Z, a-z (62 chars)
/// - Not exceed 64 characters
pub fn validate_position(pos: &str) -> Result<(), &'static str> {
    crate::model::validate_position(pos)
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;
    use crate::model::{CreateEntity, CreateProperty};

    #[test]
    fn test_validate_type_mismatch() {
        let mut schema = SchemaContext::new();
        schema.add_property([1u8; 16], DataType::Int64);

        let edit = Edit {
            id: [0u8; 16],
            name: Cow::Borrowed(""),
            authors: vec![],
            created_at: 0,
            ops: vec![Op::CreateEntity(CreateEntity {
                id: [2u8; 16],
                values: vec![PropertyValue {
                    property: [1u8; 16],
                    value: Value::Text {
                        value: Cow::Owned("not an int".to_string()),
                        language: None,
                    },
                }],
            })],
        };

        let result = validate_edit(&edit, &schema);
        assert!(matches!(result, Err(ValidationError::TypeMismatch { .. })));
    }

    #[test]
    fn test_validate_type_match() {
        let mut schema = SchemaContext::new();
        schema.add_property([1u8; 16], DataType::Int64);

        let edit = Edit {
            id: [0u8; 16],
            name: Cow::Borrowed(""),
            authors: vec![],
            created_at: 0,
            ops: vec![Op::CreateEntity(CreateEntity {
                id: [2u8; 16],
                values: vec![PropertyValue {
                    property: [1u8; 16],
                    value: Value::Int64(42),
                }],
            })],
        };

        let result = validate_edit(&edit, &schema);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_datatype_inconsistent() {
        let mut schema = SchemaContext::new();
        schema.add_property([1u8; 16], DataType::Int64);

        let edit = Edit {
            id: [0u8; 16],
            name: Cow::Borrowed(""),
            authors: vec![],
            created_at: 0,
            ops: vec![Op::CreateProperty(CreateProperty {
                id: [1u8; 16],
                data_type: DataType::Text, // Conflicts with schema!
            })],
        };

        let result = validate_edit(&edit, &schema);
        assert!(matches!(
            result,
            Err(ValidationError::DataTypeInconsistent { .. })
        ));
    }

    #[test]
    fn test_validate_unknown_property() {
        let schema = SchemaContext::new(); // Empty schema

        let edit = Edit {
            id: [0u8; 16],
            name: Cow::Borrowed(""),
            authors: vec![],
            created_at: 0,
            ops: vec![Op::CreateEntity(CreateEntity {
                id: [2u8; 16],
                values: vec![PropertyValue {
                    property: [99u8; 16], // Unknown property
                    value: Value::Text {
                        value: Cow::Owned("test".to_string()),
                        language: None,
                    },
                }],
            })],
        };

        // Unknown properties are allowed (might be defined elsewhere)
        let result = validate_edit(&edit, &schema);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_inline_create_property() {
        let schema = SchemaContext::new();

        let edit = Edit {
            id: [0u8; 16],
            name: Cow::Borrowed(""),
            authors: vec![],
            created_at: 0,
            ops: vec![
                // First create the property
                Op::CreateProperty(CreateProperty {
                    id: [1u8; 16],
                    data_type: DataType::Text,
                }),
                // Then use it
                Op::CreateEntity(CreateEntity {
                    id: [2u8; 16],
                    values: vec![PropertyValue {
                        property: [1u8; 16],
                        value: Value::Text {
                            value: Cow::Owned("test".to_string()),
                            language: None,
                        },
                    }],
                }),
            ],
        };

        let result = validate_edit(&edit, &schema);
        assert!(result.is_ok());
    }
}
