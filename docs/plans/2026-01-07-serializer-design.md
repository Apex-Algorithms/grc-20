# GRC-20 v2 Serializer Design

**Date:** 2026-01-07
**Status:** Draft
**Spec Version:** 0.9.0

## Problem Statement

Build a production-quality binary serializer for GRC-20 v2 edits that:
- Safely decodes untrusted input from the network (indexer use case)
- Encodes edits for publishing (client use case)
- Works across Rust, Wasm/JS, Python, and Go
- Resists malicious input (DoS, memory exhaustion, stack overflow)

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Primary use case | Both encode/decode, emphasis on decode security | Indexers process untrusted network data |
| Language bindings | All from start (Rust, Wasm, Python, Go) | Wide ecosystem adoption |
| Error handling | Fail-fast | Simpler, safer, no ambiguous partial states |
| Memory model | In-memory only | Edits bounded (~1-2MB typical), Wasm-friendly |
| Validation layers | Structural in codec, semantic separate | Codec is self-contained; semantic needs graph context |
| Compression | Transparent (auto-detect GRC2 vs GRC2Z) | Best UX, single entry point |

### Validation Scope

**Structural (in serializer):**
- Magic bytes, version, lengths
- Varint bounds (max 10 bytes)
- Dictionary index bounds
- UTF-8 validity
- Required fields present

**Semantic (separate module, requires context):**
- Value types match property `data_type`
- Entity lifecycle (no updates to DEAD entities)

**Explicitly NOT validated:**
- Relation targets exist (cross-space references are valid)
- Deterministic ID computation (creator's responsibility)

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        grc-20                               │
├─────────────────────────────────────────────────────────────┤
│  model/          Data structures (Edit, Op, Value, etc.)    │
│  codec/          Binary encode/decode with structural       │
│                  validation, compression auto-detect        │
│  validate/       Semantic validation (types, lifecycle)     │
│  error.rs        Rich error types with decode position      │
├─────────────────────────────────────────────────────────────┤
│                     Public API                              │
│  encode_edit(&Edit) -> Result<Vec<u8>, EncodeError>        │
│  decode_edit(&[u8]) -> Result<Edit, DecodeError>           │
│  validate(&Edit, &Context) -> Result<(), ValidationError>  │
└─────────────────────────────────────────────────────────────┘
           │                │                │
    ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐
    │   js/wasm   │  │   python/   │  │     go/     │
    │ wasm-bindgen│  │    PyO3     │  │  wazero     │
    └─────────────┘  └─────────────┘  └─────────────┘
```

**Key principle:** Rust core has `&[u8]` in, `Result<T, E>` out API. No traits or generics in public interface for FFI simplicity.

---

## Project Structure

```
grc-20/
├── README.md
├── grc-20-v2-spec.md             # spec at root, shared by all
│
├── rust/                         # Rust workspace
│   ├── Cargo.toml
│   ├── crates/
│   │   ├── grc-20/               # core library
│   │   │   └── src/
│   │   │       ├── lib.rs
│   │   │       ├── model/
│   │   │       │   ├── mod.rs
│   │   │       │   ├── id.rs
│   │   │       │   ├── value.rs
│   │   │       │   ├── op.rs
│   │   │       │   └── edit.rs
│   │   │       ├── codec/
│   │   │       │   ├── mod.rs
│   │   │       │   ├── primitives.rs
│   │   │       │   ├── decode.rs
│   │   │       │   ├── encode.rs
│   │   │       │   └── compress.rs
│   │   │       ├── validate/
│   │   │       │   ├── mod.rs
│   │   │       │   └── semantic.rs
│   │   │       ├── limits.rs
│   │   │       └── error.rs
│   │   │
│   │   └── grc-20-wasm/
│   │       └── src/lib.rs
│   │
│   ├── fuzz/
│   │   └── fuzz_targets/
│   │       └── decode.rs
│   │
│   └── tests/
│       └── fixtures/
│
├── python/
│   ├── pyproject.toml
│   ├── src/grc_20/
│   │   ├── __init__.py
│   │   └── _native.pyi
│   └── tests/
│
├── go/
│   ├── go.mod
│   ├── grc20.go
│   ├── grc20_test.go
│   └── wasm/
│       └── grc_20_bg.wasm
│
├── js/
│   ├── package.json
│   ├── src/index.ts
│   └── tests/
│
└── scripts/
    ├── build-all.sh
    └── test-all.sh
```

---

## Model Types

### Identifiers

```rust
pub type Id = [u8; 16];  // Raw UUID bytes, FFI-friendly
```

### Values (Section 2.5 of spec)

```rust
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Decimal { exponent: i32, mantissa: DecimalMantissa },
    Text(String),
    Bytes(Vec<u8>),
    Timestamp(i64),       // microseconds since epoch
    Date(String),         // ISO 8601
    Point { lat: f64, lon: f64 },
    Embedding { sub_type: EmbeddingType, data: Vec<u8> },
    Ref(Id),
}

