//! Security limits for GRC-20 encoding/decoding.
//!
//! These limits protect against resource exhaustion attacks
//! when processing untrusted input.

/// Maximum bytes for a varint (LEB128 can overflow u64 at 10 bytes).
pub const MAX_VARINT_BYTES: usize = 10;

/// Maximum length for string fields (16 MB).
pub const MAX_STRING_LEN: usize = 16 * 1024 * 1024;

/// Maximum length for bytes fields (64 MB).
pub const MAX_BYTES_LEN: usize = 64 * 1024 * 1024;

/// Maximum embedding dimensions.
pub const MAX_EMBEDDING_DIMS: usize = 65536;

/// Maximum embedding data bytes (4 bytes * max dims for float32).
pub const MAX_EMBEDDING_BYTES: usize = 4 * MAX_EMBEDDING_DIMS;

/// Maximum operations per edit.
pub const MAX_OPS_PER_EDIT: usize = 1_000_000;

/// Maximum values per entity operation.
pub const MAX_VALUES_PER_ENTITY: usize = 10_000;

/// Maximum authors per edit.
pub const MAX_AUTHORS: usize = 1_000;

/// Maximum entries in any dictionary.
pub const MAX_DICT_SIZE: usize = 1_000_000;

/// Maximum total edit size after decompression (256 MB).
pub const MAX_EDIT_SIZE: usize = 256 * 1024 * 1024;

/// Maximum position string length (spec Section 2.6).
pub const MAX_POSITION_LEN: usize = 64;

/// Magic bytes for uncompressed edits.
pub const MAGIC_UNCOMPRESSED: &[u8; 4] = b"GRC2";

/// Magic bytes for zstd-compressed edits.
pub const MAGIC_COMPRESSED: &[u8; 5] = b"GRC2Z";

/// Current binary format version.
pub const FORMAT_VERSION: u8 = 1;
