import { derivedUuidFromString, type Id } from "../util/id.js";

/**
 * Derives a genesis ID from a name.
 *
 * ```
 * id = derived_uuid("grc20:genesis:" + name)
 * ```
 */
export function genesisId(name: string): Id {
  return derivedUuidFromString(`grc20:genesis:${name}`);
}

/**
 * Derives a language ID from a BCP 47 language code.
 *
 * ```
 * id = derived_uuid("grc20:genesis:language:" + code)
 * ```
 */
export function languageId(code: string): Id {
  return derivedUuidFromString(`grc20:genesis:language:${code.toLowerCase()}`);
}

// =============================================================================
// CORE PROPERTIES (Section 7.1)
// =============================================================================

/**
 * Well-known property IDs from the Genesis Space.
 */
export const properties = {
  /** Name property - primary label (TEXT) */
  get NAME(): Id {
    return genesisId("Name");
  },

  /** Description property - summary text (TEXT) */
  get DESCRIPTION(): Id {
    return genesisId("Description");
  },

  /** Avatar property - image URL (TEXT) */
  get AVATAR(): Id {
    return genesisId("Avatar");
  },

  /** URL property - external link (TEXT) */
  get URL(): Id {
    return genesisId("URL");
  },

  /** Created property - creation time (TIMESTAMP) */
  get CREATED(): Id {
    return genesisId("Created");
  },

  /** Modified property - last modification (TIMESTAMP) */
  get MODIFIED(): Id {
    return genesisId("Modified");
  },

  // Helper functions
  name: () => genesisId("Name"),
  description: () => genesisId("Description"),
  avatar: () => genesisId("Avatar"),
  url: () => genesisId("URL"),
  created: () => genesisId("Created"),
  modified: () => genesisId("Modified"),
};

// =============================================================================
// CORE TYPES (Section 7.2)
// =============================================================================

/**
 * Well-known type entity IDs from the Genesis Space.
 */
export const types = {
  /** Person type - human individual */
  get PERSON(): Id {
    return genesisId("Person");
  },

  /** Organization type - company, DAO, institution */
  get ORGANIZATION(): Id {
    return genesisId("Organization");
  },

  /** Place type - geographic location */
  get PLACE(): Id {
    return genesisId("Place");
  },

  /** Topic type - subject or concept */
  get TOPIC(): Id {
    return genesisId("Topic");
  },

  // Helper functions
  person: () => genesisId("Person"),
  organization: () => genesisId("Organization"),
  place: () => genesisId("Place"),
  topic: () => genesisId("Topic"),
};

// =============================================================================
// CORE RELATION TYPES (Section 7.3)
// =============================================================================

/**
 * Well-known relation type IDs from the Genesis Space.
 */
export const relationTypes = {
  /** Types relation - type membership */
  get TYPES(): Id {
    return genesisId("Types");
  },

  /** PartOf relation - composition/containment */
  get PART_OF(): Id {
    return genesisId("PartOf");
  },

  /** RelatedTo relation - generic association */
  get RELATED_TO(): Id {
    return genesisId("RelatedTo");
  },

  // Helper functions
  types: () => genesisId("Types"),
  partOf: () => genesisId("PartOf"),
  relatedTo: () => genesisId("RelatedTo"),
};

// =============================================================================
// LANGUAGES (Section 7.4)
// =============================================================================

/**
 * Well-known language IDs from the Genesis Space.
 */
export const languages = {
  get ENGLISH(): Id {
    return languageId("en");
  },
  get SPANISH(): Id {
    return languageId("es");
  },
  get FRENCH(): Id {
    return languageId("fr");
  },
  get GERMAN(): Id {
    return languageId("de");
  },
  get CHINESE(): Id {
    return languageId("zh");
  },
  get CHINESE_HANS(): Id {
    return languageId("zh-hans");
  },
  get CHINESE_HANT(): Id {
    return languageId("zh-hant");
  },
  get JAPANESE(): Id {
    return languageId("ja");
  },
  get KOREAN(): Id {
    return languageId("ko");
  },
  get PORTUGUESE(): Id {
    return languageId("pt");
  },
  get ITALIAN(): Id {
    return languageId("it");
  },
  get RUSSIAN(): Id {
    return languageId("ru");
  },
  get ARABIC(): Id {
    return languageId("ar");
  },
  get HINDI(): Id {
    return languageId("hi");
  },

  // Helper functions
  english: () => languageId("en"),
  spanish: () => languageId("es"),
  french: () => languageId("fr"),
  german: () => languageId("de"),
  chinese: () => languageId("zh"),
  chineseHans: () => languageId("zh-hans"),
  chineseHant: () => languageId("zh-hant"),
  japanese: () => languageId("ja"),
  korean: () => languageId("ko"),
  portuguese: () => languageId("pt"),
  italian: () => languageId("it"),
  russian: () => languageId("ru"),
  arabic: () => languageId("ar"),
  hindi: () => languageId("hi"),

  /**
   * Returns the language ID for the given BCP 47 code.
   */
  fromCode: (code: string) => languageId(code),
};
