# GRC-20 TypeScript/WASM Implementation Plan

## Overview

This document outlines the implementation plan for a TypeScript SDK that provides GRC-20 encoding/decoding capabilities, using WebAssembly for performance-critical operations.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    TypeScript SDK                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │  Builder API    │  │   Type Defs     │  │  Utilities  │ │
│  │  (EditBuilder,  │  │  (Edit, Op,     │  │  (formatId, │ │
│  │   EntityBuilder)│  │   Value, etc.)  │  │   parseId)  │ │
│  └────────┬────────┘  └────────┬────────┘  └──────┬──────┘ │
│           │                    │                   │        │
│           ▼                    ▼                   ▼        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                    WASM Bridge                        │  │
│  │  - JSON serialization boundary                        │  │
│  │  - Uint8Array for binary data                         │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    WASM Module (Rust)                       │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │  encode_edit    │  │  decode_edit    │  │  decompress │ │
│  │  (JSON → bytes) │  │  (bytes → JSON) │  │  (zstd)     │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Design Decisions

### 1. WASM for Encoding/Decoding

**Why WASM:**
- Varint encoding/decoding is complex and error-prone in JS
- Zstd decompression requires native code
- Binary manipulation is faster in WASM
- Consistent behavior with Rust implementation

**JSON Boundary:**
- TypeScript passes Edit as JSON string to WASM
- WASM returns decoded Edit as JSON string
- Binary data (encoded bytes) passed as `Uint8Array`

This is the pragmatic choice because:
- JSON serialization overhead (~10-20ms for large edits) is acceptable
- Avoids complex memory management across WASM boundary
- TypeScript gets native objects to work with

### 2. Pure TypeScript for Types and Builders

**Why TypeScript:**
- Type safety and IntelliSense
- Familiar API for JS developers
- No WASM overhead for object construction
- Tree-shakeable for smaller bundles

### 3. BigInt for 64-bit Integers

All `i64` and `u64` values use `bigint`:
- `created_at` timestamps
- `Int64` values
- `Timestamp` values

JSON serialization will use string representation for BigInt values.

## Module Structure

```
typescript/
├── src/
│   ├── index.ts           # Public API exports
│   ├── types/
│   │   ├── id.ts          # Id type (Uint8Array[16])
│   │   ├── value.ts       # Value union type
│   │   ├── op.ts          # Op union type
│   │   └── edit.ts        # Edit interface
│   ├── builder/
│   │   ├── edit.ts        # EditBuilder
│   │   ├── entity.ts      # EntityBuilder
│   │   ├── relation.ts    # RelationBuilder
│   │   └── update.ts      # UpdateEntityBuilder
│   ├── codec/
│   │   ├── wasm.ts        # WASM module loader (core + lazy compression)
│   │   ├── encode.ts      # encodeEdit, encodeEditCompressed
│   │   └── decode.ts      # decodeEdit, decompress
│   ├── util/
│   │   ├── id.ts          # formatId, parseId, derivedUuid
│   │   └── json.ts        # BigInt-safe JSON serialization
│   └── genesis/
│       └── properties.ts  # Well-known property IDs
├── wasm/
│   ├── core/              # Core WASM crate (~100KB)
│   │   ├── Cargo.toml
│   │   └── src/lib.rs     # encode, decode (no zstd)
│   └── compression/       # Compression WASM crate (~200KB, lazy loaded)
│       ├── Cargo.toml
│       └── src/lib.rs     # encode_compressed, decompress
├── package.json
├── tsconfig.json
└── README.md
```

## Type Definitions

### Id

```typescript
// 16-byte UUID as Uint8Array
export type Id = Uint8Array & { readonly __brand: unique symbol };

export function createId(bytes: Uint8Array): Id;
export function parseId(hex: string): Id;
export function formatId(id: Id): string;
export function randomId(): Id;
```

### Value

```typescript
export type Value =
  | { type: 'bool'; value: boolean }
  | { type: 'int64'; value: bigint; unit?: Id }
  | { type: 'float64'; value: number; unit?: Id }
  | { type: 'decimal'; exponent: number; mantissa: bigint; unit?: Id }
  | { type: 'text'; value: string; language?: Id }
  | { type: 'bytes'; value: Uint8Array }
  | { type: 'timestamp'; value: bigint }
  | { type: 'date'; value: string }
  | { type: 'point'; lat: number; lon: number }
  | { type: 'embedding'; subType: EmbeddingSubType; dims: number; data: Uint8Array };

export type EmbeddingSubType = 'float32' | 'int8' | 'binary';
```

### Op

