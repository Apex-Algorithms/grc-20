import type { Id } from "./id.js";
import type { Op } from "./op.js";
import type { DataType } from "./value.js";

/**
 * A batch of operations with metadata (spec Section 4.1).
 *
 * Edits are standalone patches. They contain no parent references;
 * ordering is provided by on-chain governance.
 */
export interface Edit {
  /** The edit's unique identifier. */
  id: Id;
  /** Optional human-readable name (may be empty string). */
  name: string;
  /** Author entity IDs. */
  authors: Id[];
  /** Creation timestamp in microseconds since Unix epoch (metadata only). */
  createdAt: bigint;
  /** Operations in this edit. */
  ops: Op[];
}

/**
 * Wire-format dictionaries for encoding/decoding.
 *
 * These dictionaries map between full IDs and compact indices within an edit.
 */
export interface WireDictionaries {
  /** Properties dictionary: (ID, DataType) pairs. */
  properties: Array<{ id: Id; dataType: DataType }>;
  /** Relation type IDs. */
  relationTypes: Id[];
  /** Language entity IDs for localized TEXT values. */
  languages: Id[];
  /** Unit entity IDs for numerical values. */
  units: Id[];
  /** Object IDs (entities and relations). */
  objects: Id[];
}

/**
 * Creates empty wire dictionaries.
 */
export function createWireDictionaries(): WireDictionaries {
  return {
    properties: [],
    relationTypes: [],
    languages: [],
    units: [],
    objects: [],
  };
}