pub enum DecimalMantissa {
    I64(i64),
    Big(Vec<u8>),         // arbitrary precision, big-endian
}

pub enum EmbeddingType { Float32, Int8, Binary }

pub struct PropertyValue {
    pub property: Id,
    pub value: Value,
    pub position: Option<String>,
}
```

### Operations (Section 3 of spec)

```rust
pub enum Op {
    CreateEntity(CreateEntity),
    UpdateEntity(UpdateEntity),
    DeleteEntity(DeleteEntity),
    CreateRelation(CreateRelation),
    UpdateRelation(UpdateRelation),
    DeleteRelation(DeleteRelation),
    CreateProperty(CreateProperty),
    CreateType(CreateType),
    Snapshot(Snapshot),
}

pub struct CreateEntity {
    pub id: Id,
    pub types: Vec<Id>,
    pub values: Vec<PropertyValue>,
}

pub struct UpdateEntity {
    pub id: Id,
    pub add_types: Vec<Id>,
    pub remove_types: Vec<Id>,
    pub set_values: Vec<PropertyValue>,
    pub unset_properties: Vec<Id>,
}

pub struct DeleteEntity {
    pub id: Id,
    pub reason: Option<String>,
}

pub struct CreateRelation {
    pub id: Id,
    pub relation_type: Id,
    pub from_entity: Id,
    pub to_entity: Id,
    pub from_space: Option<Id>,
    pub to_space: Option<Id>,
    pub position: Option<String>,
    pub values: Vec<PropertyValue>,
}

pub struct UpdateRelation {
    pub id: Id,
    pub position: Option<String>,
    pub set_values: Vec<PropertyValue>,
    pub unset_properties: Vec<Id>,
}

pub struct DeleteRelation {
    pub id: Id,
    pub reason: Option<String>,
}

pub struct CreateProperty {
    pub id: Id,
    pub name: String,
    pub data_type: DataType,
    pub unit: Option<Id>,
    pub language: Option<Id>,
    pub target_types: Vec<Id>,
}

pub struct CreateType {
    pub id: Id,
    pub name: String,
    pub properties: Vec<Id>,
}

pub struct Snapshot {
    pub entity_id: Id,
    pub mode: SnapshotMode,
    pub types: Vec<Id>,
    pub values: Vec<PropertyValue>,
}

pub enum SnapshotMode { Partial, Complete }

pub enum DataType {
    Bool, Int64, Float64, Decimal, Text, Bytes,
    Timestamp, Date, Point, Embedding, Relation,
}
```

### Edit (Section 4 of spec)

```rust
pub struct Edit {
    pub id: Id,
    pub name: Option<String>,
    pub authors: Vec<Id>,
    pub parent_ids: Vec<Id>,
    pub created_at: i64,
    pub ops: Vec<Op>,
}
```

**Note:** Schema dictionaries (`property_ids`, `type_ids`, etc.) are wire format only. The codec resolves indices to full IDs during decode and builds dictionaries during encode. The `Edit` struct stays clean.

---

## Security Limits

```rust
// Varint bounds
pub const MAX_VARINT_BYTES: usize = 10;

// String/bytes limits
pub const MAX_STRING_LEN: usize = 16 * 1024 * 1024;       // 16 MB
pub const MAX_BYTES_LEN: usize = 64 * 1024 * 1024;        // 64 MB
pub const MAX_REASON_LEN: usize = 4096;

// Embedding limits
pub const MAX_EMBEDDING_DIMS: usize = 65536;
pub const MAX_EMBEDDING_BYTES: usize = 4 * MAX_EMBEDDING_DIMS;

// Collection limits
pub const MAX_OPS_PER_EDIT: usize = 1_000_000;
pub const MAX_VALUES_PER_ENTITY: usize = 10_000;
pub const MAX_TYPES_PER_ENTITY: usize = 1_000;
pub const MAX_AUTHORS: usize = 1_000;
pub const MAX_PARENTS: usize = 1_000;

// Dictionary limits
pub const MAX_DICT_SIZE: usize = 1_000_000;

