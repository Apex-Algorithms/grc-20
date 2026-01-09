//! Data model types for GRC-20.
//!
//! This module contains all the core types for representing GRC-20 data:
//! - Identifiers (UUIDs)
//! - Values (typed property instances)
//! - Operations (state changes)
//! - Edits (batched operations)

pub mod edit;
pub mod id;
pub mod op;
pub mod value;

pub use edit::{DictionaryBuilder, Edit, WireDictionaries};
pub use id::{derived_uuid, format_id, parse_id, text_value_id, unique_relation_id, value_id, Id, NIL_ID};
pub use op::{
    validate_position, CreateEntity, CreateProperty, CreateRelation, DeleteEntity, DeleteRelation,
    Op, RelationIdMode, UnsetProperty, UpdateEntity, UpdateRelation,
};
pub use value::{DataType, DecimalMantissa, EmbeddingSubType, Property, PropertyValue, Value};
