//! Value encoding/decoding for GRC-20 binary format.
//!
//! Implements the wire format for property values (spec Section 6.5).

use std::borrow::Cow;

use crate::codec::primitives::{Reader, Writer};
use crate::error::{DecodeError, EncodeError};
use crate::limits::{MAX_BYTES_LEN, MAX_EMBEDDING_BYTES, MAX_EMBEDDING_DIMS, MAX_POSITION_LEN, MAX_STRING_LEN};
use crate::model::{
    DataType, DecimalMantissa, DictionaryBuilder, EmbeddingSubType, PropertyValue, Value,
    WireDictionaries,
};

// =============================================================================
// DECODING
// =============================================================================

/// Decodes a Value from the reader based on the data type (zero-copy).
pub fn decode_value<'a>(
    reader: &mut Reader<'a>,
    data_type: DataType,
    dicts: &WireDictionaries,
) -> Result<Value<'a>, DecodeError> {
    match data_type {
        DataType::Bool => decode_bool(reader),
        DataType::Int64 => decode_int64(reader, dicts),
        DataType::Float64 => decode_float64(reader, dicts),
        DataType::Decimal => decode_decimal(reader, dicts),
        DataType::Text => decode_text(reader, dicts),
        DataType::Bytes => decode_bytes(reader),
        DataType::Timestamp => decode_timestamp(reader),
        DataType::Date => decode_date(reader),
        DataType::Point => decode_point(reader),
        DataType::Embedding => decode_embedding(reader),
    }
}

fn decode_bool<'a>(reader: &mut Reader<'a>) -> Result<Value<'a>, DecodeError> {
    let byte = reader.read_byte("bool")?;
    match byte {
        0x00 => Ok(Value::Bool(false)),
        0x01 => Ok(Value::Bool(true)),
        _ => Err(DecodeError::InvalidBool { value: byte }),
    }
}

fn decode_int64<'a>(reader: &mut Reader<'a>, dicts: &WireDictionaries) -> Result<Value<'a>, DecodeError> {
    let value = reader.read_signed_varint("int64")?;
    let unit_index = reader.read_varint("int64.unit")? as usize;
    let unit = if unit_index == 0 {
        None
    } else {
        let idx = unit_index - 1;
        if idx >= dicts.units.len() {
            return Err(DecodeError::IndexOutOfBounds {
                dict: "units",
                index: unit_index,
                size: dicts.units.len() + 1,
            });
        }
        Some(dicts.units[idx])
    };
    Ok(Value::Int64 { value, unit })
}

fn decode_float64<'a>(reader: &mut Reader<'a>, dicts: &WireDictionaries) -> Result<Value<'a>, DecodeError> {
    let value = reader.read_f64("float64")?;
    let unit_index = reader.read_varint("float64.unit")? as usize;
    let unit = if unit_index == 0 {
        None
    } else {
        let idx = unit_index - 1;
        if idx >= dicts.units.len() {
            return Err(DecodeError::IndexOutOfBounds {
                dict: "units",
                index: unit_index,
                size: dicts.units.len() + 1,
            });
        }
        Some(dicts.units[idx])
    };
    Ok(Value::Float64 { value, unit })
}

