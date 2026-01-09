//! Benchmark for old GRC-20 proto serialization using city data.
//!
//! Uses the same dataset and schema as grc-20-bench for comparison.

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::Instant;

use prost::Message;
use serde::Deserialize;

// Include generated protobuf code
pub mod grc20 {
    include!(concat!(env!("OUT_DIR"), "/grc20.rs"));
}

// =============================================================================
// HARDCODED UUIDs FOR SCHEMA (same as grc-20-bench)
// =============================================================================

const fn hex(s: &str) -> [u8; 16] {
    let bytes = s.as_bytes();
    let mut result = [0u8; 16];
    let mut i = 0;
    while i < 16 {
        let hi = hex_digit(bytes[i * 2]);
        let lo = hex_digit(bytes[i * 2 + 1]);
        result[i] = (hi << 4) | lo;
        i += 1;
    }
    result
}

const fn hex_digit(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => panic!("invalid hex digit"),
    }
}

/// Property IDs (same as grc-20-bench)
mod props {
    use super::hex;

    pub const NAME: [u8; 16] = hex("a1b2c3d4e5f6071829304050a1b2c3d4");
    pub const CODE: [u8; 16] = hex("a1b2c3d4e5f6071829304050a1b2c3d5");
    pub const NATIVE_NAME: [u8; 16] = hex("a1b2c3d4e5f6071829304050a1b2c3d6");
    pub const POPULATION: [u8; 16] = hex("a1b2c3d4e5f6071829304050a1b2c3d7");
    pub const LOCATION: [u8; 16] = hex("a1b2c3d4e5f6071829304050a1b2c3d8");
    pub const TIMEZONE: [u8; 16] = hex("a1b2c3d4e5f6071829304050a1b2c3d9");
    pub const WIKIDATA_ID: [u8; 16] = hex("a1b2c3d4e5f6071829304050a1b2c3da");
    pub const CITY_TYPE: [u8; 16] = hex("a1b2c3d4e5f6071829304050a1b2c3db");
}

/// Type IDs (same as grc-20-bench)
mod types {
    use super::hex;

    pub const CITY: [u8; 16] = hex("b1b2c3d4e5f6071829304050a1b2c3d4");
    pub const STATE: [u8; 16] = hex("b1b2c3d4e5f6071829304050a1b2c3d5");
    pub const COUNTRY: [u8; 16] = hex("b1b2c3d4e5f6071829304050a1b2c3d6");
}

/// Relation type IDs (same as grc-20-bench)
mod rel_types {
    use super::hex;

    pub const TYPES: [u8; 16] = hex("c1b2c3d4e5f6071829304050a1b2c3d4");
    pub const IN_STATE: [u8; 16] = hex("c1b2c3d4e5f6071829304050a1b2c3d5");
    pub const IN_COUNTRY: [u8; 16] = hex("c1b2c3d4e5f6071829304050a1b2c3d6");
}

/// Language IDs (same as grc-20-bench)
mod langs {
    use super::hex;

    pub const BRETON: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3d0");
    pub const KOREAN: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3d1");
    pub const PORTUGUESE_BR: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3d2");
    pub const PORTUGUESE: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3d3");
    pub const DUTCH: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3d4");
    pub const CROATIAN: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3d5");
    pub const PERSIAN: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3d6");
    pub const GERMAN: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3d7");
    pub const SPANISH: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3d8");
    pub const FRENCH: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3d9");
    pub const JAPANESE: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3da");
    pub const ITALIAN: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3db");
    pub const CHINESE: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3dc");
    pub const TURKISH: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3dd");
    pub const RUSSIAN: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3de");
    pub const UKRAINIAN: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3df");
    pub const POLISH: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3e0");
    pub const ARABIC: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3e1");
    pub const HINDI: [u8; 16] = hex("d1b2c3d4e5f6071829304050a1b2c3e2");

