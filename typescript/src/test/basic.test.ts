import { describe, it, expect } from "vitest";
import {
  EditBuilder,
  encodeEdit,
  decodeEdit,
  parseId,
  formatId,
  randomId,
  derivedUuid,
  derivedUuidFromString,
  uniqueRelationId,
  relationEntityId,
  properties,
  types,
  relationTypes,
  languages,
  DataType,
  idsEqual,
} from "../index.js";

describe("ID utilities", () => {
  it("formatId produces 32 hex chars", () => {
    const id = randomId();
    const hex = formatId(id);
    expect(hex.length).toBe(32);
    expect(/^[0-9a-f]{32}$/.test(hex)).toBe(true);
  });

  it("parseId roundtrips with formatId", () => {
    const id = randomId();
    const hex = formatId(id);
    const parsed = parseId(hex);
    expect(parsed).toBeDefined();
    expect(idsEqual(id, parsed!)).toBe(true);
  });

  it("parseId accepts hyphens", () => {
    const withHyphens = "550e8400-e29b-41d4-a716-446655440000";
    const withoutHyphens = "550e8400e29b41d4a716446655440000";

    const id1 = parseId(withHyphens);
    const id2 = parseId(withoutHyphens);

    expect(id1).toBeDefined();
    expect(id2).toBeDefined();
    expect(idsEqual(id1!, id2!)).toBe(true);
  });

  it("derivedUuid is deterministic", () => {
    const input = new TextEncoder().encode("test");
    const id1 = derivedUuid(input);
    const id2 = derivedUuid(input);
    expect(idsEqual(id1, id2)).toBe(true);
  });

  it("derivedUuid produces valid UUIDv8", () => {
    const id = derivedUuidFromString("test");
    // Version 8 in byte 6
    expect((id[6] & 0xf0)).toBe(0x80);
    // Variant in byte 8
    expect((id[8] & 0xc0)).toBe(0x80);
  });

  it("uniqueRelationId is deterministic", () => {
    const from = parseId("11111111111111111111111111111111")!;
    const to = parseId("22222222222222222222222222222222")!;
    const type = parseId("33333333333333333333333333333333")!;

    const id1 = uniqueRelationId(from, to, type);
    const id2 = uniqueRelationId(from, to, type);
    expect(idsEqual(id1, id2)).toBe(true);

    // Different from -> different id
    const id3 = uniqueRelationId(to, from, type);
    expect(idsEqual(id1, id3)).toBe(false);
  });

  it("relationEntityId differs from relation id", () => {
    const relationId = randomId();
    const entityId = relationEntityId(relationId);
    expect(idsEqual(relationId, entityId)).toBe(false);
  });
});

describe("Genesis IDs", () => {
  it("properties.NAME matches spec", () => {
    const id = properties.NAME;
    expect(formatId(id)).toBe("a126ca530c8e48d5b88882c734c38935");
    expect(idsEqual(id, properties.name())).toBe(true);
  });

  it("properties.DESCRIPTION matches spec", () => {
    const id = properties.DESCRIPTION;
    expect(formatId(id)).toBe("9b1f76ff9711404c861e59dc3fa7d037");
  });

  it("properties.COVER matches spec", () => {
    const id = properties.COVER;
    expect(formatId(id)).toBe("34f535072e6b42c5a84443981a77cfa2");
  });

  it("types.IMAGE matches spec", () => {
    const id = types.IMAGE;
    expect(formatId(id)).toBe("f3f790c4c74e4d23a0a91e8ef84e30d9");
  });

  it("relationTypes.TYPES matches spec", () => {
    const id = relationTypes.TYPES;
    expect(formatId(id)).toBe("8f151ba4de204e3c9cb499ddf96f48f1");
    expect(idsEqual(id, relationTypes.types())).toBe(true);
  });

  it("languages.english is deterministic", () => {
    const id1 = languages.english();
    const id2 = languages.fromCode("en");
    expect(idsEqual(id1, id2)).toBe(true);
  });
});