fn decode_decimal<'a>(reader: &mut Reader<'a>, dicts: &WireDictionaries) -> Result<Value<'a>, DecodeError> {
    let exponent = reader.read_signed_varint("decimal.exponent")? as i32;
    let mantissa_type = reader.read_byte("decimal.mantissa_type")?;

    let mantissa = match mantissa_type {
        0x00 => {
            let v = reader.read_signed_varint("decimal.mantissa")?;
            DecimalMantissa::I64(v)
        }
        0x01 => {
            let len = reader.read_varint("decimal.mantissa_len")? as usize;
            let bytes = reader.read_bytes(len, "decimal.mantissa_bytes")?;

            // Validate minimal encoding
            if !bytes.is_empty() {
                let first = bytes[0];
                // Check for redundant sign extension
                if bytes.len() > 1 {
                    let second = bytes[1];
                    if (first == 0x00 && (second & 0x80) == 0)
                        || (first == 0xFF && (second & 0x80) != 0) {
                        return Err(DecodeError::DecimalMantissaNotMinimal);
                    }
                }
            }

            DecimalMantissa::Big(Cow::Borrowed(bytes))
        }
        _ => {
            return Err(DecodeError::MalformedEncoding {
                context: "invalid decimal mantissa type"
            });
        }
    };

    // Validate normalization
    match &mantissa {
        DecimalMantissa::I64(v) => {
            if *v == 0 {
                if exponent != 0 {
                    return Err(DecodeError::DecimalNotNormalized);
                }
            } else if *v % 10 == 0 {
                return Err(DecodeError::DecimalNotNormalized);
            }
        }
        DecimalMantissa::Big(_) => {
            // TODO: full normalization check for big decimals
        }
    }

    let unit_index = reader.read_varint("decimal.unit")? as usize;
    let unit = if unit_index == 0 {
        None
    } else {
        let idx = unit_index - 1;
        if idx >= dicts.units.len() {
            return Err(DecodeError::IndexOutOfBounds {
                dict: "units",
                index: unit_index,
                size: dicts.units.len() + 1,
            });
        }
        Some(dicts.units[idx])
    };

    Ok(Value::Decimal { exponent, mantissa, unit })
}

fn decode_text<'a>(reader: &mut Reader<'a>, dicts: &WireDictionaries) -> Result<Value<'a>, DecodeError> {
    let value = reader.read_str(MAX_STRING_LEN, "text")?;
    let lang_index = reader.read_varint("text.language")? as usize;

    let language = if lang_index == 0 {
        None
    } else {
        let idx = lang_index - 1;
        if idx >= dicts.languages.len() {
            return Err(DecodeError::IndexOutOfBounds {
                dict: "languages",
                index: lang_index,
                size: dicts.languages.len() + 1, // +1 for index 0
            });
        }
        Some(dicts.languages[idx])
    };

    Ok(Value::Text { value: Cow::Borrowed(value), language })
}

fn decode_bytes<'a>(reader: &mut Reader<'a>) -> Result<Value<'a>, DecodeError> {
    let len = reader.read_varint("bytes.len")? as usize;
    if len > MAX_BYTES_LEN {
        return Err(DecodeError::LengthExceedsLimit {
            field: "bytes",
            len,
            max: MAX_BYTES_LEN,
        });
    }
    let bytes = reader.read_bytes(len, "bytes")?;
    Ok(Value::Bytes(Cow::Borrowed(bytes)))
}

fn decode_timestamp<'a>(reader: &mut Reader<'a>) -> Result<Value<'a>, DecodeError> {
    let value = reader.read_signed_varint("timestamp")?;
    Ok(Value::Timestamp(value))
}

fn decode_date<'a>(reader: &mut Reader<'a>) -> Result<Value<'a>, DecodeError> {
    let value = reader.read_str(MAX_STRING_LEN, "date")?;
    // TODO: validate ISO 8601 format
    Ok(Value::Date(Cow::Borrowed(value)))
}

fn decode_point<'a>(reader: &mut Reader<'a>) -> Result<Value<'a>, DecodeError> {
    let lat = reader.read_f64("point.lat")?;
    let lon = reader.read_f64("point.lon")?;

    // Validate bounds
    if !(-90.0..=90.0).contains(&lat) {
        return Err(DecodeError::LatitudeOutOfRange { lat });
    }
    if !(-180.0..=180.0).contains(&lon) {
        return Err(DecodeError::LongitudeOutOfRange { lon });
    }

    Ok(Value::Point { lat, lon })
}

