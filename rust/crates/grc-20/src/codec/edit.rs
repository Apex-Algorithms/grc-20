//! Edit encoding/decoding for GRC-20 binary format.
//!
//! Implements the wire format for edits (spec Section 6.3).

use std::borrow::Cow;
use std::collections::HashMap;
use std::io::Read;

use crate::codec::op::{decode_op, encode_op};
use crate::codec::primitives::{Reader, Writer};
use crate::error::{DecodeError, EncodeError};
use crate::limits::{
    FORMAT_VERSION, MAGIC_COMPRESSED, MAGIC_UNCOMPRESSED, MAX_AUTHORS, MAX_DICT_SIZE,
    MAX_EDIT_SIZE, MAX_OPS_PER_EDIT, MAX_STRING_LEN,
};
use crate::model::{
    DataType, DictionaryBuilder, Edit, Id, Op, PropertyValue, Value, WireDictionaries,
};

// =============================================================================
// DECODING
// =============================================================================

/// Decodes an Edit from binary data.
///
/// Automatically detects and handles zstd compression (GRC2Z magic).
pub fn decode_edit(input: &[u8]) -> Result<Edit, DecodeError> {
    if input.len() < 4 {
        return Err(DecodeError::UnexpectedEof { context: "magic" });
    }

    // Detect compression
    let data: Cow<[u8]> = if input.len() >= 5 && &input[0..5] == MAGIC_COMPRESSED {
        let decompressed = decompress_zstd(&input[5..])?;
        if decompressed.len() > MAX_EDIT_SIZE {
            return Err(DecodeError::LengthExceedsLimit {
                field: "edit",
                len: decompressed.len(),
                max: MAX_EDIT_SIZE,
            });
        }
        Cow::Owned(decompressed)
    } else if &input[0..4] == MAGIC_UNCOMPRESSED {
        if input.len() > MAX_EDIT_SIZE {
            return Err(DecodeError::LengthExceedsLimit {
                field: "edit",
                len: input.len(),
                max: MAX_EDIT_SIZE,
            });
        }
        Cow::Borrowed(input)
    } else {
        let mut found = [0u8; 4];
        found.copy_from_slice(&input[0..4]);
        return Err(DecodeError::InvalidMagic { found });
    };

    let mut reader = Reader::new(&data);

    // Skip magic (already validated)
    reader.read_bytes(4, "magic")?;

    // Version
    let version = reader.read_byte("version")?;
    if version != FORMAT_VERSION {
        return Err(DecodeError::UnsupportedVersion { version });
    }

    // Header
    let edit_id = reader.read_id("edit_id")?;
    let name = reader.read_string(MAX_STRING_LEN, "name")?;
    let authors = reader.read_id_vec(MAX_AUTHORS, "authors")?;
    let created_at = reader.read_signed_varint("created_at")?;

    // Schema dictionaries
    let property_count = reader.read_varint("property_count")? as usize;
    if property_count > MAX_DICT_SIZE {
        return Err(DecodeError::LengthExceedsLimit {
            field: "properties",
            len: property_count,
            max: MAX_DICT_SIZE,
        });
    }
    let mut properties = Vec::with_capacity(property_count);
    for _ in 0..property_count {
        let id = reader.read_id("property_id")?;
        let dt_byte = reader.read_byte("data_type")?;
        let data_type = DataType::from_u8(dt_byte)
            .ok_or(DecodeError::InvalidDataType { data_type: dt_byte })?;
        properties.push((id, data_type));
    }

    let relation_types = reader.read_id_vec(MAX_DICT_SIZE, "relation_types")?;
    let languages = reader.read_id_vec(MAX_DICT_SIZE, "languages")?;
    let objects = reader.read_id_vec(MAX_DICT_SIZE, "objects")?;

    let dicts = WireDictionaries {
        properties,
        relation_types,
        languages,
        objects,
    };

    // Operations
    let op_count = reader.read_varint("op_count")? as usize;
    if op_count > MAX_OPS_PER_EDIT {
        return Err(DecodeError::LengthExceedsLimit {
            field: "ops",
            len: op_count,
            max: MAX_OPS_PER_EDIT,
        });
    }

    let mut ops = Vec::with_capacity(op_count);
    for _ in 0..op_count {
        ops.push(decode_op(&mut reader, &dicts)?);
    }

    Ok(Edit {
        id: edit_id,
        name,
        authors,
        created_at,
        ops,
    })
}

