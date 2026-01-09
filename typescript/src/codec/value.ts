import type { Id } from "../types/id.js";
import type { DecimalMantissa, PropertyValue, Value } from "../types/value.js";
import { DataType, EmbeddingSubType, embeddingBytesForDims } from "../types/value.js";
import { DecodeError, Reader, Writer } from "./primitives.js";

/**
 * Dictionary builder for tracking property/language/unit indices.
 */
export interface DictionaryIndices {
  getPropertyIndex(id: Id): number;
  getLanguageIndex(id: Id | undefined): number;
  getUnitIndex(id: Id | undefined): number;
  getDataType(propertyId: Id): DataType;
}

/**
 * Dictionary lookups for decoding.
 */
export interface DictionaryLookups {
  getProperty(index: number): { id: Id; dataType: DataType };
  getLanguage(index: number): Id | undefined;
  getUnit(index: number): Id | undefined;
}

/**
 * Encodes a value payload (without property index).
 */
export function encodeValuePayload(writer: Writer, value: Value): void {
  switch (value.type) {
    case "bool":
      writer.writeByte(value.value ? 0x01 : 0x00);
      break;

    case "int64":
      writer.writeSignedVarint(value.value);
      break;

    case "float64":
      if (Number.isNaN(value.value)) {
        throw new Error("NaN is not allowed in Float64");
      }
      writer.writeFloat64(value.value);
      break;

    case "decimal":
      encodeDecimal(writer, value.exponent, value.mantissa);
      break;

    case "text":
      writer.writeString(value.value);
      break;

    case "bytes":
      writer.writeLengthPrefixedBytes(value.value);
      break;

    case "timestamp":
      writer.writeSignedVarint(value.value);
      break;

    case "date":
      writer.writeString(value.value);
      break;

    case "point":
      if (Number.isNaN(value.lat) || Number.isNaN(value.lon)) {
        throw new Error("NaN is not allowed in Point coordinates");
      }
      if (value.lat < -90 || value.lat > 90) {
        throw new Error("latitude out of range [-90, +90]");
      }
      if (value.lon < -180 || value.lon > 180) {
        throw new Error("longitude out of range [-180, +180]");
      }
      writer.writeFloat64(value.lat);
      writer.writeFloat64(value.lon);
      break;

    case "embedding": {
      const expected = embeddingBytesForDims(value.subType, value.dims);
      if (value.data.length !== expected) {
        throw new Error(`embedding data length ${value.data.length} doesn't match expected ${expected}`);
      }
      writer.writeByte(value.subType);
      writer.writeVarintNumber(value.dims);
      writer.writeBytes(value.data);
      break;
    }
  }
}

/**
 * Encodes a decimal value.
 */
function encodeDecimal(writer: Writer, exponent: number, mantissa: DecimalMantissa): void {
  writer.writeSignedVarint(BigInt(exponent));

  if (mantissa.type === "i64") {
    writer.writeByte(0x00); // mantissa_type = varint
    writer.writeSignedVarint(mantissa.value);
  } else {
    writer.writeByte(0x01); // mantissa_type = bytes
    writer.writeLengthPrefixedBytes(mantissa.bytes);
  }
}

/**
 * Encodes a property value (with property index, language, unit).
 */
export function encodePropertyValue(
  writer: Writer,
  pv: PropertyValue,
  dicts: DictionaryIndices
): void {
  // Write property index
  const propIndex = dicts.getPropertyIndex(pv.property);
  writer.writeVarintNumber(propIndex);

  // Write payload
  encodeValuePayload(writer, pv.value);

  // Write language index for TEXT
  if (pv.value.type === "text") {
    const langIndex = dicts.getLanguageIndex(pv.value.language);
    writer.writeVarintNumber(langIndex);
  }

  // Write unit index for numerical types
  if (pv.value.type === "int64" || pv.value.type === "float64" || pv.value.type === "decimal") {
    const unitIndex = dicts.getUnitIndex(pv.value.unit);
    writer.writeVarintNumber(unitIndex);
  }
}