fn decode_embedding<'a>(reader: &mut Reader<'a>) -> Result<Value<'a>, DecodeError> {
    let sub_type_byte = reader.read_byte("embedding.sub_type")?;
    let sub_type = EmbeddingSubType::from_u8(sub_type_byte)
        .ok_or(DecodeError::InvalidEmbeddingSubType { sub_type: sub_type_byte })?;

    let dims = reader.read_varint("embedding.dims")? as usize;
    if dims > MAX_EMBEDDING_DIMS {
        return Err(DecodeError::LengthExceedsLimit {
            field: "embedding.dims",
            len: dims,
            max: MAX_EMBEDDING_DIMS,
        });
    }

    let expected_bytes = sub_type.bytes_for_dims(dims);
    if expected_bytes > MAX_EMBEDDING_BYTES {
        return Err(DecodeError::LengthExceedsLimit {
            field: "embedding.data",
            len: expected_bytes,
            max: MAX_EMBEDDING_BYTES,
        });
    }

    let data = reader.read_bytes(expected_bytes, "embedding.data")?;

    // Validate no NaN in float32 embeddings
    if sub_type == EmbeddingSubType::Float32 {
        for chunk in data.chunks_exact(4) {
            let f = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            if f.is_nan() {
                return Err(DecodeError::FloatIsNan);
            }
        }
    }

    // Validate binary embedding has zeros in unused bits
    if sub_type == EmbeddingSubType::Binary && dims % 8 != 0 {
        let last_byte = data[data.len() - 1];
        let unused_bits = 8 - (dims % 8);
        let mask = !((1u8 << (8 - unused_bits)) - 1);
        if last_byte & mask != 0 {
            return Err(DecodeError::MalformedEncoding {
                context: "binary embedding has non-zero unused bits",
            });
        }
    }

    Ok(Value::Embedding { sub_type, dims, data: Cow::Borrowed(data) })
}

/// Decodes a PropertyValue (property index + value + optional language).
pub fn decode_property_value<'a>(
    reader: &mut Reader<'a>,
    dicts: &WireDictionaries,
) -> Result<PropertyValue<'a>, DecodeError> {
    let prop_index = reader.read_varint("property")? as usize;
    if prop_index >= dicts.properties.len() {
        return Err(DecodeError::IndexOutOfBounds {
            dict: "properties",
            index: prop_index,
            size: dicts.properties.len(),
        });
    }

    let (property, data_type) = dicts.properties[prop_index];
    let value = decode_value(reader, data_type, dicts)?;

    Ok(PropertyValue { property, value })
}

// =============================================================================
// ENCODING
// =============================================================================

/// Encodes a Value to the writer.
pub fn encode_value(
    writer: &mut Writer,
    value: &Value<'_>,
    dict_builder: &mut DictionaryBuilder,
) -> Result<(), EncodeError> {
    match value {
        Value::Bool(v) => {
            writer.write_byte(if *v { 0x01 } else { 0x00 });
        }
        Value::Int64 { value, unit } => {
            writer.write_signed_varint(*value);
            let unit_index = dict_builder.add_unit(*unit);
            writer.write_varint(unit_index as u64);
        }
        Value::Float64 { value, unit } => {
            if value.is_nan() {
                return Err(EncodeError::FloatIsNan);
            }
            writer.write_f64(*value);
            let unit_index = dict_builder.add_unit(*unit);
            writer.write_varint(unit_index as u64);
        }
        Value::Decimal { exponent, mantissa, unit } => {
            encode_decimal(writer, *exponent, mantissa)?;
            let unit_index = dict_builder.add_unit(*unit);
            writer.write_varint(unit_index as u64);
        }
        Value::Text { value, language } => {
            writer.write_string(value);
            let lang_index = dict_builder.add_language(*language);
            writer.write_varint(lang_index as u64);
        }
        Value::Bytes(bytes) => {
            writer.write_bytes_prefixed(bytes);
        }
        Value::Timestamp(v) => {
            writer.write_signed_varint(*v);
        }
        Value::Date(s) => {
            writer.write_string(s);
        }
        Value::Point { lat, lon } => {
            if *lat < -90.0 || *lat > 90.0 {
                return Err(EncodeError::LatitudeOutOfRange { lat: *lat });
            }
            if *lon < -180.0 || *lon > 180.0 {
                return Err(EncodeError::LongitudeOutOfRange { lon: *lon });
            }
            if lat.is_nan() || lon.is_nan() {
                return Err(EncodeError::FloatIsNan);
            }
            writer.write_f64(*lat);
            writer.write_f64(*lon);
        }
        Value::Embedding { sub_type, dims, data } => {
            let expected = sub_type.bytes_for_dims(*dims);
            if data.len() != expected {
                return Err(EncodeError::EmbeddingDimensionMismatch {
                    sub_type: *sub_type as u8,
                    dims: *dims,
                    data_len: data.len(),
                });
            }
            // Check for NaN in float32
            if *sub_type == EmbeddingSubType::Float32 {
                for chunk in data.chunks_exact(4) {
                    let f = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    if f.is_nan() {
                        return Err(EncodeError::FloatIsNan);
                    }
                }
            }
            writer.write_byte(*sub_type as u8);
            writer.write_varint(*dims as u64);
            writer.write_bytes(data);
        }
    }
    Ok(())
}