    pub fn from_code(code: &str) -> Option<[u8; 16]> {
        match code {
            "br" => Some(BRETON),
            "ko" => Some(KOREAN),
            "pt-BR" => Some(PORTUGUESE_BR),
            "pt" => Some(PORTUGUESE),
            "nl" => Some(DUTCH),
            "hr" => Some(CROATIAN),
            "fa" => Some(PERSIAN),
            "de" => Some(GERMAN),
            "es" => Some(SPANISH),
            "fr" => Some(FRENCH),
            "ja" => Some(JAPANESE),
            "it" => Some(ITALIAN),
            "zh-CN" => Some(CHINESE),
            "tr" => Some(TURKISH),
            "ru" => Some(RUSSIAN),
            "uk" => Some(UKRAINIAN),
            "pl" => Some(POLISH),
            "ar" => Some(ARABIC),
            "hi" => Some(HINDI),
            _ => None,
        }
    }
}

// Entity ID prefixes (same as grc-20-bench)
const PREFIX_CITY: u8 = 0x01;
const PREFIX_STATE: u8 = 0x02;
const PREFIX_COUNTRY: u8 = 0x03;

// =============================================================================
// JSON DATA STRUCTURES (same as grc-20-bench)
// =============================================================================

#[derive(Debug, Deserialize)]
struct City {
    id: u32,
    name: String,
    state_id: u32,
    state_code: String,
    state_name: String,
    country_id: u32,
    country_code: String,
    country_name: String,
    latitude: String,
    longitude: String,
    native: Option<String>,
    #[serde(rename = "type")]
    city_type: Option<String>,
    population: Option<i64>,
    timezone: Option<String>,
    translations: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "wikiDataId")]
    wikidata_id: Option<String>,
}

// =============================================================================
// ID GENERATION (same as grc-20-bench)
// =============================================================================

fn make_entity_id(prefix: u8, id: u32) -> Vec<u8> {
    let mut uuid = [0u8; 16];
    uuid[0] = prefix;
    uuid[12..16].copy_from_slice(&id.to_be_bytes());
    // Set version 8 and variant
    uuid[6] = (uuid[6] & 0x0F) | 0x80;
    uuid[8] = (uuid[8] & 0x3F) | 0x80;
    uuid.to_vec()
}

/// Create a deterministic relation entity ID.
fn make_rel_entity_id(prefix: u8, entity_id: u32, rel_type: u8, index: u16) -> Vec<u8> {
    let mut result = [0u8; 16];
    result[0] = prefix;
    result[1] = rel_type;
    result[2..4].copy_from_slice(&index.to_be_bytes());
    result[12..16].copy_from_slice(&entity_id.to_be_bytes());
    result.to_vec()
}

// =============================================================================
// CONVERSION CONTEXT
// =============================================================================

struct ConversionContext {
    ops: Vec<grc20::Op>,
    created_states: HashSet<u32>,
    created_countries: HashSet<u32>,
}

impl ConversionContext {
    fn new() -> Self {
        Self {
            ops: Vec::new(),
            created_states: HashSet::new(),
            created_countries: HashSet::new(),
        }
    }

    fn make_value(&self, property: &[u8], value: String) -> grc20::Value {
        grc20::Value {
            property: property.to_vec(),
            value,
            options: None,
        }
    }

    fn make_text_value(
        &self,
        property: &[u8],
        value: String,
        language: Option<[u8; 16]>,
    ) -> grc20::Value {
        grc20::Value {
            property: property.to_vec(),
            value,
            options: language.map(|lang| grc20::Options {
                value: Some(grc20::options::Value::Text(grc20::TextOptions {
                    language: Some(lang.to_vec()),
                })),
            }),
        }
    }

    fn create_relation(
        &mut self,
        from_entity: Vec<u8>,
        to_entity: Vec<u8>,
        rel_type: [u8; 16],
        rel_entity_id: Vec<u8>,
    ) {
        self.ops.push(grc20::Op {
            payload: Some(grc20::op::Payload::CreateRelation(grc20::Relation {
                id: rel_entity_id.clone(),
                r#type: rel_type.to_vec(),
                from_entity,
                from_space: None,
                from_version: None,
                to_entity,
                to_space: None,
                to_version: None,
                entity: rel_entity_id,
                position: None,
                verified: None,
            })),
        });
    }