fn decompress_zstd(compressed: &[u8]) -> Result<Vec<u8>, DecodeError> {
    // Read uncompressed size
    let mut reader = Reader::new(compressed);
    let declared_size = reader.read_varint("uncompressed_size")? as usize;

    if declared_size > MAX_EDIT_SIZE {
        return Err(DecodeError::LengthExceedsLimit {
            field: "uncompressed_size",
            len: declared_size,
            max: MAX_EDIT_SIZE,
        });
    }

    let compressed_data = reader.remaining();

    let mut decoder = zstd::Decoder::new(compressed_data)
        .map_err(|e| DecodeError::DecompressionFailed(e.to_string()))?;

    let mut decompressed = Vec::with_capacity(declared_size);
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| DecodeError::DecompressionFailed(e.to_string()))?;

    if decompressed.len() != declared_size {
        return Err(DecodeError::UncompressedSizeMismatch {
            declared: declared_size,
            actual: decompressed.len(),
        });
    }

    Ok(decompressed)
}

// =============================================================================
// ENCODING
// =============================================================================

/// Encodes an Edit to binary format (uncompressed).
pub fn encode_edit(edit: &Edit) -> Result<Vec<u8>, EncodeError> {
    // First pass: collect all IDs and build dictionaries
    let (dict_builder, property_types) = collect_edit_ids(edit);
    let dicts = dict_builder.build();

    // Second pass: encode
    let mut writer = Writer::with_capacity(estimate_edit_size(edit));

    // Magic and version
    writer.write_bytes(MAGIC_UNCOMPRESSED);
    writer.write_byte(FORMAT_VERSION);

    // Header
    writer.write_id(&edit.id);
    writer.write_string(&edit.name);
    writer.write_id_vec(&edit.authors);
    writer.write_signed_varint(edit.created_at);

    // Schema dictionaries
    writer.write_varint(dicts.properties.len() as u64);
    for (id, data_type) in &dicts.properties {
        writer.write_id(id);
        writer.write_byte(*data_type as u8);
    }

    writer.write_id_vec(&dicts.relation_types);
    writer.write_id_vec(&dicts.languages);
    writer.write_id_vec(&dicts.objects);

    // Operations
    writer.write_varint(edit.ops.len() as u64);

    // Re-create dict_builder for encoding (to get correct indices)
    let mut dict_builder = DictionaryBuilder::new();
    // Pre-populate with the same order
    for (id, data_type) in &dicts.properties {
        dict_builder.add_property(*id, *data_type);
    }
    for id in &dicts.relation_types {
        dict_builder.add_relation_type(*id);
    }
    for id in &dicts.languages {
        dict_builder.add_language(Some(*id));
    }
    for id in &dicts.objects {
        dict_builder.add_object(*id);
    }

    for op in &edit.ops {
        encode_op(&mut writer, op, &mut dict_builder, &property_types)?;
    }

    Ok(writer.into_bytes())
}

/// Encodes an Edit to binary format with zstd compression.
pub fn encode_edit_compressed(edit: &Edit, level: i32) -> Result<Vec<u8>, EncodeError> {
    let uncompressed = encode_edit(edit)?;

    let compressed = zstd::encode_all(uncompressed.as_slice(), level)
        .map_err(|e| EncodeError::CompressionFailed(e.to_string()))?;

    let mut writer = Writer::with_capacity(5 + 10 + compressed.len());
    writer.write_bytes(MAGIC_COMPRESSED);
    writer.write_varint(uncompressed.len() as u64);
    writer.write_bytes(&compressed);

    Ok(writer.into_bytes())
}

/// Collects all IDs from an edit and builds the dictionaries.
fn collect_edit_ids(edit: &Edit) -> (DictionaryBuilder, HashMap<Id, DataType>) {
    let mut builder = DictionaryBuilder::new();
    let mut property_types: HashMap<Id, DataType> = HashMap::new();

    for op in &edit.ops {
        match op {
            Op::CreateEntity(ce) => {
                collect_property_values(&ce.values, &mut builder, &mut property_types);
            }
            Op::UpdateEntity(ue) => {
                builder.add_object(ue.id);
                collect_property_values(&ue.set_properties, &mut builder, &mut property_types);
                collect_property_values(&ue.add_values, &mut builder, &mut property_types);
                collect_property_values(&ue.remove_values, &mut builder, &mut property_types);
                for prop_id in &ue.unset_properties {
                    // For unset, we don't know the type, use a placeholder
                    builder.add_property(*prop_id, DataType::Bool);
                }
            }
            Op::DeleteEntity(de) => {
                builder.add_object(de.id);
            }
            Op::CreateRelation(cr) => {
                builder.add_relation_type(cr.relation_type);
                builder.add_object(cr.from);
                builder.add_object(cr.to);
            }
            Op::UpdateRelation(ur) => {
                builder.add_object(ur.id);
            }
            Op::DeleteRelation(dr) => {
                builder.add_object(dr.id);
            }
            Op::CreateProperty(cp) => {
                property_types.insert(cp.id, cp.data_type);
            }
        }
    }

    (builder, property_types)
}

