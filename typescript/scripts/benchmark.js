#!/usr/bin/env node

/**
 * Performance benchmark for GRC-20 encoding/decoding with compression.
 */

import { EditBuilder, encodeEdit, decodeEdit, encodeEditCompressed, decodeEditCompressed, properties, randomId } from "../dist/index.js";

function formatBytes(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

function formatTime(ms) {
  if (ms < 1) return `${(ms * 1000).toFixed(2)} µs`;
  if (ms < 1000) return `${ms.toFixed(2)} ms`;
  return `${(ms / 1000).toFixed(2)} s`;
}

function createTestEdit(entityCount) {
  const editId = randomId();
  const builder = new EditBuilder(editId)
    .setName(`Benchmark Edit with ${entityCount} entities`)
    .setCreatedAt(BigInt(Date.now()) * 1000n);

  for (let i = 0; i < entityCount; i++) {
    const entityId = randomId();
    builder.createEntity(entityId, (e) =>
      e.text(properties.name(), `Entity number ${i} with some padding text to make it realistic`, undefined)
       .text(properties.description(), "This is a description that should compress well because it repeats across many entities in the edit", undefined)
    );
  }

  return builder.build();
}

async function benchmark(name, fn, iterations = 10) {
  // Warmup
  await fn();
  await fn();

  const times = [];
  for (let i = 0; i < iterations; i++) {
    const start = performance.now();
    await fn();
    times.push(performance.now() - start);
  }

  const avg = times.reduce((a, b) => a + b, 0) / times.length;
  const min = Math.min(...times);
  const max = Math.max(...times);

  return { name, avg, min, max };
}

async function main() {
  console.log("GRC-20 Performance Benchmark\n");
  console.log("=".repeat(70));

  // Test with different sizes
  const sizes = [10, 100, 500];

  for (const entityCount of sizes) {
    console.log(`\n📊 Edit with ${entityCount} entities:\n`);

    const edit = createTestEdit(entityCount);

    // Encode uncompressed
    let encoded;
    const encodeResult = await benchmark("Encode (uncompressed)", () => {
      encoded = encodeEdit(edit);
      return encoded;
    });

    // Decode uncompressed
    const decodeResult = await benchmark("Decode (uncompressed)", () => {
      return decodeEdit(encoded);
    });

    // Encode compressed
    let compressed;
    const encodeCompResult = await benchmark("Encode (compressed)", async () => {
      compressed = await encodeEditCompressed(edit);
      return compressed;
    });

    // Decode compressed
    const decodeCompResult = await benchmark("Decode (compressed)", async () => {
      return await decodeEditCompressed(compressed);
    });

    // Print results
    console.log("Operation".padEnd(25) + "Avg".padStart(12) + "Min".padStart(12) + "Max".padStart(12));
    console.log("-".repeat(61));

    for (const result of [encodeResult, decodeResult, encodeCompResult, decodeCompResult]) {
      console.log(
        result.name.padEnd(25) +
        formatTime(result.avg).padStart(12) +
        formatTime(result.min).padStart(12) +
        formatTime(result.max).padStart(12)
      );
    }

    console.log("");
    console.log(`Uncompressed size: ${formatBytes(encoded.length)}`);
    console.log(`Compressed size:   ${formatBytes(compressed.length)} (${((1 - compressed.length / encoded.length) * 100).toFixed(1)}% smaller)`);

    // Throughput
    const encodeOpsPerSec = 1000 / encodeResult.avg;
    const decodeOpsPerSec = 1000 / decodeResult.avg;
    const encodeCompOpsPerSec = 1000 / encodeCompResult.avg;
    const decodeCompOpsPerSec = 1000 / decodeCompResult.avg;

    console.log("");
    console.log(`Throughput (encode):      ${encodeOpsPerSec.toFixed(0)} ops/s (uncompressed), ${encodeCompOpsPerSec.toFixed(0)} ops/s (compressed)`);
    console.log(`Throughput (decode):      ${decodeOpsPerSec.toFixed(0)} ops/s (uncompressed), ${decodeCompOpsPerSec.toFixed(0)} ops/s (compressed)`);
  }

  console.log("\n" + "=".repeat(70));
  console.log("✅ Benchmark complete\n");
}

main().catch(console.error);