```typescript
export type Op =
  | { type: 'createEntity'; id: Id; values: PropertyValue[] }
  | { type: 'updateEntity'; id: Id; set: PropertyValue[]; unset: UnsetProperty[] }
  | { type: 'deleteEntity'; id: Id }
  | { type: 'restoreEntity'; id: Id }
  | { type: 'createRelation'; idMode: RelationIdMode; relationType: Id; from: Id; to: Id; ... }
  | { type: 'updateRelation'; id: Id; position?: string }
  | { type: 'deleteRelation'; id: Id }
  | { type: 'restoreRelation'; id: Id }
  | { type: 'createProperty'; id: Id; dataType: DataType };

export interface PropertyValue {
  property: Id;
  value: Value;
}

export type RelationIdMode =
  | { type: 'unique' }
  | { type: 'many'; id: Id };
```

### Edit

```typescript
export interface Edit {
  id: Id;
  name: string;
  authors: Id[];
  createdAt: bigint;  // microseconds since epoch
  ops: Op[];
}
```

## Builder API

### EditBuilder

```typescript
class EditBuilder {
  constructor(id: Id);

  name(name: string): this;
  author(authorId: Id): this;
  authors(authorIds: Id[]): this;
  createdAt(timestamp: bigint): this;
  createdNow(): this;

  createProperty(id: Id, dataType: DataType): this;

  createEntity(id: Id, build: (b: EntityBuilder) => EntityBuilder): this;
  createEmptyEntity(id: Id): this;
  updateEntity(id: Id, build: (b: UpdateEntityBuilder) => UpdateEntityBuilder): this;
  deleteEntity(id: Id): this;
  restoreEntity(id: Id): this;

  createRelationUnique(from: Id, to: Id, relationType: Id): this;
  createRelationMany(id: Id, from: Id, to: Id, relationType: Id): this;
  createRelation(build: (b: RelationBuilder) => RelationBuilder): this;
  updateRelation(id: Id, position?: string): this;
  deleteRelation(id: Id): this;
  restoreRelation(id: Id): this;

  build(): Edit;
}
```

### EntityBuilder

```typescript
class EntityBuilder {
  text(property: Id, value: string, language?: Id): this;
  int64(property: Id, value: bigint, unit?: Id): this;
  float64(property: Id, value: number, unit?: Id): this;
  bool(property: Id, value: boolean): this;
  bytes(property: Id, value: Uint8Array): this;
  point(property: Id, lat: number, lon: number): this;
  date(property: Id, value: string): this;
  timestamp(property: Id, micros: bigint): this;
  decimal(property: Id, exponent: number, mantissa: bigint, unit?: Id): this;
  embedding(property: Id, subType: EmbeddingSubType, dims: number, data: Uint8Array): this;
  value(property: Id, value: Value): this;
}
```

## Codec API

### Encoding

```typescript
// Encode Edit to binary (uncompressed)
export async function encodeEdit(edit: Edit): Promise<Uint8Array>;

// Encode Edit to binary (compressed with zstd)
export async function encodeEditCompressed(edit: Edit, level?: number): Promise<Uint8Array>;

// Encode with options
export interface EncodeOptions {
  canonical?: boolean;  // Deterministic encoding for signing
}
export async function encodeEditWithOptions(edit: Edit, options: EncodeOptions): Promise<Uint8Array>;
```

### Decoding

```typescript
// Decode binary to Edit (handles both compressed and uncompressed)
export async function decodeEdit(bytes: Uint8Array): Promise<Edit>;

// Decompress only (for zero-copy workflows - less relevant in JS)
export async function decompress(bytes: Uint8Array): Promise<Uint8Array>;
```

## WASM Implementation

### Core WASM Crate (no zstd dependency, ~100KB)

```rust
// wasm/core/src/lib.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn encode(edit_json: &str, canonical: bool) -> Result<Vec<u8>, JsValue> {
    let edit: grc_20::Edit = serde_json::from_str(edit_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let options = if canonical {
        grc_20::EncodeOptions::canonical()
    } else {
        grc_20::EncodeOptions::default()
    };

    grc_20::encode_edit_with_options(&edit, options)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn decode(bytes: &[u8]) -> Result<String, JsValue> {
    // Only handles uncompressed data (GRC2 magic)
    // Returns error for compressed data - caller should use compression module
    if bytes.len() >= 5 && &bytes[0..5] == b"GRC2Z" {
        return Err(JsValue::from_str(
            "Compressed data detected. Use decompress() from compression module first."
        ));
    }

    let edit = grc_20::decode_edit(bytes)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    serde_json::to_string(&edit)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
```

```toml
# wasm/core/Cargo.toml
[package]
name = "grc-20-wasm-core"
version = "0.1.0"

[lib]
crate-type = ["cdylib"]

[dependencies]
grc-20 = { path = "../../rust/crates/grc-20", default-features = false }
wasm-bindgen = "0.2"
serde_json = "1.0"

[features]
default = []  # No zstd!
```