// Total edit size (after decompression)
pub const MAX_EDIT_SIZE: usize = 256 * 1024 * 1024;       // 256 MB
```

**Key principle:** Check limits *before* allocation, not after.

---

## Error Types

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum DecodeError {
    // Structural
    InvalidMagic { found: [u8; 4] },
    UnsupportedVersion { version: u8 },
    UnexpectedEof { context: &'static str },

    // Varint
    VarintTooLong,
    VarintOverflow,

    // Length limits
    LengthExceedsLimit { field: &'static str, len: usize, max: usize },

    // Dictionary bounds
    IndexOutOfBounds { dict: &'static str, index: usize, len: usize },

    // Data integrity
    InvalidUtf8 { field: &'static str },
    InvalidOpType { op_type: u8 },
    InvalidValueType { value_type: u8 },
    InvalidDataType { data_type: u8 },
    InvalidEmbeddingSubType { sub_type: u8 },
    InvalidSnapshotMode { mode: u8 },

    // Compression
    DecompressionFailed { source: String },
    UncompressedSizeMismatch { declared: usize, actual: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum EncodeError {
    StringTooLong { field: &'static str, len: usize, max: usize },
    TooManyOps { count: usize, max: usize },
    EmbeddingDimensionMismatch { sub_type: EmbeddingType, data_len: usize },
    CompressionFailed { source: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    TypeMismatch { property: Id, expected: DataType, found: ValueKind },
    EntityIsDead { entity: Id },
    PropertyNotFound { property: Id },
}
```

---

## Codec Implementation

### Primitive Readers

```rust
pub fn read_varint(r: &mut &[u8]) -> Result<u64, DecodeError> {
    let mut result: u64 = 0;
    let mut shift = 0;

    for _ in 0..MAX_VARINT_BYTES {
        let byte = read_byte(r)?;
        result |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            return Ok(result);
        }
        shift += 7;
    }
    Err(DecodeError::VarintTooLong)
}

pub fn read_byte(r: &mut &[u8]) -> Result<u8, DecodeError> {
    if r.is_empty() {
        return Err(DecodeError::UnexpectedEof { context: "byte" });
    }
    let byte = r[0];
    *r = &r[1..];
    Ok(byte)
}

pub fn read_bytes_exact(r: &mut &[u8], n: usize, ctx: &'static str) -> Result<&[u8], DecodeError> {
    if r.len() < n {
        return Err(DecodeError::UnexpectedEof { context: ctx });
    }
    let (head, tail) = r.split_at(n);
    *r = tail;
    Ok(head)
}
```

**Key pattern:** Use `&mut &[u8]` for zero-copy slicing and simple EOF checks.

### Top-Level Decode

```rust
pub fn decode_edit(input: &[u8]) -> Result<Edit, DecodeError> {
    if input.len() < 4 {
        return Err(DecodeError::UnexpectedEof { context: "magic" });
    }

    // Detect compression
    let data: Cow<[u8]> = if &input[0..4] == b"GRC2" {
        Cow::Borrowed(input)
    } else if input.len() >= 5 && &input[0..5] == b"GRC2Z" {
        let decompressed = decompress_zstd(&input[5..])?;
        if decompressed.len() > MAX_EDIT_SIZE {
            return Err(DecodeError::LengthExceedsLimit {
                field: "edit", len: decompressed.len(), max: MAX_EDIT_SIZE,
            });
        }
        Cow::Owned(decompressed)
    } else {
        let mut found = [0u8; 4];
        found.copy_from_slice(&input[0..4]);
        return Err(DecodeError::InvalidMagic { found });
    };

    let r = &mut data.as_ref();
    *r = &r[4..];  // skip magic

    let version = read_byte(r)?;
    if version != 1 {
        return Err(DecodeError::UnsupportedVersion { version });
    }

    // Header
    let edit_id = read_id(r)?;
    let name = read_optional_string(r, MAX_STRING_LEN, "name")?;
    let authors = read_id_vec(r, MAX_AUTHORS, "authors")?;
    let parent_ids = read_id_vec(r, MAX_PARENTS, "parent_ids")?;
    let created_at = read_signed_varint(r)?;

    // Dictionaries
    let dicts = WireDictionaries {
        property_ids: read_id_vec(r, MAX_DICT_SIZE, "property_ids")?,
        type_ids: read_id_vec(r, MAX_DICT_SIZE, "type_ids")?,
        relation_type_ids: read_id_vec(r, MAX_DICT_SIZE, "relation_type_ids")?,
        entity_ids: read_id_vec(r, MAX_DICT_SIZE, "entity_ids")?,
    };

    // Ops
    let op_count = read_varint(r)? as usize;
    if op_count > MAX_OPS_PER_EDIT {
        return Err(DecodeError::LengthExceedsLimit {
            field: "ops", len: op_count, max: MAX_OPS_PER_EDIT,
        });
    }

    let mut ops = Vec::with_capacity(op_count);
    for _ in 0..op_count {
        ops.push(decode_op(r, &dicts)?);
    }

    Ok(Edit { id: edit_id, name, authors, parent_ids, created_at, ops })
}
```