fn encode_decimal(
    writer: &mut Writer,
    exponent: i32,
    mantissa: &DecimalMantissa<'_>,
) -> Result<(), EncodeError> {
    // Validate normalization
    match mantissa {
        DecimalMantissa::I64(v) => {
            if *v == 0 {
                if exponent != 0 {
                    return Err(EncodeError::DecimalNotNormalized);
                }
            } else if *v % 10 == 0 {
                return Err(EncodeError::DecimalNotNormalized);
            }
        }
        DecimalMantissa::Big(_) => {
            // TODO: full normalization check
        }
    }

    writer.write_signed_varint(exponent as i64);

    match mantissa {
        DecimalMantissa::I64(v) => {
            writer.write_byte(0x00);
            writer.write_signed_varint(*v);
        }
        DecimalMantissa::Big(bytes) => {
            writer.write_byte(0x01);
            writer.write_varint(bytes.len() as u64);
            writer.write_bytes(bytes);
        }
    }

    Ok(())
}

/// Encodes a PropertyValue (property index + value + optional language).
pub fn encode_property_value(
    writer: &mut Writer,
    pv: &PropertyValue<'_>,
    dict_builder: &mut DictionaryBuilder,
    data_type: DataType,
) -> Result<(), EncodeError> {
    let prop_index = dict_builder.add_property(pv.property, data_type);
    writer.write_varint(prop_index as u64);
    encode_value(writer, &pv.value, dict_builder)?;
    Ok(())
}

/// Validates a position string according to spec rules.
pub fn validate_position(pos: &str) -> Result<(), EncodeError> {
    if pos.len() > MAX_POSITION_LEN {
        return Err(EncodeError::PositionTooLong);
    }
    for c in pos.chars() {
        if !c.is_ascii_alphanumeric() {
            return Err(EncodeError::InvalidPositionChar);
        }
    }
    Ok(())
}

