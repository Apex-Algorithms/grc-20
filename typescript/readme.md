# @geoprotocol/grc-20

TypeScript library for encoding and decoding GRC-20 binary property graph data.

## Installation

```bash
npm install @geoprotocol/grc-20
```

## Quick Start

```typescript
import {
  EditBuilder,
  encodeEdit,
  decodeEdit,
  randomId,
  properties,
} from "@geoprotocol/grc-20";

// Create an edit with an entity
const edit = new EditBuilder(randomId())
  .setName("Create Alice")
  .addAuthor(randomId())
  .setCreatedNow()
  .createEntity(randomId(), (e) =>
    e
      .text(properties.name(), "Alice", undefined)
      .text(properties.description(), "A person", undefined)
  )
  .build();

// Encode to binary
const bytes = encodeEdit(edit);

// Decode back
const decoded = decodeEdit(bytes);
```

## Features

- **Type-safe API** - Full TypeScript definitions
- **Builder pattern** - Fluent API for constructing edits
- **Binary codec** - Pure TypeScript encoder/decoder (no WASM)
- **Tree-shakeable** - Separate entry points for minimal bundles
- **Cross-platform** - Works in Node.js and browsers

## Bundle Sizes

| Entry Point | Gzipped |
|-------------|---------|
| Full library | ~8.7 KB |
| Types only | ~1.4 KB |
| Builder only | ~1.2 KB |
| Codec only | ~4.8 KB |
| Genesis IDs | ~1.9 KB |
| Utilities | ~1.6 KB |

### Lazy Loading

For optimal initial load, import the codec separately:

```typescript
// Initial load (~4.6 KB gzipped)
import { EditBuilder, randomId, properties } from "@geoprotocol/grc-20/builder";
import { properties } from "@geoprotocol/grc-20/genesis";

// Lazy load codec when needed (~4.8 KB gzipped)
const { encodeEdit } = await import("@geoprotocol/grc-20/codec");
```

## API Reference

### Types

```typescript
import {
  Id,              // 16-byte UUID (Uint8Array branded type)
  Edit,            // Batch of operations with metadata
  Op,              // Union of all operation types
  Value,           // Union of all value types
  DataType,        // Enum: Bool, Int64, Float64, Text, etc.
  PropertyValue,   // Property ID + Value pair
} from "@geoprotocol/grc-20";
```

### Builders

```typescript
import {
  EditBuilder,          // Build Edit objects
  EntityBuilder,        // Build entity values
  UpdateEntityBuilder,  // Build update operations
  RelationBuilder,      // Build relation operations
} from "@geoprotocol/grc-20";
```

#### EditBuilder

```typescript
const edit = new EditBuilder(editId)
  .setName("My Edit")
  .addAuthor(authorId)
  .setCreatedAt(BigInt(Date.now()) * 1000n)  // microseconds
  .createEntity(entityId, e => e
    .text(propId, "value", languageId)
    .int64(propId, 42n, unitId)
    .float64(propId, 3.14, undefined)
    .bool(propId, true)
    .bytes(propId, new Uint8Array([1, 2, 3]))
    .point(propId, 40.7128, -74.006)
    .date(propId, "2024-01-15")
    .timestamp(propId, 1704067200000000n)
  )
  .updateEntity(entityId, u => u
    .setText(propId, "new value", undefined)
    .unsetAll(propId)
  )
  .deleteEntity(entityId)
  .restoreEntity(entityId)
  .createRelationUnique(fromId, toId, relationTypeId)
  .createRelationMany(relationId, fromId, toId, relationTypeId)
  .deleteRelation(relationId)
  .createProperty(propId, DataType.Text)
  .build();
```

### Codec

```typescript
import { encodeEdit, decodeEdit } from "@geoprotocol/grc-20";

// Encode
const bytes = encodeEdit(edit);
const bytesCanonical = encodeEdit(edit, { canonical: true });

// Decode
const edit = decodeEdit(bytes);
```

### ID Utilities

```typescript
import {
  randomId,           // Generate random UUIDv4
  parseId,            // Parse hex string to Id
  formatId,           // Format Id as hex string
  derivedUuid,        // Derive UUIDv8 from bytes (SHA-256)
  derivedUuidFromString,
  uniqueRelationId,   // Derive relation ID from endpoints
  relationEntityId,   // Derive entity ID from relation ID
  idsEqual,           // Compare two Ids
  NIL_ID,             // Zero UUID
} from "@geoprotocol/grc-20";
```

### Genesis IDs

Well-known IDs from the Genesis Space:

```typescript
import { properties, types, relationTypes, languages } from "@geoprotocol/grc-20";

// Properties
properties.name()        // a126ca530c8e48d5b88882c734c38935 - Name (TEXT)
properties.description() // 9b1f76ff9711404c861e59dc3fa7d037 - Description (TEXT)
properties.cover()       // 34f535072e6b42c5a84443981a77cfa2 - Cover image URL (TEXT)

// Types
types.image()            // f3f790c4c74e4d23a0a91e8ef84e30d9 - Image entity

// Relation Types
relationTypes.types()    // 8f151ba4de204e3c9cb499ddf96f48f1 - Type membership

// Languages (derived from BCP 47 codes)
languages.english()      // or languages.fromCode("en")
languages.spanish()
languages.french()
// ... etc
```

## Entry Points

For tree-shaking, use specific entry points:

```typescript
import { ... } from "@geoprotocol/grc-20";          // Full library
import { ... } from "@geoprotocol/grc-20/types";    // Types only
import { ... } from "@geoprotocol/grc-20/builder";  // Builders only
import { ... } from "@geoprotocol/grc-20/codec";    // Codec only
import { ... } from "@geoprotocol/grc-20/genesis";  // Genesis IDs only
import { ... } from "@geoprotocol/grc-20/util";     // Utilities only
```

## Development

```bash
# Install dependencies
npm install

# Build
npm run build

# Test (Node.js)
npm test

# Test (Browser via Playwright)
npm run test:browser

# Test both
npm run test:all

# Analyze bundle sizes
npm run bundle:analyze

# Run browser demo
npm run demo
# Then open http://localhost:3000/examples/browser-demo.html
```

## License

MIT