### Encoding

```rust
pub fn encode_edit(edit: &Edit) -> Result<Vec<u8>, EncodeError> {
    if edit.ops.len() > MAX_OPS_PER_EDIT {
        return Err(EncodeError::TooManyOps {
            count: edit.ops.len(), max: MAX_OPS_PER_EDIT
        });
    }

    let dicts = build_dictionaries(edit);
    let mut buf = Vec::with_capacity(estimate_size(edit));

    buf.extend_from_slice(b"GRC2");
    buf.push(1);  // version

    // Header + dictionaries + ops...

    Ok(buf)
}

pub fn encode_edit_compressed(edit: &Edit, level: i32) -> Result<Vec<u8>, EncodeError> {
    let uncompressed = encode_edit(edit)?;
    let compressed = zstd::encode_all(uncompressed.as_slice(), level)
        .map_err(|e| EncodeError::CompressionFailed { source: e.to_string() })?;

    let mut buf = Vec::with_capacity(5 + compressed.len());
    buf.extend_from_slice(b"GRC2Z");
    buf.extend(compressed);
    Ok(buf)
}
```

**Dictionary building:** Collect unique IDs into a `HashSet`, then convert to `Vec` for writing. Sorting by UUID bytes is recommended but not required per spec Section 4.3.

---

## FFI Bindings

### Wasm (wasm-bindgen)

```rust
#[wasm_bindgen]
pub fn decode(data: &[u8]) -> Result<WasmEdit, JsError> {
    let edit = decode_edit(data)
        .map_err(|e| JsError::new(&format!("{:?}", e)))?;
    Ok(WasmEdit { inner: edit })
}
```

### Python (PyO3)

```rust
#[pyfunction]
fn decode(data: &[u8]) -> PyResult<PyEdit> {
    decode_edit(data)
        .map(|e| PyEdit { inner: e })
        .map_err(|e| PyValueError::new_err(format!("{:?}", e)))
}
```

### Go (wazero + embedded wasm)

```go
//go:embed grc_20_bg.wasm
var wasmBytes []byte

func (r *Runtime) Decode(ctx context.Context, data []byte) (*Edit, error) {
    // Call into wasm module
}
```

**Design principle:** Opaque wrapper types hide Rust internals. Errors become language-native. Byte arrays are the universal interchange.

---

## Testing Strategy

### Layer 1: Unit tests (primitives)
- Varint roundtrip for edge cases (0, 127, 128, MAX)
- Signed varint for negative values
- Varint rejection for overlong encoding

### Layer 2: Value roundtrips
- All 11 value types with edge cases
- Empty strings, max-length strings
- NaN/Infinity floats
- BCE dates

### Layer 3: Full edit roundtrip
- Minimal edit
- Edit with all op types
- Large edit (1000+ entities)
- Compressed roundtrip

### Layer 4: Property-based (proptest)
```rust
proptest! {
    fn doesnt_crash_on_arbitrary_bytes(data: Vec<u8>) {
        let _ = decode_edit(&data);  // never panics
    }

    fn roundtrip_arbitrary_edit(edit in arb_edit()) {
        let encoded = encode_edit(&edit).unwrap();
        let decoded = decode_edit(&encoded).unwrap();
        prop_assert_eq!(edit, decoded);
    }
}
```

### Layer 5: Malformed input corpus
- Invalid magic bytes
- Truncated at every position
- Index out of bounds
- Overlong varints

### Layer 6: Fuzz testing (cargo-fuzz)
```rust
fuzz_target!(|data: &[u8]| {
    let _ = decode_edit(data);
});
```

---

## Dependencies

```toml
[dependencies]
zstd = "0.13"        # compression
thiserror = "1"      # error derive macros

[dev-dependencies]
proptest = "1"       # property-based testing
```

Note: No `indexmap` needed - dictionaries are built with `HashSet` then sorted by UUID bytes per spec Section 4.3.

---

## Open Questions

None at this time.

---

## Changelog

- 2026-01-07: Initial design draft