/// Decodes a position string with validation (zero-copy).
pub fn decode_position<'a>(reader: &mut Reader<'a>) -> Result<Cow<'a, str>, DecodeError> {
    let pos = reader.read_str(MAX_POSITION_LEN, "position")?;
    for c in pos.chars() {
        if !c.is_ascii_alphanumeric() {
            return Err(DecodeError::InvalidPositionChar { char: c });
        }
    }
    Ok(Cow::Borrowed(pos))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_roundtrip() {
        for v in [true, false] {
            let value = Value::Bool(v);
            let dicts = WireDictionaries::default();
            let mut dict_builder = DictionaryBuilder::new();

            let mut writer = Writer::new();
            encode_value(&mut writer, &value, &mut dict_builder).unwrap();

            let mut reader = Reader::new(writer.as_bytes());
            let decoded = decode_value(&mut reader, DataType::Bool, &dicts).unwrap();

            assert_eq!(value, decoded);
        }
    }

    #[test]
    fn test_int64_roundtrip() {
        for v in [0i64, 1, -1, i64::MAX, i64::MIN, 12345678] {
            let value = Value::Int64 { value: v, unit: None };
            let mut dict_builder = DictionaryBuilder::new();

            let mut writer = Writer::new();
            encode_value(&mut writer, &value, &mut dict_builder).unwrap();

            let dicts = dict_builder.build();
            let mut reader = Reader::new(writer.as_bytes());
            let decoded = decode_value(&mut reader, DataType::Int64, &dicts).unwrap();

            assert_eq!(value, decoded);
        }
    }

    #[test]
    fn test_float64_roundtrip() {
        for v in [0.0, 1.0, -1.0, f64::INFINITY, f64::NEG_INFINITY, 3.14159] {
            let value = Value::Float64 { value: v, unit: None };
            let mut dict_builder = DictionaryBuilder::new();

            let mut writer = Writer::new();
            encode_value(&mut writer, &value, &mut dict_builder).unwrap();

            let dicts = dict_builder.build();
            let mut reader = Reader::new(writer.as_bytes());
            let decoded = decode_value(&mut reader, DataType::Float64, &dicts).unwrap();

            assert_eq!(value, decoded);
        }
    }

    #[test]
    fn test_text_roundtrip() {
        let value = Value::Text {
            value: Cow::Owned("hello world".to_string()),
            language: None,
        };
        let mut dict_builder = DictionaryBuilder::new();

        let mut writer = Writer::new();
        encode_value(&mut writer, &value, &mut dict_builder).unwrap();

        // Build dicts for decoding
        let decode_dicts = dict_builder.build();

        let mut reader = Reader::new(writer.as_bytes());
        let decoded = decode_value(&mut reader, DataType::Text, &decode_dicts).unwrap();

        // Compare inner values since one is Owned and one is Borrowed
        match (&value, &decoded) {
            (Value::Text { value: v1, language: l1 }, Value::Text { value: v2, language: l2 }) => {
                assert_eq!(v1.as_ref(), v2.as_ref());
                assert_eq!(l1, l2);
            }
            _ => panic!("expected Text values"),
        }
    }

    #[test]
    fn test_point_roundtrip() {
        let value = Value::Point { lat: 37.7749, lon: -122.4194 };
        let dicts = WireDictionaries::default();
        let mut dict_builder = DictionaryBuilder::new();

        let mut writer = Writer::new();
        encode_value(&mut writer, &value, &mut dict_builder).unwrap();

        let mut reader = Reader::new(writer.as_bytes());
        let decoded = decode_value(&mut reader, DataType::Point, &dicts).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn test_point_validation() {
        // Latitude out of range
        let value = Value::Point { lat: 91.0, lon: 0.0 };
        let mut dict_builder = DictionaryBuilder::new();
        let mut writer = Writer::new();
        let result = encode_value(&mut writer, &value, &mut dict_builder);
        assert!(result.is_err());
    }

    #[test]
    fn test_embedding_roundtrip() {
        let value = Value::Embedding {
            sub_type: EmbeddingSubType::Float32,
            dims: 4,
            data: Cow::Owned(vec![0u8; 16]), // 4 dims * 4 bytes
        };
        let dicts = WireDictionaries::default();
        let mut dict_builder = DictionaryBuilder::new();

        let mut writer = Writer::new();
        encode_value(&mut writer, &value, &mut dict_builder).unwrap();

        let mut reader = Reader::new(writer.as_bytes());
        let decoded = decode_value(&mut reader, DataType::Embedding, &dicts).unwrap();

        // Compare inner values since one is Owned and one is Borrowed
        match (&value, &decoded) {
            (
                Value::Embedding { sub_type: s1, dims: d1, data: data1 },
                Value::Embedding { sub_type: s2, dims: d2, data: data2 },
            ) => {
                assert_eq!(s1, s2);
                assert_eq!(d1, d2);
                assert_eq!(data1.as_ref(), data2.as_ref());
            }
            _ => panic!("expected Embedding values"),
        }
    }

    #[test]
    fn test_decimal_normalized() {
        // Valid: 12.34 = 1234 * 10^-2
        let valid = Value::Decimal {
            exponent: -2,
            mantissa: DecimalMantissa::I64(1234),
            unit: None,
        };
        let mut dict_builder = DictionaryBuilder::new();
        let mut writer = Writer::new();
        assert!(encode_value(&mut writer, &valid, &mut dict_builder).is_ok());

        // Invalid: has trailing zeros
        let invalid = Value::Decimal {
            exponent: -2,
            mantissa: DecimalMantissa::I64(1230),
            unit: None,
        };
        let mut dict_builder = DictionaryBuilder::new();
        let mut writer = Writer::new();
        assert!(encode_value(&mut writer, &invalid, &mut dict_builder).is_err());
    }
}
