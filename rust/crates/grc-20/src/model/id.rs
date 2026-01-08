//! UUID-based identifiers for GRC-20.
//!
//! All identifiers in GRC-20 are RFC 4122 UUIDs stored as 16 raw bytes.

use sha2::{Digest, Sha256};

/// A 16-byte UUID identifier.
///
/// This is the universal identifier type for entities, relations, properties,
/// types, spaces, authors, and all other objects in GRC-20.
pub type Id = [u8; 16];

/// The zero/nil UUID.
pub const NIL_ID: Id = [0u8; 16];

/// Derives a UUIDv8 from input bytes using SHA-256.
///
/// This implements the `derived_uuid` function from spec Section 2.1:
/// ```text
/// hash = SHA-256(input_bytes)[0:16]
/// hash[6] = (hash[6] & 0x0F) | 0x80  // version 8
/// hash[8] = (hash[8] & 0x3F) | 0x80  // RFC 4122 variant
/// ```
pub fn derived_uuid(input: &[u8]) -> Id {
    let hash = Sha256::digest(input);
    let mut id = [0u8; 16];
    id.copy_from_slice(&hash[..16]);

    // Set version 8 (bits 4-7 of byte 6)
    id[6] = (id[6] & 0x0F) | 0x80;
    // Set RFC 4122 variant (bits 6-7 of byte 8)
    id[8] = (id[8] & 0x3F) | 0x80;

    id
}

/// Computes the value identity hash for a non-TEXT value.
///
/// ```text
/// value_id = SHA-256(property_id || canonical_payload)[0:16]
/// ```
pub fn value_id(property_id: &Id, canonical_payload: &[u8]) -> Id {
    let mut hasher = Sha256::new();
    hasher.update(property_id);
    hasher.update(canonical_payload);
    let hash = hasher.finalize();

    let mut id = [0u8; 16];
    id.copy_from_slice(&hash[..16]);
    id
}

/// Computes the value identity hash for a TEXT value with language.
///
/// ```text
/// value_id = SHA-256(property_id || canonical_payload || language_id)[0:16]
/// ```
///
/// If `language_id` is `None`, uses 16 zero bytes (default language).
pub fn text_value_id(property_id: &Id, text: &[u8], language_id: Option<&Id>) -> Id {
    let mut hasher = Sha256::new();
    hasher.update(property_id);
    hasher.update(text);
    hasher.update(language_id.unwrap_or(&NIL_ID));
    let hash = hasher.finalize();

    let mut id = [0u8; 16];
    id.copy_from_slice(&hash[..16]);
    id
}

/// Derives a unique-mode relation ID.
///
/// ```text
/// id = derived_uuid(from_id || to_id || type_id)
/// ```
pub fn unique_relation_id(from_id: &Id, to_id: &Id, type_id: &Id) -> Id {
    let mut input = [0u8; 48];
    input[0..16].copy_from_slice(from_id);
    input[16..32].copy_from_slice(to_id);
    input[32..48].copy_from_slice(type_id);
    derived_uuid(&input)
}

/// Formats a UUID as non-hyphenated lowercase hex (recommended display format).
pub fn format_id(id: &Id) -> String {
    let mut s = String::with_capacity(32);
    for byte in id {
        s.push_str(&format!("{:02x}", byte));
    }
    s
}

/// Parses a UUID from hex string (with or without hyphens).
pub fn parse_id(s: &str) -> Option<Id> {
    // Remove hyphens if present
    let hex: String = s.chars().filter(|c| *c != '-').collect();
    if hex.len() != 32 {
        return None;
    }

    let mut id = [0u8; 16];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let byte_str = std::str::from_utf8(chunk).ok()?;
        id[i] = u8::from_str_radix(byte_str, 16).ok()?;
    }
    Some(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derived_uuid_version_and_variant() {
        let id = derived_uuid(b"test");
        // Version should be 8 (0x80 in high nibble of byte 6)
        assert_eq!(id[6] & 0xF0, 0x80);
        // Variant should be RFC 4122 (0b10 in high 2 bits of byte 8)
        assert_eq!(id[8] & 0xC0, 0x80);
    }

    #[test]
    fn test_derived_uuid_deterministic() {
        let id1 = derived_uuid(b"hello world");
        let id2 = derived_uuid(b"hello world");
        assert_eq!(id1, id2);

        let id3 = derived_uuid(b"different");
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_format_parse_roundtrip() {
        let id = derived_uuid(b"test");
        let formatted = format_id(&id);
        let parsed = parse_id(&formatted).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_parse_with_hyphens() {
        let hex = "550e8400e29b41d4a716446655440000";
        let with_hyphens = "550e8400-e29b-41d4-a716-446655440000";

        let id1 = parse_id(hex).unwrap();
        let id2 = parse_id(with_hyphens).unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_unique_relation_id() {
        let from = [1u8; 16];
        let to = [2u8; 16];
        let type_id = [3u8; 16];

        let id1 = unique_relation_id(&from, &to, &type_id);
        let id2 = unique_relation_id(&from, &to, &type_id);
        assert_eq!(id1, id2);

        // Different inputs produce different IDs
        let id3 = unique_relation_id(&to, &from, &type_id);
        assert_ne!(id1, id3);
    }
}
