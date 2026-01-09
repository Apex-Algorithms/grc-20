import { derivedUuidFromString } from "../util/id.js";
import type { Id } from "../types/id.js";

// Re-export derivedUuidFromString for backward compatibility
export { derivedUuidFromString };

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

/**
 * Well-known language IDs.
 */
export const languages = {
  english: () => languageId("en"),
  spanish: () => languageId("es"),
  french: () => languageId("fr"),
  german: () => languageId("de"),
  chinese: () => languageId("zh"),
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
