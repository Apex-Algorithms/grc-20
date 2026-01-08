//! Genesis Space well-known IDs.
//!
//! The Genesis Space provides well-known IDs for core properties, types,
//! and relation types (spec Section 7).

use crate::model::{derived_uuid, Id};

// =============================================================================
// ID DERIVATION
// =============================================================================

/// Derives a genesis ID from a name.
///
/// ```text
/// id = derived_uuid("grc20:genesis:" + name)
/// ```
pub fn genesis_id(name: &str) -> Id {
    let input = format!("grc20:genesis:{}", name);
    derived_uuid(input.as_bytes())
}

/// Derives a language ID from an ISO language code.
///
/// ```text
/// id = derived_uuid("grc20:genesis:language:" + code)
/// ```
pub fn language_id(code: &str) -> Id {
    let input = format!("grc20:genesis:language:{}", code);
    derived_uuid(input.as_bytes())
}

// =============================================================================
// CORE PROPERTIES (Section 7.1)
// =============================================================================

/// Well-known property IDs from the Genesis Space.
pub mod properties {
    use super::*;

    lazy_static::lazy_static! {
        /// Name property - primary label (TEXT)
        pub static ref NAME: Id = genesis_id("Name");

        /// Description property - summary text (TEXT)
        pub static ref DESCRIPTION: Id = genesis_id("Description");

        /// Avatar property - image URL (TEXT)
        pub static ref AVATAR: Id = genesis_id("Avatar");

        /// URL property - external link (TEXT)
        pub static ref URL: Id = genesis_id("URL");

        /// Created property - creation time (TIMESTAMP)
        pub static ref CREATED: Id = genesis_id("Created");

        /// Modified property - last modification (TIMESTAMP)
        pub static ref MODIFIED: Id = genesis_id("Modified");
    }

    /// Returns the Name property ID.
    pub fn name() -> Id {
        *NAME
    }

    /// Returns the Description property ID.
    pub fn description() -> Id {
        *DESCRIPTION
    }

    /// Returns the Avatar property ID.
    pub fn avatar() -> Id {
        *AVATAR
    }

    /// Returns the URL property ID.
    pub fn url() -> Id {
        *URL
    }

    /// Returns the Created property ID.
    pub fn created() -> Id {
        *CREATED
    }

    /// Returns the Modified property ID.
    pub fn modified() -> Id {
        *MODIFIED
    }
}

// =============================================================================
// CORE TYPES (Section 7.2)
// =============================================================================

/// Well-known type entity IDs from the Genesis Space.
pub mod types {
    use super::*;

    lazy_static::lazy_static! {
        /// Person type - human individual
        pub static ref PERSON: Id = genesis_id("Person");

        /// Organization type - company, DAO, institution
        pub static ref ORGANIZATION: Id = genesis_id("Organization");

        /// Place type - geographic location
        pub static ref PLACE: Id = genesis_id("Place");

        /// Topic type - subject or concept
        pub static ref TOPIC: Id = genesis_id("Topic");
    }

    /// Returns the Person type ID.
    pub fn person() -> Id {
        *PERSON
    }

    /// Returns the Organization type ID.
    pub fn organization() -> Id {
        *ORGANIZATION
    }

    /// Returns the Place type ID.
    pub fn place() -> Id {
        *PLACE
    }

    /// Returns the Topic type ID.
    pub fn topic() -> Id {
        *TOPIC
    }
}

// =============================================================================
// CORE RELATION TYPES (Section 7.3)
// =============================================================================

/// Well-known relation type IDs from the Genesis Space.
pub mod relation_types {
    use super::*;

    lazy_static::lazy_static! {
        /// Types relation - type membership
        pub static ref TYPES: Id = genesis_id("Types");

        /// PartOf relation - composition/containment
        pub static ref PART_OF: Id = genesis_id("PartOf");

        /// RelatedTo relation - generic association
        pub static ref RELATED_TO: Id = genesis_id("RelatedTo");
    }

    /// Returns the Types relation type ID.
    pub fn types() -> Id {
        *TYPES
    }

    /// Returns the PartOf relation type ID.
    pub fn part_of() -> Id {
        *PART_OF
    }

    /// Returns the RelatedTo relation type ID.
    pub fn related_to() -> Id {
        *RELATED_TO
    }
}

// =============================================================================
// LANGUAGES (Section 7.4)
// =============================================================================

/// Well-known language IDs from the Genesis Space.
pub mod languages {
    use super::*;

