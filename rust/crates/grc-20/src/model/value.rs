//! Value types for GRC-20 properties.
//!
//! Values are typed attribute instances on entities and relations.

use crate::model::Id;

/// Data types for property values (spec Section 2.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DataType {
    Bool = 1,
    Int64 = 2,
    Float64 = 3,
    Decimal = 4,
    Text = 5,
    Bytes = 6,
    Timestamp = 7,
    Date = 8,
    Point = 9,
    Embedding = 10,
    Ref = 11,
}

impl DataType {
    /// Creates a DataType from its wire representation.
    pub fn from_u8(v: u8) -> Option<DataType> {
        match v {
            1 => Some(DataType::Bool),
            2 => Some(DataType::Int64),
            3 => Some(DataType::Float64),
            4 => Some(DataType::Decimal),
            5 => Some(DataType::Text),
            6 => Some(DataType::Bytes),
            7 => Some(DataType::Timestamp),
            8 => Some(DataType::Date),
            9 => Some(DataType::Point),
            10 => Some(DataType::Embedding),
            11 => Some(DataType::Ref),
            _ => None,
        }
    }
}

/// Embedding sub-types (spec Section 2.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EmbeddingSubType {
    /// 32-bit IEEE 754 float, little-endian (4 bytes per dim)
    Float32 = 0,
    /// Signed 8-bit integer (1 byte per dim)
    Int8 = 1,
    /// Bit-packed binary, LSB-first (1/8 byte per dim)
    Binary = 2,
}

impl EmbeddingSubType {
    /// Creates an EmbeddingSubType from its wire representation.
    pub fn from_u8(v: u8) -> Option<EmbeddingSubType> {
        match v {
            0 => Some(EmbeddingSubType::Float32),
            1 => Some(EmbeddingSubType::Int8),
            2 => Some(EmbeddingSubType::Binary),
            _ => None,
        }
    }

    /// Returns the number of bytes needed for the given number of dimensions.
    pub fn bytes_for_dims(self, dims: usize) -> usize {
        match self {
            EmbeddingSubType::Float32 => dims * 4,
            EmbeddingSubType::Int8 => dims,
            EmbeddingSubType::Binary => dims.div_ceil(8),
        }
    }
}

/// Decimal mantissa representation.
///
/// Most decimals fit in i64; larger values use big-endian two's complement bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecimalMantissa {
    /// Mantissa fits in signed 64-bit integer.
    I64(i64),
    /// Arbitrary precision: big-endian two's complement, minimal-length.
    Big(Vec<u8>),
}

impl DecimalMantissa {
    /// Returns whether this mantissa has trailing zeros (not normalized).
    pub fn has_trailing_zeros(&self) -> bool {
        match self {
            DecimalMantissa::I64(v) => *v != 0 && *v % 10 == 0,
            DecimalMantissa::Big(bytes) => {
                // For big mantissas, we'd need to convert to check
                // This is a simplification - full check would convert to decimal
                !bytes.is_empty() && bytes[bytes.len() - 1] == 0
            }
        }
    }

    /// Returns true if this is the zero mantissa.
    pub fn is_zero(&self) -> bool {
        match self {
            DecimalMantissa::I64(v) => *v == 0,
            DecimalMantissa::Big(bytes) => bytes.iter().all(|b| *b == 0),
        }
    }
}

/// A typed value that can be stored on an entity or relation.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Boolean value.
    Bool(bool),

    /// 64-bit signed integer.
    Int64(i64),

    /// 64-bit IEEE 754 float (NaN not allowed).
    Float64(f64),

    /// Arbitrary-precision decimal: value = mantissa * 10^exponent.
    Decimal {
        exponent: i32,
        mantissa: DecimalMantissa,
    },

    /// UTF-8 text with optional language.
    Text {
        value: String,
        /// Language entity ID, or None for default language.
        language: Option<Id>,
    },

    /// Opaque byte array.
    Bytes(Vec<u8>),

    /// Microseconds since Unix epoch.
    Timestamp(i64),

    /// ISO 8601 date string (variable precision).
    Date(String),

    /// WGS84 geographic coordinate.
    Point {
        /// Latitude in degrees (-90 to +90).
        lat: f64,
        /// Longitude in degrees (-180 to +180).
        lon: f64,
    },

    /// Dense vector for semantic similarity search.
    Embedding {
        sub_type: EmbeddingSubType,
        dims: usize,
        /// Raw bytes in the format specified by sub_type.
        data: Vec<u8>,
    },

    /// Non-traversable object reference.
    Ref(Id),
}

