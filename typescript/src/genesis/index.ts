import { parseId } from "../util/id.js";
import type { Id } from "../types/id.js";

// Re-export for convenience
export { derivedUuidFromString as genesisId, languageId, languages } from "./language.js";

// =============================================================================
// CORE PROPERTIES (Section 7.1)
// =============================================================================

/**
 * Well-known property IDs from the Genesis Space.
 */
export const properties = {
  /** Name property - primary label (TEXT) */
  NAME: parseId("a126ca530c8e48d5b88882c734c38935")!,

  /** Description property - summary text (TEXT) */
  DESCRIPTION: parseId("9b1f76ff9711404c861e59dc3fa7d037")!,

  /** Cover property - cover image URL (TEXT) */
  COVER: parseId("34f535072e6b42c5a84443981a77cfa2")!,

  // Helper functions
  name(): Id {
    return this.NAME;
  },
  description(): Id {
    return this.DESCRIPTION;
  },
  cover(): Id {
    return this.COVER;
  },
};

// =============================================================================
// CORE TYPES (Section 7.2)
// =============================================================================

/**
 * Well-known type entity IDs from the Genesis Space.
 */
export const types = {
  /** Image type - image entity */
  IMAGE: parseId("f3f790c4c74e4d23a0a91e8ef84e30d9")!,

  // Helper functions
  image(): Id {
    return this.IMAGE;
  },
};

// =============================================================================
// CORE RELATION TYPES (Section 7.3)
// =============================================================================

/**
 * Well-known relation type IDs from the Genesis Space.
 */
export const relationTypes = {
  /** Types relation - type membership */
  TYPES: parseId("8f151ba4de204e3c9cb499ddf96f48f1")!,

  // Helper functions
  types(): Id {
    return this.TYPES;
  },
};
