/**
 * Zstd compression support for GRC-20 edits.
 *
 * This module uses lazy loading - the @bokuweb/zstd-wasm library is only
 * imported when compression/decompression is actually used.
 */

import type { Edit } from "../types/edit.js";
import { DecodeError, Reader, Writer } from "./primitives.js";
import { encodeEdit, decodeEdit, type EncodeOptions } from "./edit.js";

// Magic bytes for compressed format
const MAGIC_COMPRESSED = new TextEncoder().encode("GRC2Z");

// Default compression level (matches Rust implementation)
const DEFAULT_COMPRESSION_LEVEL = 3;

// Cached zstd functions
let zstdCompress: ((data: Uint8Array, level?: number) => Uint8Array) | null = null;
let zstdDecompress: ((data: Uint8Array) => Uint8Array) | null = null;
let zstdLoadPromise: Promise<void> | null = null;

/**
 * Loads the zstd-wasm library.
 */
async function loadZstd(): Promise<void> {
  const zstd = await import("@bokuweb/zstd-wasm");
  await zstd.init();
  zstdCompress = zstd.compress;
  zstdDecompress = zstd.decompress;
}

/**
 * Starts preloading zstd in the background (non-blocking).
 * Fails silently if the module can't be loaded (e.g., in browsers without bundler).
 */
function preloadZstd(): void {
  if (zstdLoadPromise !== null) return;

  zstdLoadPromise = loadZstd().catch(() => {
    // Preload failed (e.g., bare specifier in browser without bundler)
    // Will retry on-demand when compression is actually used
    zstdLoadPromise = null;
  });
}

// Start preloading in the background (non-blocking, fails silently)
preloadZstd();

/**
 * Ensures zstd is loaded and ready.
 * If preload failed, retries loading on-demand.
 */
async function ensureZstdLoaded(): Promise<void> {
  if (zstdCompress !== null && zstdDecompress !== null) {
    return;
  }

  if (zstdLoadPromise !== null) {
    await zstdLoadPromise;
    if (zstdCompress !== null) return;
  }

  // Preload failed or not started, try loading now
  try {
    zstdLoadPromise = loadZstd();
    await zstdLoadPromise;
  } catch (error) {
    throw new Error(
      "Failed to load zstd-wasm. Compression requires a bundler (Vite, webpack, etc.) " +
      "or an import map to resolve '@bokuweb/zstd-wasm'. " +
      "Original error: " + (error instanceof Error ? error.message : String(error))
    );
  }
}

/**
 * Checks if the compression module is loaded and ready.
 *
 * Use this to check if compression functions can be called synchronously
 * (after preloading) or if they will need to load the WASM first.
 *
 * @returns true if WASM is loaded and compression is ready
 */
export function isCompressionReady(): boolean {
  return zstdCompress !== null && zstdDecompress !== null;
}

/**
 * Preloads the compression WASM module.
 *
 * Call this on app startup to ensure compression is ready when needed.
 * The promise resolves when WASM is fully loaded and initialized.
 *
 * @example
 * ```typescript
 * // On app startup
 * import { preloadCompression } from '@geoprotocol/grc-20';
 *
 * preloadCompression().then(() => {
 *   console.log('Compression ready!');
 * });
 * ```
 *
 * @returns Promise that resolves when compression is ready
 */
export async function preloadCompression(): Promise<void> {
  await ensureZstdLoaded();
}

/**
 * Checks if data appears to be compressed (starts with GRC2Z magic).
 */
export function isCompressed(data: Uint8Array): boolean {
  if (data.length < 5) return false;
  for (let i = 0; i < 5; i++) {
    if (data[i] !== MAGIC_COMPRESSED[i]) return false;
  }
  return true;
}

/**
 * Compresses raw bytes using Zstd.
 *
 * @param data - The data to compress
 * @param level - Compression level (default: 3)
 * @returns The compressed data (without GRC2Z header)
 */
export async function compress(data: Uint8Array, level?: number): Promise<Uint8Array> {
  await ensureZstdLoaded();
  return zstdCompress!(data, level ?? DEFAULT_COMPRESSION_LEVEL);
}

/**
 * Decompresses raw Zstd-compressed bytes.
 *
 * @param data - The compressed data (without GRC2Z header)
 * @returns The decompressed data
 */
export async function decompress(data: Uint8Array): Promise<Uint8Array> {
  await ensureZstdLoaded();
  return zstdDecompress!(data);
}

/**
 * Encodes an Edit to compressed binary format (GRC2Z).
 *
 * Format:
 * - 5 bytes: "GRC2Z" magic
 * - varint: uncompressed size
 * - bytes: zstd-compressed GRC2 data
 *
 * @param edit - The edit to encode
 * @param options - Encoding options (e.g., canonical mode)
 * @returns The compressed binary data
 */