impl Value {
    /// Returns the data type of this value.
    pub fn data_type(&self) -> DataType {
        match self {
            Value::Bool(_) => DataType::Bool,
            Value::Int64(_) => DataType::Int64,
            Value::Float64(_) => DataType::Float64,
            Value::Decimal { .. } => DataType::Decimal,
            Value::Text { .. } => DataType::Text,
            Value::Bytes(_) => DataType::Bytes,
            Value::Timestamp(_) => DataType::Timestamp,
            Value::Date(_) => DataType::Date,
            Value::Point { .. } => DataType::Point,
            Value::Embedding { .. } => DataType::Embedding,
            Value::Ref(_) => DataType::Ref,
        }
    }

    /// Validates this value according to spec rules.
    ///
    /// Returns an error description if invalid, None if valid.
    pub fn validate(&self) -> Option<&'static str> {
        match self {
            Value::Float64(v) => {
                if v.is_nan() {
                    return Some("NaN is not allowed in Float64");
                }
            }
            Value::Decimal { exponent, mantissa } => {
                // Zero must be {0, 0}
                if mantissa.is_zero() && *exponent != 0 {
                    return Some("zero DECIMAL must have exponent 0");
                }
                // Non-zero must not have trailing zeros
                if !mantissa.is_zero() && mantissa.has_trailing_zeros() {
                    return Some("DECIMAL mantissa has trailing zeros (not normalized)");
                }
            }
            Value::Point { lat, lon } => {
                if *lat < -90.0 || *lat > 90.0 {
                    return Some("latitude out of range [-90, +90]");
                }
                if *lon < -180.0 || *lon > 180.0 {
                    return Some("longitude out of range [-180, +180]");
                }
                if lat.is_nan() || lon.is_nan() {
                    return Some("NaN is not allowed in Point coordinates");
                }
            }
            Value::Embedding {
                sub_type,
                dims,
                data,
            } => {
                let expected = sub_type.bytes_for_dims(*dims);
                if data.len() != expected {
                    return Some("embedding data length doesn't match dims");
                }
                // Check for NaN in float32 embeddings
                if *sub_type == EmbeddingSubType::Float32 {
                    for chunk in data.chunks_exact(4) {
                        let f = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                        if f.is_nan() {
                            return Some("NaN is not allowed in float32 embedding");
                        }
                    }
                }
            }
            _ => {}
        }
        None
    }
}

/// A property-value pair that can be attached to an object.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyValue {
    /// The property ID this value is for.
    pub property: Id,
    /// The value.
    pub value: Value,
}

/// A property definition in the schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    /// The property's unique identifier.
    pub id: Id,
    /// The data type for values of this property.
    pub data_type: DataType,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_bytes_for_dims() {
        assert_eq!(EmbeddingSubType::Float32.bytes_for_dims(10), 40);
        assert_eq!(EmbeddingSubType::Int8.bytes_for_dims(10), 10);
        assert_eq!(EmbeddingSubType::Binary.bytes_for_dims(10), 2);
        assert_eq!(EmbeddingSubType::Binary.bytes_for_dims(8), 1);
        assert_eq!(EmbeddingSubType::Binary.bytes_for_dims(9), 2);
    }

    #[test]
    fn test_value_validation_nan() {
        assert!(Value::Float64(f64::NAN).validate().is_some());
        assert!(Value::Float64(f64::INFINITY).validate().is_none());
        assert!(Value::Float64(-f64::INFINITY).validate().is_none());
        assert!(Value::Float64(42.0).validate().is_none());
    }

    #[test]
    fn test_value_validation_point() {
        assert!(Value::Point { lat: 91.0, lon: 0.0 }.validate().is_some());
        assert!(Value::Point { lat: -91.0, lon: 0.0 }.validate().is_some());
        assert!(Value::Point { lat: 0.0, lon: 181.0 }.validate().is_some());
        assert!(Value::Point { lat: 0.0, lon: -181.0 }.validate().is_some());
        assert!(Value::Point { lat: 90.0, lon: 180.0 }.validate().is_none());
        assert!(Value::Point {
            lat: -90.0,
            lon: -180.0
        }
        .validate()
        .is_none());
    }

    #[test]
    fn test_decimal_normalization() {
        // Zero must have exponent 0
        let zero_bad = Value::Decimal {
            exponent: 1,
            mantissa: DecimalMantissa::I64(0),
        };
        assert!(zero_bad.validate().is_some());

        // Non-zero with trailing zeros is invalid
        let trailing = Value::Decimal {
            exponent: 0,
            mantissa: DecimalMantissa::I64(1230),
        };
        assert!(trailing.validate().is_some());

        // Valid decimal
        let valid = Value::Decimal {
            exponent: -2,
            mantissa: DecimalMantissa::I64(1234),
        };
        assert!(valid.validate().is_none());
    }
}