    fn ensure_country(&mut self, country_id: u32, name: &str, code: &str) {
        if self.created_countries.contains(&country_id) {
            return;
        }
        self.created_countries.insert(country_id);

        let entity_id = make_entity_id(PREFIX_COUNTRY, country_id);

        // Create country entity
        self.ops.push(grc20::Op {
            payload: Some(grc20::op::Payload::UpdateEntity(grc20::Entity {
                id: entity_id.clone(),
                values: vec![
                    self.make_value(&props::NAME, name.to_string()),
                    self.make_value(&props::CODE, code.to_string()),
                ],
            })),
        });

        // TYPES relation
        let rel_entity_id = make_rel_entity_id(PREFIX_COUNTRY, country_id, 0, 0);
        self.create_relation(
            entity_id,
            types::COUNTRY.to_vec(),
            rel_types::TYPES,
            rel_entity_id,
        );
    }

    fn ensure_state(&mut self, state_id: u32, name: &str, code: &str, country_id: u32) {
        if self.created_states.contains(&state_id) {
            return;
        }
        self.created_states.insert(state_id);

        let entity_id = make_entity_id(PREFIX_STATE, state_id);
        let country_entity_id = make_entity_id(PREFIX_COUNTRY, country_id);

        // Create state entity
        self.ops.push(grc20::Op {
            payload: Some(grc20::op::Payload::UpdateEntity(grc20::Entity {
                id: entity_id.clone(),
                values: vec![
                    self.make_value(&props::NAME, name.to_string()),
                    self.make_value(&props::CODE, code.to_string()),
                ],
            })),
        });

        // TYPES relation
        let rel_entity_id = make_rel_entity_id(PREFIX_STATE, state_id, 0, 0);
        self.create_relation(
            entity_id.clone(),
            types::STATE.to_vec(),
            rel_types::TYPES,
            rel_entity_id,
        );

        // IN_COUNTRY relation
        let rel_entity_id = make_rel_entity_id(PREFIX_STATE, state_id, 1, 0);
        self.create_relation(
            entity_id,
            country_entity_id,
            rel_types::IN_COUNTRY,
            rel_entity_id,
        );
    }

    fn add_city(&mut self, city: &City) {
        let entity_id = make_entity_id(PREFIX_CITY, city.id);
        let state_entity_id = make_entity_id(PREFIX_STATE, city.state_id);
        let country_entity_id = make_entity_id(PREFIX_COUNTRY, city.country_id);

        // Ensure country exists
        self.ensure_country(city.country_id, &city.country_name, &city.country_code);

        // Ensure state exists
        self.ensure_state(
            city.state_id,
            &city.state_name,
            &city.state_code,
            city.country_id,
        );

        // Build city values
        let mut values = vec![self.make_value(&props::NAME, city.name.clone())];

        // Native name
        if let Some(ref native) = city.native {
            if !native.is_empty() {
                values.push(self.make_value(&props::NATIVE_NAME, native.clone()));
            }
        }

        // City type
        if let Some(ref city_type) = city.city_type {
            values.push(self.make_value(&props::CITY_TYPE, city_type.clone()));
        }

        // Population (as string for proto)
        if let Some(pop) = city.population {
            values.push(self.make_value(&props::POPULATION, pop.to_string()));
        }

        // Location as "lat,lon" string (proto doesn't have native Point type)
        if let (Ok(lat), Ok(lon)) = (city.latitude.parse::<f64>(), city.longitude.parse::<f64>()) {
            values.push(self.make_value(&props::LOCATION, format!("{},{}", lat, lon)));
        }

        // Timezone
        if let Some(ref tz) = city.timezone {
            values.push(self.make_value(&props::TIMEZONE, tz.clone()));
        }

        // Wikidata ID
        if let Some(ref wiki_id) = city.wikidata_id {
            values.push(self.make_value(&props::WIKIDATA_ID, wiki_id.clone()));
        }

        // Translations (multi-value TEXT with language)
        if let Some(ref translations) = city.translations {
            for (lang_code, translation) in translations {
                if let Some(lang_id) = langs::from_code(lang_code) {
                    values.push(self.make_text_value(&props::NAME, translation.clone(), Some(lang_id)));
                }
            }
        }

        // Create city entity
        self.ops.push(grc20::Op {
            payload: Some(grc20::op::Payload::UpdateEntity(grc20::Entity {
                id: entity_id.clone(),
                values,
            })),
        });

        // TYPES relation
        let rel_entity_id = make_rel_entity_id(PREFIX_CITY, city.id, 0, 0);
        self.create_relation(
            entity_id.clone(),
            types::CITY.to_vec(),
            rel_types::TYPES,
            rel_entity_id,
        );

        // IN_STATE relation
        let rel_entity_id = make_rel_entity_id(PREFIX_CITY, city.id, 1, 0);
        self.create_relation(
            entity_id.clone(),
            state_entity_id,
            rel_types::IN_STATE,
            rel_entity_id,
        );

        // IN_COUNTRY relation
        let rel_entity_id = make_rel_entity_id(PREFIX_CITY, city.id, 2, 0);
        self.create_relation(entity_id, country_entity_id, rel_types::IN_COUNTRY, rel_entity_id);
    }
}