### Compression WASM Crate (with zstd, ~200KB, lazy loaded)

```rust
// wasm/compression/src/lib.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn encode_compressed(edit_json: &str, level: i32) -> Result<Vec<u8>, JsValue> {
    let edit: grc_20::Edit = serde_json::from_str(edit_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    grc_20::encode_edit_compressed(&edit, level)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn decompress(bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
    grc_20::decompress(bytes)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn decode_compressed(bytes: &[u8]) -> Result<String, JsValue> {
    // Handles both compressed and uncompressed
    let edit = grc_20::decode_edit(bytes)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    serde_json::to_string(&edit)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
```

```toml
# wasm/compression/Cargo.toml
[package]
name = "grc-20-wasm-compression"
version = "0.1.0"

[lib]
crate-type = ["cdylib"]

[dependencies]
grc-20 = { path = "../../rust/crates/grc-20" }  # Full features including zstd
wasm-bindgen = "0.2"
serde_json = "1.0"
```

### TypeScript WASM Bridge

```typescript
// src/codec/wasm.ts

// Core module (encode/decode without compression)
let coreModule: typeof import('../wasm/pkg-core') | null = null;

// Compression module (lazy loaded)
let compressionModule: typeof import('../wasm/pkg-compression') | null = null;

export async function loadWasm(): Promise<void> {
  if (!coreModule) {
    coreModule = await import('../wasm/pkg-core');
  }
}

export function getCore() {
  if (!coreModule) {
    throw new Error('WASM not loaded. Call loadWasm() first.');
  }
  return coreModule;
}

// Lazy load compression on first use
export async function getCompression() {
  if (!compressionModule) {
    compressionModule = await import('../wasm/pkg-compression');
  }
  return compressionModule;
}
```

```typescript
// src/codec/encode.ts

export async function encodeEdit(edit: Edit): Promise<Uint8Array> {
  const wasm = getCore();
  const json = editToJson(edit);
  return wasm.encode(json, false);
}

export async function encodeEditCompressed(edit: Edit, level = 3): Promise<Uint8Array> {
  // Lazy loads compression module on first call
  const wasm = await getCompression();
  const json = editToJson(edit);
  return wasm.encode_compressed(json, level);
}
```

## JSON Serialization

BigInt values need special handling since JSON doesn't support them natively.

```typescript
// src/util/json.ts

// Serialize Edit to JSON with BigInt support
export function editToJson(edit: Edit): string {
  return JSON.stringify(edit, (key, value) => {
    if (typeof value === 'bigint') {
      return { __bigint: value.toString() };
    }
    if (value instanceof Uint8Array) {
      return { __bytes: Array.from(value) };
    }
    return value;
  });
}

// Deserialize JSON to Edit with BigInt support
export function jsonToEdit(json: string): Edit {
  return JSON.parse(json, (key, value) => {
    if (value && typeof value === 'object') {
      if ('__bigint' in value) {
        return BigInt(value.__bigint);
      }
      if ('__bytes' in value) {
        return new Uint8Array(value.__bytes);
      }
    }
    return value;
  });
}
```

## Performance Considerations

### Expected Performance

Based on Rust benchmarks and typical WASM overhead:

| Operation | Rust | Expected JS/WASM |
|-----------|------|------------------|
| Encode (fast) | 510 MB/s | ~100-200 MB/s |
| Decode | 500 MB/s | ~100-200 MB/s |
| JSON serialization | N/A | ~50-100 MB/s |

The JSON boundary will be the bottleneck for very large edits, but acceptable for typical use cases (edits < 1MB).

### Optimization Opportunities

1. **Streaming decode**: For very large edits, decode ops incrementally
2. **Worker threads**: Offload encoding/decoding to Web Workers
3. **Shared memory**: Use SharedArrayBuffer for zero-copy (requires COOP/COEP headers)
4. **Lazy parsing**: Only parse ops when accessed

## Implementation Phases

### Phase 1: Rust Preparation
- [ ] Add serde derives to grc-20 crate types (`Edit`, `Op`, `Value`, etc.)
- [ ] Add feature flag for optional zstd (`features = ["compression"]`)
- [ ] Test JSON round-trip in Rust
- [ ] Verify serde handles `Cow<'a, str>` and `Id` correctly

### Phase 2: Core WASM Module
- [ ] Set up `wasm/core/` crate structure
- [ ] Implement `encode()` and `decode()` bindings
- [ ] Build with wasm-pack (`--target bundler`)
- [ ] Verify ~100KB bundle size without zstd

