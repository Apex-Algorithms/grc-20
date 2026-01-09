# GRC-20

A binary property graph format for decentralized knowledge networks.

GRC-20 is designed for encoding, decoding, and synchronizing graph data across distributed systems with support for event-sourced architecture, efficient binary transmission, and cross-language interoperability.

## Features

- **Property Graph Model** — Entities, relations, and typed properties with 10+ data types
- **Event Sourced** — All state changes expressed as append-only operations
- **Binary Optimized** — Dictionary interning and zstd compression for minimal wire size
- **Deterministic** — Canonical encoding for content addressing and signatures
- **Cross-Platform** — Rust and TypeScript implementations with identical behavior

## Installation

### Rust

```toml
[dependencies]
grc-20 = { git = "https://github.com/geobrowser/grc-20" }
```

### TypeScript

```bash
npm install @geoprotocol/grc-20
```

## Quick Start

### Rust

```rust
use grc_20::{EditBuilder, DataType, encode_edit, decode_edit};

// Create an edit with entities and relations
let edit = EditBuilder::new(edit_id)
    .name("My Edit")
    .author(author_id)
    .create_property(name_prop, DataType::Text)
    .create_entity(entity_id, |e| e.text(name_prop, "Hello", None))
    .build();

// Encode to binary
let bytes = encode_edit(&edit)?;

// Decode back
let decoded = decode_edit(&bytes)?;
```

### TypeScript

```typescript
import { EditBuilder, encodeEdit, decodeEdit } from '@geoprotocol/grc-20';

// Create an edit
const edit = new EditBuilder(editId)
  .name('My Edit')
  .author(authorId)
  .createEntity(entityId, (e) => e.text(nameProp, 'Hello'))
  .build();

// Encode to binary
const bytes = encodeEdit(edit);

// Decode back
const decoded = decodeEdit(bytes);
```

## Data Types

| Type | Description |
|------|-------------|
| `BOOL` | Boolean value |
| `INT64` | 64-bit signed integer |
| `FLOAT64` | IEEE 754 double precision |
| `DECIMAL` | Arbitrary-precision decimal |
| `TEXT` | UTF-8 string with optional language |
| `BYTES` | Opaque byte array |
| `TIMESTAMP` | Microseconds since epoch |
| `DATE` | ISO 8601 date with variable precision |
| `POINT` | WGS84 coordinates (lat, lon) |
| `EMBEDDING` | Dense vectors for semantic search |

## Operations

| Operation | Description |
|-----------|-------------|
| `CreateEntity` | Create or upsert an entity with values |
| `UpdateEntity` | Modify entity values |
| `DeleteEntity` | Tombstone an entity |
| `RestoreEntity` | Restore a deleted entity |
| `CreateRelation` | Create a directed edge |
| `UpdateRelation` | Update relation position |
| `DeleteRelation` | Tombstone a relation |
| `RestoreRelation` | Restore a deleted relation |
| `CreateProperty` | Define a property in the schema |

## Binary Format

GRC-20 uses a custom binary format optimized for size and decode speed:

- **GRC2** — Uncompressed format with dictionary interning
- **GRC2Z** — zstd compressed format

Both formats support canonical encoding for deterministic content addressing.

## Benchmarks

Run the comparison benchmark:

```bash
cd rust/crates/grc-20-compare
cargo run --release
```

Example output comparing GRC-20 vs Protocol Buffers on 153k cities:

```
╔══════════════════════════════════════════════════════════════════════════════╗
║                     GRC-20 vs Proto Benchmark Comparison                     ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Dataset: 153728 cities | JSON size:   193.7 MB                            ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  SIZE                                                                        ║
║  ┌─────────────────┬─────────────────┬─────────────────┬───────────────────┐ ║
║  │                 │     GRC-20      │      Proto      │      Winner       │ ║
║  ├─────────────────┼─────────────────┼─────────────────┼───────────────────┤ ║
║  │ Uncompressed    │       73.5 MB   │      252.6 MB   │    GRC-20 3.4x    │ ║
║  │ Compressed      │       25.2 MB   │       34.3 MB   │    GRC-20 1.4x    │ ║
║  │ vs JSON         │         13.0%   │         17.7%   │                   │ ║
║  └─────────────────┴─────────────────┴─────────────────┴───────────────────┘ ║
╠──────────────────────────────────────────────────────────────────────────────╣
║  ENCODE TIME                                                                 ║
║  ┌─────────────────┬─────────────────┬─────────────────┬───────────────────┐ ║
║  │                 │     GRC-20      │      Proto      │      Winner       │ ║
║  ├─────────────────┼─────────────────┼─────────────────┼───────────────────┤ ║
║  │ Uncompressed    │      120.0 ms   │      180.0 ms   │    GRC-20 1.5x    │ ║
║  │ Compressed      │      320.0 ms   │      360.0 ms   │    GRC-20 1.1x    │ ║
║  └─────────────────┴─────────────────┴─────────────────┴───────────────────┘ ║
╠──────────────────────────────────────────────────────────────────────────────╣
║  DECODE TIME                                                                 ║
║  ┌─────────────────┬─────────────────┬─────────────────┬───────────────────┐ ║
║  │                 │     GRC-20      │      Proto      │      Winner       │ ║
║  ├─────────────────┼─────────────────┼─────────────────┼───────────────────┤ ║
║  │ Uncompressed    │      145.0 ms   │      710.0 ms   │    GRC-20 4.9x    │ ║
║  │ Compressed      │      295.0 ms   │      850.0 ms   │    GRC-20 2.9x    │ ║
║  └─────────────────┴─────────────────┴─────────────────┴───────────────────┘ ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

## Project Structure

```
grc-20/
├── spec.md                    # GRC-20 v2 specification
├── docs/                      # Design documentation
│   ├── requirements.md        # P0-P3 requirements
│   └── design-faq.md          # Design rationale
├── rust/                      # Rust implementation
│   └── crates/
│       ├── grc-20/            # Core library
│       ├── grc-20-bench/      # Benchmarks
│       ├── grc-20-compare/    # Format comparison tool
│       └── grc-20-proto-bench/# Protobuf baseline
├── typescript/                # TypeScript implementation
│   └── src/
│       ├── builder/           # EditBuilder API
│       ├── codec/             # Encoder/decoder
│       ├── types/             # Type definitions
│       ├── genesis/           # Well-known IDs
│       └── util/              # Utilities
└── data/                      # Sample datasets (compressed)
```

## Building

### Rust

```bash
cd rust
cargo build --release
cargo test
```

### TypeScript

```bash
cd typescript
npm install
npm run build
npm test
```

## Documentation

- [Specification](spec.md) — Complete binary format specification
- [Requirements](docs/requirements.md) — Design requirements and priorities
- [Design FAQ](docs/design-faq.md) — Rationale for design decisions

## Why GRC-20?

### Why not Protocol Buffers?

1. **Determinism** — GRC-20 supports canonical encoding for reproducible content hashes
2. **Size** — Dictionary interning saves ~12 bytes per UUID reference
3. **Simplicity** — ~200 lines to implement vs 500-1000 for Protobuf

### Why event sourcing?

- No pre-write state validation needed
- O(1) append vs O(log N) disk reads
- Supports offline-first workflows
- Enables CRDT-style convergence

### Why a custom binary format?

- Optimized specifically for property graph operations
- Native support for multi-value properties and language variants
- Built-in compression with zstd
- Designed for content addressing from the start

## License

MIT OR Apache-2.0