fn main() {
    // Find the data file (look in out/ directory)
    let data_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../../../out/cities.json".to_string());

    println!("Loading cities from: {}", data_path);

    // Check if file exists, if not try to decompress from data/
    if !Path::new(&data_path).exists() {
        let compressed_path = data_path.replace("/out/", "/data/") + ".gz";
        if Path::new(&compressed_path).exists() {
            println!("Decompressing {} to {}", compressed_path, data_path);
            let compressed = fs::read(&compressed_path).expect("Failed to read compressed file");
            let mut decoder = flate2::read::GzDecoder::new(compressed.as_slice());
            let mut decompressed = String::new();
            std::io::Read::read_to_string(&mut decoder, &mut decompressed)
                .expect("Failed to decompress");
            fs::create_dir_all(Path::new(&data_path).parent().unwrap()).ok();
            fs::write(&data_path, &decompressed).expect("Failed to write decompressed file");
        }
    }

    let json_data = fs::read_to_string(&data_path).expect("Failed to read cities.json");

    let parse_start = Instant::now();
    let cities: Vec<City> = serde_json::from_str(&json_data).expect("Failed to parse JSON");
    let parse_time = parse_start.elapsed();

    println!("Loaded {} cities in {:?}", cities.len(), parse_time);

    // Convert to proto operations
    let convert_start = Instant::now();
    let mut ctx = ConversionContext::new();
    for city in &cities {
        ctx.add_city(city);
    }
    let convert_time = convert_start.elapsed();

    println!(
        "Converted to {} operations in {:?}",
        ctx.ops.len(),
        convert_time
    );
    println!(
        "  - {} states, {} countries",
        ctx.created_states.len(),
        ctx.created_countries.len()
    );

    // Count operation types
    let mut entity_count = 0;
    let mut relation_count = 0;
    let mut total_values = 0;
    for op in &ctx.ops {
        match &op.payload {
            Some(grc20::op::Payload::UpdateEntity(e)) => {
                entity_count += 1;
                total_values += e.values.len();
            }
            Some(grc20::op::Payload::CreateRelation(_)) => relation_count += 1,
            _ => {}
        }
    }
    println!(
        "  - {} entities, {} relations, {} total values",
        entity_count, relation_count, total_values
    );

    // Create edit
    let edit = grc20::Edit {
        id: make_entity_id(0xFF, 1),
        name: "Cities Import".to_string(),
        ops: ctx.ops,
        authors: vec![make_entity_id(0xAA, 1)],
        language: None,
    };

    // Wrap in File message
    let file = grc20::File {
        version: "1.0.0".to_string(),
        payload: Some(grc20::file::Payload::AddEdit(edit.clone())),
    };

    // Benchmark encoding (uncompressed)
    let encode_start = Instant::now();
    let encoded = file.encode_to_vec();
    let encode_time = encode_start.elapsed();

    println!(
        "\nUncompressed: {} bytes in {:?}",
        encoded.len(),
        encode_time
    );
    println!(
        "  Throughput: {:.2} MB/s",
        (encoded.len() as f64 / 1_000_000.0) / encode_time.as_secs_f64()
    );

    // Benchmark encoding (compressed with zstd)
    let compress_start = Instant::now();
    let compressed = zstd::encode_all(encoded.as_slice(), 3).expect("Failed to compress");
    let compress_time = compress_start.elapsed();

    println!(
        "\nCompressed (level 3): {} bytes in {:?}",
        compressed.len(),
        compress_time
    );
    println!(
        "  Compression ratio: {:.1}x",
        encoded.len() as f64 / compressed.len() as f64
    );
    println!(
        "  Throughput: {:.2} MB/s (uncompressed equivalent)",
        (encoded.len() as f64 / 1_000_000.0) / compress_time.as_secs_f64()
    );

    // Benchmark decoding (uncompressed) - multiple iterations
    const DECODE_ITERS: u32 = 10; // Fewer iterations due to larger data
    // Warmup
    for _ in 0..3 {
        let _ = grc20::File::decode(encoded.as_slice()).expect("Failed to decode");
    }
    let decode_start = Instant::now();
    let mut decoded = None;
    for _ in 0..DECODE_ITERS {
        decoded = Some(grc20::File::decode(encoded.as_slice()).expect("Failed to decode"));
    }
    let decode_time = decode_start.elapsed() / DECODE_ITERS;
    let decoded = decoded.unwrap();

    println!(
        "\nDecode (uncompressed): {:?} (avg of {} iterations)",
        decode_time, DECODE_ITERS
    );
    println!(
        "  Throughput: {:.2} MB/s",
        (encoded.len() as f64 / 1_000_000.0) / decode_time.as_secs_f64()
    );

    // Verify decode
    if let Some(grc20::file::Payload::AddEdit(decoded_edit)) = decoded.payload {
        assert_eq!(decoded_edit.ops.len(), edit.ops.len());
    }

    // Benchmark decoding (compressed) - multiple iterations
    // Warmup
    for _ in 0..3 {
        let decompressed = zstd::decode_all(compressed.as_slice()).expect("Failed to decompress");
        let _ = grc20::File::decode(decompressed.as_slice()).expect("Failed to decode");
    }
    let decode_compressed_start = Instant::now();
    let mut decoded_compressed = None;
    for _ in 0..DECODE_ITERS {
        let decompressed = zstd::decode_all(compressed.as_slice()).expect("Failed to decompress");
        decoded_compressed =
            Some(grc20::File::decode(decompressed.as_slice()).expect("Failed to decode"));
    }
    let decode_compressed_time = decode_compressed_start.elapsed() / DECODE_ITERS;
    let decoded_compressed = decoded_compressed.unwrap();

    println!(
        "\nDecode (compressed): {:?} (avg of {} iterations)",
        decode_compressed_time, DECODE_ITERS
    );
    println!(
        "  Throughput: {:.2} MB/s (uncompressed equivalent)",
        (encoded.len() as f64 / 1_000_000.0) / decode_compressed_time.as_secs_f64()
    );

    if let Some(grc20::file::Payload::AddEdit(decoded_edit)) = decoded_compressed.payload {
        assert_eq!(decoded_edit.ops.len(), edit.ops.len());
    }

    // Write output files
    let input_path = Path::new(&data_path);
    let stem = input_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let parent = input_path.parent().unwrap_or(Path::new("."));

    let output_uncompressed = parent.join(format!("{}.pb", stem));
    let output_compressed = parent.join(format!("{}.pbz", stem));

    fs::write(&output_uncompressed, &encoded).expect("Failed to write .pb file");
    fs::write(&output_compressed, &compressed).expect("Failed to write .pbz file");

    println!("\n=== Output Files ===");
    println!("Uncompressed: {}", output_uncompressed.display());
    println!("Compressed:   {}", output_compressed.display());

    // Summary
    println!("\n=== Summary ===");
    println!("Cities: {}", cities.len());
    println!("States: {}", ctx.created_states.len());
    println!("Countries: {}", ctx.created_countries.len());
    println!("Total operations: {}", edit.ops.len());
    println!(
        "JSON size: {} bytes ({:.1} MB)",
        json_data.len(),
        json_data.len() as f64 / 1_000_000.0
    );
    println!(
        "Proto uncompressed: {} bytes ({:.1} MB)",
        encoded.len(),
        encoded.len() as f64 / 1_000_000.0
    );
    println!(
        "Proto compressed: {} bytes ({:.1} MB)",
        compressed.len(),
        compressed.len() as f64 / 1_000_000.0
    );
    println!(
        "Size vs JSON: {:.1}% (uncompressed), {:.1}% (compressed)",
        100.0 * encoded.len() as f64 / json_data.len() as f64,
        100.0 * compressed.len() as f64 / json_data.len() as f64
    );
}