### Phase 3: TypeScript Foundation
- [ ] Set up TypeScript project (tsconfig, package.json, build scripts)
- [ ] Implement type definitions (Id, Value, Op, Edit)
- [ ] Implement JSON serialization with BigInt support
- [ ] Create WASM loader for core module
- [ ] Basic `encodeEdit()` and `decodeEdit()` working

### Phase 4: Compression Module
- [ ] Set up `wasm/compression/` crate structure
- [ ] Implement `encode_compressed()`, `decompress()`, `decode_compressed()`
- [ ] Implement lazy loading in TypeScript
- [ ] Verify compression module only loads on demand

### Phase 5: Builder API
- [ ] EditBuilder implementation
- [ ] EntityBuilder implementation
- [ ] UpdateEntityBuilder implementation
- [ ] RelationBuilder implementation
- [ ] Unit tests for all builders

### Phase 6: Utilities and Polish
- [ ] ID utilities (formatId, parseId, derivedUuid, randomId)
- [ ] Genesis property IDs
- [ ] Error types matching Rust errors
- [ ] Documentation and examples

### Phase 7: Testing and Release
- [ ] Integration tests with cities.json data
- [ ] Cross-platform testing (Node.js, Chrome, Firefox, Safari)
- [ ] Bundle size verification (core < 150KB, compression < 250KB)
- [ ] Performance benchmarks
- [ ] npm package publishing

## Usage Example

```typescript
import {
  EditBuilder,
  encodeEdit,
  encodeEditCompressed,
  decodeEdit,
  parseId,
  randomId,
  loadWasm
} from '@geo-web/grc-20';

// Initialize core WASM (call once at startup, ~100KB)
await loadWasm();

// Create an edit using the builder API
const edit = new EditBuilder(randomId())
  .name('Create Alice')
  .author(parseId('550e8400-e29b-41d4-a716-446655440000'))
  .createdNow()
  .createProperty(parseId('...'), 'text')
  .createEntity(randomId(), (e) => e
    .text(parseId('...'), 'Alice')
    .int64(parseId('...'), 30n)
  )
  .createRelationUnique(
    parseId('...'),  // from
    parseId('...'),  // to
    parseId('...')   // relation type
  )
  .build();

// Encode to binary (uncompressed)
const bytes = await encodeEdit(edit);
console.log(`Encoded to ${bytes.length} bytes`);

// Encode with compression (lazy loads compression module on first call, ~200KB)
const compressed = await encodeEditCompressed(edit, 3);
console.log(`Compressed to ${compressed.length} bytes`);

// Decode (handles both compressed and uncompressed)
const decoded = await decodeEdit(bytes);
console.log(`Decoded: ${decoded.name}`);
```

### Compression-Only Usage

For apps that only need to decode (e.g., read-only viewers):

```typescript
import { decodeEdit, loadWasm } from '@geo-web/grc-20';

await loadWasm();

// Decoding uncompressed data uses core module only (~100KB)
const edit = await decodeEdit(uncompressedBytes);

// Decoding compressed data auto-loads compression module
const edit2 = await decodeEdit(compressedBytes); // Lazy loads ~200KB
```

## Design Decisions (Resolved)

### 1. Serde for Rust Edit

**Decision:** Add serde derives directly to existing Rust types.

```rust
#[derive(Serialize, Deserialize)]
pub struct Edit<'a> {
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    // ...
}
```

- Single source of truth, no code duplication
- Zero-copy deserialization still possible with `#[serde(borrow)]`
- Serde is already a common dependency

### 2. WASM Bundle Size

**Decision:** Lazy load compression module.

```typescript
// Core module loads immediately (~100KB)
const bytes = await encodeEdit(edit);

// Compression module loads on first use (~200KB additional)
const compressed = await encodeEditCompressed(edit);
```

- Fast initial load for most use cases
- Compression overhead only paid when needed
- Most apps decode more than encode, many don't need compression

### 3. Browser vs Node.js Loading

**Decision:** Use wasm-pack with `--target bundler` as default.

```bash
wasm-pack build --target bundler  # Works with webpack/rollup/vite
```

- Well-tested, handles edge cases automatically
- Works with modern bundlers out of the box
- Can add separate `--target nodejs` build later if needed

### 4. Error Types

Define TypeScript error types that mirror Rust errors:

```typescript
export class GrcError extends Error {
  constructor(
    public code: ErrorCode,
    message: string
  ) {
    super(message);
  }
}

export type ErrorCode =
  | 'E001'  // Invalid magic/version
  | 'E002'  // Index out of bounds
  | 'E003'  // Invalid signature
  | 'E004'  // Invalid UTF-8
  | 'E005'; // Malformed encoding

export class DecodeError extends GrcError {}
export class EncodeError extends GrcError {}
```