fn collect_property_values(
    values: &[PropertyValue],
    builder: &mut DictionaryBuilder,
    property_types: &mut HashMap<Id, DataType>,
) {
    for pv in values {
        let data_type = property_types
            .get(&pv.property)
            .copied()
            .unwrap_or_else(|| pv.value.data_type());
        builder.add_property(pv.property, data_type);
        property_types.insert(pv.property, data_type);

        // Collect language for TEXT values
        if let Value::Text { language, .. } = &pv.value {
            builder.add_language(*language);
        }

        // Collect REF targets
        if let Value::Ref(id) = &pv.value {
            builder.add_object(*id);
        }
    }
}

fn estimate_edit_size(edit: &Edit) -> usize {
    // Rough estimate: 1KB base + 100 bytes per op
    1024 + edit.ops.len() * 100
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CreateEntity, CreateProperty, PropertyValue, Value};

    fn make_test_edit() -> Edit {
        Edit {
            id: [1u8; 16],
            name: "Test Edit".to_string(),
            authors: vec![[2u8; 16]],
            created_at: 1234567890,
            ops: vec![
                Op::CreateProperty(CreateProperty {
                    id: [10u8; 16],
                    data_type: DataType::Text,
                }),
                Op::CreateEntity(CreateEntity {
                    id: [3u8; 16],
                    values: vec![PropertyValue {
                        property: [10u8; 16],
                        value: Value::Text {
                            value: "Hello".to_string(),
                            language: None,
                        },
                    }],
                }),
            ],
        }
    }

    #[test]
    fn test_edit_roundtrip() {
        let edit = make_test_edit();

        let encoded = encode_edit(&edit).unwrap();
        let decoded = decode_edit(&encoded).unwrap();

        assert_eq!(edit.id, decoded.id);
        assert_eq!(edit.name, decoded.name);
        assert_eq!(edit.authors, decoded.authors);
        assert_eq!(edit.created_at, decoded.created_at);
        assert_eq!(edit.ops.len(), decoded.ops.len());
    }

    #[test]
    fn test_edit_compressed_roundtrip() {
        let edit = make_test_edit();

        let encoded = encode_edit_compressed(&edit, 3).unwrap();
        let decoded = decode_edit(&encoded).unwrap();

        assert_eq!(edit.id, decoded.id);
        assert_eq!(edit.name, decoded.name);
        assert_eq!(edit.authors, decoded.authors);
        assert_eq!(edit.created_at, decoded.created_at);
        assert_eq!(edit.ops.len(), decoded.ops.len());
    }

    #[test]
    fn test_compression_magic() {
        let edit = make_test_edit();

        let uncompressed = encode_edit(&edit).unwrap();
        let compressed = encode_edit_compressed(&edit, 3).unwrap();

        assert_eq!(&uncompressed[0..4], b"GRC2");
        assert_eq!(&compressed[0..5], b"GRC2Z");
    }

    #[test]
    fn test_invalid_magic() {
        let data = b"XXXX";
        let result = decode_edit(data);
        assert!(matches!(result, Err(DecodeError::InvalidMagic { .. })));
    }

    #[test]
    fn test_unsupported_version() {
        let mut data = Vec::new();
        data.extend_from_slice(MAGIC_UNCOMPRESSED);
        data.push(99); // Invalid version
        // Add enough bytes to not trigger EOF
        data.extend_from_slice(&[0u8; 100]);

        let result = decode_edit(&data);
        assert!(matches!(result, Err(DecodeError::UnsupportedVersion { version: 99 })));
    }

    #[test]
    fn test_empty_edit() {
        let edit = Edit {
            id: [0u8; 16],
            name: String::new(),
            authors: vec![],
            created_at: 0,
            ops: vec![],
        };

        let encoded = encode_edit(&edit).unwrap();
        let decoded = decode_edit(&encoded).unwrap();

        assert_eq!(edit.id, decoded.id);
        assert!(decoded.name.is_empty());
        assert!(decoded.authors.is_empty());
        assert!(decoded.ops.is_empty());
    }
}