/**
 * Decodes a value payload based on data type.
 */
export function decodeValuePayload(reader: Reader, dataType: DataType): Value {
  switch (dataType) {
    case DataType.Bool: {
      const byte = reader.readByte();
      if (byte !== 0x00 && byte !== 0x01) {
        throw new DecodeError("E005", `invalid bool value: ${byte}`);
      }
      return { type: "bool", value: byte === 0x01 };
    }

    case DataType.Int64: {
      const value = reader.readSignedVarint();
      return { type: "int64", value };
    }

    case DataType.Float64: {
      const value = reader.readFloat64();
      if (Number.isNaN(value)) {
        throw new DecodeError("E005", "float value is NaN");
      }
      return { type: "float64", value };
    }

    case DataType.Decimal: {
      const exponent = Number(reader.readSignedVarint());
      const mantissaType = reader.readByte();
      let mantissa: DecimalMantissa;
      if (mantissaType === 0x00) {
        mantissa = { type: "i64", value: reader.readSignedVarint() };
      } else if (mantissaType === 0x01) {
        mantissa = { type: "big", bytes: reader.readLengthPrefixedBytes() };
      } else {
        throw new DecodeError("E005", `invalid decimal mantissa type: ${mantissaType}`);
      }
      return { type: "decimal", exponent, mantissa };
    }

    case DataType.Text: {
      const value = reader.readString();
      return { type: "text", value };
    }

    case DataType.Bytes: {
      const value = reader.readLengthPrefixedBytes();
      return { type: "bytes", value };
    }

    case DataType.Timestamp: {
      const value = reader.readSignedVarint();
      return { type: "timestamp", value };
    }

    case DataType.Date: {
      const value = reader.readString();
      return { type: "date", value };
    }

    case DataType.Point: {
      const lat = reader.readFloat64();
      const lon = reader.readFloat64();
      if (Number.isNaN(lat) || Number.isNaN(lon)) {
        throw new DecodeError("E005", "NaN is not allowed in Point coordinates");
      }
      if (lat < -90 || lat > 90) {
        throw new DecodeError("E005", `POINT latitude ${lat} out of range [-90, +90]`);
      }
      if (lon < -180 || lon > 180) {
        throw new DecodeError("E005", `POINT longitude ${lon} out of range [-180, +180]`);
      }
      return { type: "point", lat, lon };
    }

    case DataType.Embedding: {
      const subTypeByte = reader.readByte();
      if (subTypeByte > 2) {
        throw new DecodeError("E005", `invalid embedding sub-type: ${subTypeByte}`);
      }
      const subType = subTypeByte as EmbeddingSubType;
      const dims = reader.readVarintNumber();
      const expectedBytes = embeddingBytesForDims(subType, dims);
      const data = new Uint8Array(reader.readBytes(expectedBytes));
      return { type: "embedding", subType, dims, data };
    }

    default:
      throw new DecodeError("E005", `invalid data type: ${dataType}`);
  }
}

/**
 * Decodes a property value (with property index, language, unit).
 */
export function decodePropertyValue(
  reader: Reader,
  dicts: DictionaryLookups
): PropertyValue {
  // Read property index
  const propIndex = reader.readVarintNumber();
  const prop = dicts.getProperty(propIndex);

  // Read payload
  let value = decodeValuePayload(reader, prop.dataType);

  // Read language index for TEXT
  if (prop.dataType === DataType.Text) {
    const langIndex = reader.readVarintNumber();
    const language = dicts.getLanguage(langIndex);
    value = { ...value, language } as Value;
  }

  // Read unit index for numerical types
  if (
    prop.dataType === DataType.Int64 ||
    prop.dataType === DataType.Float64 ||
    prop.dataType === DataType.Decimal
  ) {
    const unitIndex = reader.readVarintNumber();
    const unit = dicts.getUnit(unitIndex);
    value = { ...value, unit } as Value;
  }

  return { property: prop.id, value };
}