describe("EditBuilder", () => {
  it("creates a simple edit with entity", () => {
    const editId = randomId();
    const entityId = randomId();
    const authorId = randomId();

    const edit = new EditBuilder(editId)
      .setName("Test Edit")
      .addAuthor(authorId)
      .setCreatedAt(1234567890n)
      .createEntity(entityId, (e) =>
        e.text(properties.name(), "Alice", undefined)
         .int64(parseId("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")!, 42n, undefined)
      )
      .build();

    expect(idsEqual(edit.id, editId)).toBe(true);
    expect(edit.name).toBe("Test Edit");
    expect(edit.authors.length).toBe(1);
    expect(edit.createdAt).toBe(1234567890n);
    expect(edit.ops.length).toBe(1);
    expect(edit.ops[0].type).toBe("createEntity");
  });

  it("creates relations in unique and many mode", () => {
    const editId = randomId();
    const from = randomId();
    const to = randomId();
    const relId = randomId();

    const edit = new EditBuilder(editId)
      .createEmptyEntity(from)
      .createEmptyEntity(to)
      .createRelationUnique(from, to, relationTypes.types())
      .createRelationMany(relId, from, to, relationTypes.types())
      .build();

    expect(edit.ops.length).toBe(4);
    expect(edit.ops[2].type).toBe("createRelation");
    expect(edit.ops[3].type).toBe("createRelation");

    const rel1 = edit.ops[2];
    const rel2 = edit.ops[3];
    if (rel1.type === "createRelation" && rel2.type === "createRelation") {
      expect(rel1.idMode.type).toBe("unique");
      expect(rel2.idMode.type).toBe("many");
    }
  });

  it("creates update entity operations", () => {
    const editId = randomId();
    const entityId = randomId();
    const propId = randomId();

    const edit = new EditBuilder(editId)
      .updateEntity(entityId, (u) =>
        u.setText(propId, "New value", undefined)
         .unsetAll(properties.description())
      )
      .build();

    expect(edit.ops.length).toBe(1);
    expect(edit.ops[0].type).toBe("updateEntity");

    const op = edit.ops[0];
    if (op.type === "updateEntity") {
      expect(op.setProperties.length).toBe(1);
      expect(op.unsetProperties.length).toBe(1);
    }
  });
});