    lazy_static::lazy_static! {
        pub static ref ENGLISH: Id = language_id("en");
        pub static ref SPANISH: Id = language_id("es");
        pub static ref FRENCH: Id = language_id("fr");
        pub static ref GERMAN: Id = language_id("de");
        pub static ref CHINESE: Id = language_id("zh");
        pub static ref JAPANESE: Id = language_id("ja");
        pub static ref KOREAN: Id = language_id("ko");
        pub static ref PORTUGUESE: Id = language_id("pt");
        pub static ref ITALIAN: Id = language_id("it");
        pub static ref RUSSIAN: Id = language_id("ru");
        pub static ref ARABIC: Id = language_id("ar");
        pub static ref HINDI: Id = language_id("hi");
    }

    /// Returns the language ID for the given ISO code.
    ///
    /// This dynamically derives the ID - for frequently used languages,
    /// use the static constants instead.
    pub fn from_code(code: &str) -> Id {
        language_id(code)
    }

    pub fn english() -> Id {
        *ENGLISH
    }

    pub fn spanish() -> Id {
        *SPANISH
    }

    pub fn french() -> Id {
        *FRENCH
    }

    pub fn german() -> Id {
        *GERMAN
    }

    pub fn chinese() -> Id {
        *CHINESE
    }

    pub fn japanese() -> Id {
        *JAPANESE
    }

    pub fn korean() -> Id {
        *KOREAN
    }

    pub fn portuguese() -> Id {
        *PORTUGUESE
    }

    pub fn italian() -> Id {
        *ITALIAN
    }

    pub fn russian() -> Id {
        *RUSSIAN
    }

    pub fn arabic() -> Id {
        *ARABIC
    }

    pub fn hindi() -> Id {
        *HINDI
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::format_id;

    #[test]
    fn test_genesis_id_deterministic() {
        let id1 = genesis_id("Name");
        let id2 = genesis_id("Name");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_genesis_id_unique() {
        let name_id = genesis_id("Name");
        let desc_id = genesis_id("Description");
        assert_ne!(name_id, desc_id);
    }

    #[test]
    fn test_language_id_deterministic() {
        let en1 = language_id("en");
        let en2 = language_id("en");
        assert_eq!(en1, en2);
    }

    #[test]
    fn test_genesis_id_format() {
        // Genesis IDs should be valid UUIDv8
        let id = genesis_id("Name");
        // Version should be 8 (0x80 in high nibble of byte 6)
        assert_eq!(id[6] & 0xF0, 0x80);
        // Variant should be RFC 4122 (0b10 in high 2 bits of byte 8)
        assert_eq!(id[8] & 0xC0, 0x80);
    }

    #[test]
    fn test_static_properties() {
        // Verify static properties match dynamic derivation
        assert_eq!(properties::name(), genesis_id("Name"));
        assert_eq!(properties::description(), genesis_id("Description"));
        assert_eq!(properties::avatar(), genesis_id("Avatar"));
        assert_eq!(properties::url(), genesis_id("URL"));
        assert_eq!(properties::created(), genesis_id("Created"));
        assert_eq!(properties::modified(), genesis_id("Modified"));
    }

    #[test]
    fn test_static_types() {
        assert_eq!(types::person(), genesis_id("Person"));
        assert_eq!(types::organization(), genesis_id("Organization"));
        assert_eq!(types::place(), genesis_id("Place"));
        assert_eq!(types::topic(), genesis_id("Topic"));
    }

    #[test]
    fn test_static_relation_types() {
        assert_eq!(relation_types::types(), genesis_id("Types"));
        assert_eq!(relation_types::part_of(), genesis_id("PartOf"));
        assert_eq!(relation_types::related_to(), genesis_id("RelatedTo"));
    }

    #[test]
    fn test_static_languages() {
        assert_eq!(languages::english(), language_id("en"));
        assert_eq!(languages::spanish(), language_id("es"));
        assert_eq!(languages::from_code("en"), languages::english());
    }

    #[test]
    fn test_print_genesis_ids() {
        // This test prints genesis IDs for documentation
        println!("=== Core Properties ===");
        println!("Name: {}", format_id(&properties::name()));
        println!("Description: {}", format_id(&properties::description()));
        println!("Avatar: {}", format_id(&properties::avatar()));
        println!("URL: {}", format_id(&properties::url()));
        println!("Created: {}", format_id(&properties::created()));
        println!("Modified: {}", format_id(&properties::modified()));

        println!("\n=== Core Types ===");
        println!("Person: {}", format_id(&types::person()));
        println!("Organization: {}", format_id(&types::organization()));
        println!("Place: {}", format_id(&types::place()));
        println!("Topic: {}", format_id(&types::topic()));

        println!("\n=== Core Relation Types ===");
        println!("Types: {}", format_id(&relation_types::types()));
        println!("PartOf: {}", format_id(&relation_types::part_of()));
        println!("RelatedTo: {}", format_id(&relation_types::related_to()));

        println!("\n=== Languages ===");
        println!("English (en): {}", format_id(&languages::english()));
        println!("Spanish (es): {}", format_id(&languages::spanish()));
        println!("French (fr): {}", format_id(&languages::french()));
    }
}
