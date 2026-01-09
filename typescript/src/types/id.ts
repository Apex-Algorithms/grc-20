/**
 * A 16-byte UUID identifier.
 *
 * This is the universal identifier type for entities, relations, properties,
 * types, spaces, authors, and all other objects in GRC-20.
 */
export type Id = Uint8Array & { readonly __brand: unique symbol };

/**
 * Creates an Id from a Uint8Array.
 * @throws Error if the array is not exactly 16 bytes.
 */
export function createId(bytes: Uint8Array): Id {
  if (bytes.length !== 16) {
    throw new Error(`Id must be 16 bytes, got ${bytes.length}`);
  }
  return bytes as Id;
}

/**
 * Creates a copy of an Id.
 */
export function copyId(id: Id): Id {
  const copy = new Uint8Array(16);
  copy.set(id);
  return copy as Id;
}

/**
 * The nil/zero UUID.
 */
export const NIL_ID: Id = createId(new Uint8Array(16));

/**
 * Compares two Ids for equality.
 */
export function idsEqual(a: Id, b: Id): boolean {
  for (let i = 0; i < 16; i++) {
    if (a[i] !== b[i]) return false;
  }
  return true;
}

/**
 * Compares two Ids lexicographically.
 * Returns negative if a < b, 0 if a === b, positive if a > b.
 */
export function compareIds(a: Id, b: Id): number {
  for (let i = 0; i < 16; i++) {
    if (a[i] !== b[i]) {
      return a[i] - b[i];
    }
  }
  return 0;
}