describe("Codec", () => {
  it("encodes and decodes a simple edit", () => {
    const editId = randomId();
    const entityId = randomId();

    const edit = new EditBuilder(editId)
      .setName("Test Edit")
      .setCreatedAt(1234567890000000n)
      .createEntity(entityId, (e) =>
        e.text(properties.name(), "Alice", undefined)
         .bool(parseId("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")!, true)
      )
      .build();

    const encoded = encodeEdit(edit);
    expect(encoded.length).toBeGreaterThan(0);

    // Check magic bytes
    expect(String.fromCharCode(...encoded.slice(0, 4))).toBe("GRC2");

    const decoded = decodeEdit(encoded);

    expect(idsEqual(decoded.id, edit.id)).toBe(true);
    expect(decoded.name).toBe(edit.name);
    expect(decoded.createdAt).toBe(edit.createdAt);
    expect(decoded.ops.length).toBe(edit.ops.length);
  });

  it("encodes and decodes all value types", () => {
    const editId = randomId();
    const entityId = randomId();

    const edit = new EditBuilder(editId)
      .setName("All Types Test")
      .createEntity(entityId, (e) =>
        e.bool(parseId("11111111111111111111111111111111")!, true)
         .int64(parseId("22222222222222222222222222222222")!, -42n, undefined)
         .float64(parseId("33333333333333333333333333333333")!, 3.14159, undefined)
         .text(parseId("44444444444444444444444444444444")!, "Hello World", undefined)
         .bytes(parseId("55555555555555555555555555555555")!, new Uint8Array([1, 2, 3, 4]))
         .timestamp(parseId("66666666666666666666666666666666")!, 1704067200000000n)
         .date(parseId("77777777777777777777777777777777")!, "2024-01-15")
         .point(parseId("88888888888888888888888888888888")!, 40.7128, -74.006)
      )
      .build();

    const encoded = encodeEdit(edit);
    const decoded = decodeEdit(encoded);

    expect(decoded.ops.length).toBe(1);
    const op = decoded.ops[0];
    expect(op.type).toBe("createEntity");

    if (op.type === "createEntity") {
      expect(op.values.length).toBe(8);

      // Check each value type
      const boolVal = op.values.find(v => v.value.type === "bool");
      expect(boolVal?.value).toEqual({ type: "bool", value: true });

      const intVal = op.values.find(v => v.value.type === "int64");
      expect(intVal?.value).toEqual({ type: "int64", value: -42n, unit: undefined });

      const floatVal = op.values.find(v => v.value.type === "float64");
      if (floatVal?.value.type === "float64") {
        expect(floatVal.value.value).toBeCloseTo(3.14159, 5);
      }

      const textVal = op.values.find(v => v.value.type === "text");
      expect(textVal?.value).toEqual({ type: "text", value: "Hello World", language: undefined });

      const pointVal = op.values.find(v => v.value.type === "point");
      if (pointVal?.value.type === "point") {
        expect(pointVal.value.lat).toBeCloseTo(40.7128, 4);
        expect(pointVal.value.lon).toBeCloseTo(-74.006, 3);
      }
    }
  });

  it("encodes and decodes relations", () => {
    const editId = randomId();
    const from = randomId();
    const to = randomId();

    const edit = new EditBuilder(editId)
      .createEmptyEntity(from)
      .createEmptyEntity(to)
      .createRelationUnique(from, to, relationTypes.types())
      .build();

    const encoded = encodeEdit(edit);
    const decoded = decodeEdit(encoded);

    expect(decoded.ops.length).toBe(3);
    const rel = decoded.ops[2];
    expect(rel.type).toBe("createRelation");

    if (rel.type === "createRelation") {
      expect(rel.idMode.type).toBe("unique");
      expect(idsEqual(rel.from, from)).toBe(true);
      expect(idsEqual(rel.to, to)).toBe(true);
    }
  });

  it("encodes and decodes update/delete/restore operations", () => {
    const editId = randomId();
    const entityId = randomId();
    const relationId = randomId();
    const propId = randomId();

    const edit = new EditBuilder(editId)
      .updateEntity(entityId, (u) =>
        u.setText(propId, "Updated", undefined)
      )
      .deleteEntity(entityId)
      .restoreEntity(entityId)
      .updateRelation(relationId, "abc")
      .deleteRelation(relationId)
      .restoreRelation(relationId)
      .build();

    const encoded = encodeEdit(edit);
    const decoded = decodeEdit(encoded);

    expect(decoded.ops.length).toBe(6);
    expect(decoded.ops[0].type).toBe("updateEntity");
    expect(decoded.ops[1].type).toBe("deleteEntity");
    expect(decoded.ops[2].type).toBe("restoreEntity");
    expect(decoded.ops[3].type).toBe("updateRelation");
    expect(decoded.ops[4].type).toBe("deleteRelation");
    expect(decoded.ops[5].type).toBe("restoreRelation");

    const updateRel = decoded.ops[3];
    if (updateRel.type === "updateRelation") {
      expect(updateRel.position).toBe("abc");
    }
  });

  it("encodes and decodes createProperty", () => {
    const editId = randomId();
    const propId = randomId();

    const edit = new EditBuilder(editId)
      .createProperty(propId, DataType.Text)
      .build();

    const encoded = encodeEdit(edit);
    const decoded = decodeEdit(encoded);

    expect(decoded.ops.length).toBe(1);
    expect(decoded.ops[0].type).toBe("createProperty");

    const op = decoded.ops[0];
    if (op.type === "createProperty") {
      expect(idsEqual(op.id, propId)).toBe(true);
      expect(op.dataType).toBe(DataType.Text);
    }
  });

  it("canonical encoding is deterministic", () => {
    const editId = parseId("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")!;
    const entityId = parseId("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")!;

    const edit = new EditBuilder(editId)
      .setName("Canonical Test")
      .setCreatedAt(1000000n)
      .createEntity(entityId, (e) =>
        e.text(properties.name(), "Test", undefined)
      )
      .build();

    const encoded1 = encodeEdit(edit, { canonical: true });
    const encoded2 = encodeEdit(edit, { canonical: true });

    expect(encoded1.length).toBe(encoded2.length);
    for (let i = 0; i < encoded1.length; i++) {
      expect(encoded1[i]).toBe(encoded2[i]);
    }
  });
});