export async function encodeEditCompressed(
  edit: Edit,
  options?: EncodeOptions
): Promise<Uint8Array> {
  // First encode to uncompressed format
  const uncompressed = encodeEdit(edit, options);

  // Compress the data
  const compressed = await compress(uncompressed);

  // Build the final output: magic + uncompressed size + compressed data
  const writer = new Writer(5 + 10 + compressed.length);
  writer.writeBytes(MAGIC_COMPRESSED);
  writer.writeVarintNumber(uncompressed.length);
  writer.writeBytes(compressed);

  return writer.finish();
}

/**
 * Options for auto-encoding.
 */
export interface EncodeAutoOptions extends EncodeOptions {
  /**
   * Minimum uncompressed size (in bytes) before compression is applied.
   * If the uncompressed data is smaller than this threshold, it will
   * be returned as-is (uncompressed GRC2 format).
   *
   * Default: 256 bytes
   *
   * Set to 0 to always compress.
   * Set to Infinity to never compress.
   */
  threshold?: number;
}

// Default threshold for auto-compression (256 bytes)
const DEFAULT_COMPRESSION_THRESHOLD = 256;

/**
 * Encodes an Edit to binary format, automatically choosing compression.
 *
 * This is the recommended encoding function for most use cases. It:
 * - Encodes small edits without compression (faster, no WASM needed)
 * - Compresses larger edits for better size efficiency
 *
 * The output can be decoded with `decodeEditAuto()`.
 *
 * @param edit - The edit to encode
 * @param options - Encoding options including compression threshold
 * @returns The binary data (GRC2 or GRC2Z format depending on size)
 *
 * @example
 * ```typescript
 * // Auto-compress (default threshold: 256 bytes)
 * const data = await encodeEditAuto(edit);
 *
 * // Always compress
 * const compressed = await encodeEditAuto(edit, { threshold: 0 });
 *
 * // Never compress
 * const uncompressed = await encodeEditAuto(edit, { threshold: Infinity });
 *
 * // Custom threshold (1KB)
 * const data = await encodeEditAuto(edit, { threshold: 1024 });
 * ```
 */
export async function encodeEditAuto(
  edit: Edit,
  options?: EncodeAutoOptions
): Promise<Uint8Array> {
  const threshold = options?.threshold ?? DEFAULT_COMPRESSION_THRESHOLD;

  // First encode to uncompressed format
  const uncompressed = encodeEdit(edit, options);

  // If below threshold, return uncompressed
  if (uncompressed.length < threshold) {
    return uncompressed;
  }

  // Compress the data
  const compressed = await compress(uncompressed);

  // Build the final output: magic + uncompressed size + compressed data
  const writer = new Writer(5 + 10 + compressed.length);
  writer.writeBytes(MAGIC_COMPRESSED);
  writer.writeVarintNumber(uncompressed.length);
  writer.writeBytes(compressed);

  return writer.finish();
}

/**
 * Decodes a compressed Edit from binary data (GRC2Z format).
 *
 * @param data - The compressed binary data
 * @returns The decoded Edit
 * @throws DecodeError if the data is invalid or not compressed
 */
export async function decodeEditCompressed(data: Uint8Array): Promise<Edit> {
  // Check magic
  if (!isCompressed(data)) {
    throw new DecodeError(
      "E001",
      "invalid magic bytes: expected GRC2Z for compressed data"
    );
  }

  const reader = new Reader(data);

  // Skip magic
  reader.readBytes(5);

  // Read declared uncompressed size
  const declaredSize = reader.readVarintNumber();

  // Read compressed data
  const compressedData = reader.readBytes(reader.remaining());

  // Decompress
  const decompressed = await decompress(compressedData);

  // Verify size
  if (decompressed.length !== declaredSize) {
    throw new DecodeError(
      "E001",
      `uncompressed size mismatch: declared ${declaredSize}, actual ${decompressed.length}`
    );
  }

  // Decode the uncompressed edit
  return decodeEdit(decompressed);
}

/**
 * Decodes an Edit from binary data, automatically detecting compression.
 *
 * This is a convenience function that handles both compressed (GRC2Z)
 * and uncompressed (GRC2) formats.
 *
 * @param data - The binary data (compressed or uncompressed)
 * @returns The decoded Edit
 */
export async function decodeEditAuto(data: Uint8Array): Promise<Edit> {
  if (isCompressed(data)) {
    return decodeEditCompressed(data);
  }
  return decodeEdit(data);
}
