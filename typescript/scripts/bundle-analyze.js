#!/usr/bin/env node

import * as esbuild from "esbuild";
import { gzipSync } from "zlib";
import { readFileSync, rmSync, mkdirSync, existsSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const outDir = join(__dirname, "../.bundle-analysis");

// Entry points to analyze
const entryPoints = [
  { name: "full", entry: "../dist/index.js", description: "Full library" },
  { name: "types-only", entry: "../dist/types/index.js", description: "Types only" },
  { name: "builder-only", entry: "../dist/builder/index.js", description: "Builder only" },
  { name: "codec-only", entry: "../dist/codec/index.js", description: "Codec only" },
  { name: "genesis-only", entry: "../dist/genesis/index.js", description: "Genesis only" },
  { name: "util-only", entry: "../dist/util/index.js", description: "Utilities only" },
];

// Also test lazy loading scenario
const lazyLoadScenarios = [
  {
    name: "lazy-codec",
    description: "Types + Builder (codec lazy loaded)",
    code: `
      export * from "../dist/types/index.js";
      export * from "../dist/builder/index.js";
      export * from "../dist/genesis/index.js";
      export * from "../dist/util/index.js";
    `,
  },
];

async function analyzeBundle(name, entryOrCode, isCode = false) {
  const outfile = join(outDir, `${name}.js`);

  try {
    if (isCode) {
      // Write temp file
      const tempFile = join(outDir, `${name}-temp.js`);
      const fs = await import("fs/promises");
      await fs.writeFile(tempFile, entryOrCode);

      await esbuild.build({
        entryPoints: [tempFile],
        bundle: true,
        minify: true,
        format: "esm",
        outfile,
        platform: "browser",
        target: "es2022",
        external: [],
      });

      await fs.unlink(tempFile);
    } else {
      await esbuild.build({
        entryPoints: [join(__dirname, entryOrCode)],
        bundle: true,
        minify: true,
        format: "esm",
        outfile,
        platform: "browser",
        target: "es2022",
        external: [],
      });
    }

    const content = readFileSync(outfile);
    const gzipped = gzipSync(content);

    return {
      raw: content.length,
      gzip: gzipped.length,
    };
  } catch (error) {
    return { error: error.message };
  }
}

function formatBytes(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

async function main() {
  console.log("Bundle Size Analysis for @geoprotocol/grc-20\n");
  console.log("=".repeat(70));

  // Clean and create output dir
  if (existsSync(outDir)) {
    rmSync(outDir, { recursive: true });
  }
  mkdirSync(outDir, { recursive: true });

  // Check if dist exists
  if (!existsSync(join(__dirname, "../dist/index.js"))) {
    console.error("Error: dist/ not found. Run 'npm run build' first.");
    process.exit(1);
  }

  // Analyze entry points
  console.log("\n📦 Entry Point Sizes:\n");
  console.log("Entry Point".padEnd(25) + "Raw".padStart(12) + "Gzipped".padStart(12));
  console.log("-".repeat(49));

  for (const ep of entryPoints) {
    const result = await analyzeBundle(ep.name, ep.entry);
    if (result.error) {
      console.log(`${ep.description.padEnd(25)} ERROR: ${result.error}`);
    } else {
      console.log(
        `${ep.description.padEnd(25)}${formatBytes(result.raw).padStart(12)}${formatBytes(result.gzip).padStart(12)}`
      );
    }
  }

  // Analyze lazy loading scenarios
  console.log("\n📦 Lazy Loading Scenarios:\n");
  console.log("Scenario".padEnd(35) + "Raw".padStart(12) + "Gzipped".padStart(12));
  console.log("-".repeat(59));

  for (const scenario of lazyLoadScenarios) {
    const result = await analyzeBundle(scenario.name, scenario.code, true);
    if (result.error) {
      console.log(`${scenario.description.padEnd(35)} ERROR: ${result.error}`);
    } else {
      console.log(
        `${scenario.description.padEnd(35)}${formatBytes(result.raw).padStart(12)}${formatBytes(result.gzip).padStart(12)}`
      );
    }
  }

  // Calculate what you save with lazy loading
  console.log("\n💡 Bundle Splitting Analysis:\n");

  const fullResult = await analyzeBundle("full-check", "../dist/index.js");
  const lazyResult = await analyzeBundle("lazy-check", lazyLoadScenarios[0].code, true);
  const codecResult = await analyzeBundle("codec-check", "../dist/codec/index.js");

  if (!fullResult.error && !lazyResult.error && !codecResult.error) {
    console.log(`Full library:                    ${formatBytes(fullResult.gzip)} gzipped`);
    console.log(`Without codec (initial load):    ${formatBytes(lazyResult.gzip)} gzipped`);
    console.log(`Codec (lazy loaded):             ${formatBytes(codecResult.gzip)} gzipped`);
    console.log("");
    console.log(
      `Savings on initial load: ${formatBytes(fullResult.gzip - lazyResult.gzip)} (${(
        ((fullResult.gzip - lazyResult.gzip) / fullResult.gzip) *
        100
      ).toFixed(1)}%)`
    );
  }

  // Analyze WASM dependency size
  console.log("\n📦 Zstd WASM Dependency (lazy loaded with compression):\n");

  const wasmPath = join(__dirname, "../node_modules/@bokuweb/zstd-wasm/dist/web/zstd.wasm");
  const wasmJsPath = join(__dirname, "../node_modules/@bokuweb/zstd-wasm/dist/web/index.web.js");

  if (existsSync(wasmPath)) {
    const wasmContent = readFileSync(wasmPath);
    const wasmGzipped = gzipSync(wasmContent);

    // Also measure the JS wrapper
    let jsSize = { raw: 0, gzip: 0 };
    try {
      const wasmJsResult = await esbuild.build({
        entryPoints: [wasmJsPath],
        bundle: true,
        minify: true,
        format: "esm",
        write: false,
        platform: "browser",
        target: "es2022",
        external: ["*.wasm"],
      });
      const jsContent = wasmJsResult.outputFiles[0].contents;
      jsSize = { raw: jsContent.length, gzip: gzipSync(jsContent).length };
    } catch (e) {
      // Ignore errors
    }

    console.log("Component".padEnd(25) + "Raw".padStart(12) + "Gzipped".padStart(12));
    console.log("-".repeat(49));
    console.log(`${"WASM binary".padEnd(25)}${formatBytes(wasmContent.length).padStart(12)}${formatBytes(wasmGzipped.length).padStart(12)}`);
    if (jsSize.raw > 0) {
      console.log(`${"JS wrapper".padEnd(25)}${formatBytes(jsSize.raw).padStart(12)}${formatBytes(jsSize.gzip).padStart(12)}`);
      console.log("-".repeat(49));
      console.log(`${"Total (compression)".padEnd(25)}${formatBytes(wasmContent.length + jsSize.raw).padStart(12)}${formatBytes(wasmGzipped.length + jsSize.gzip).padStart(12)}`);
    }

    console.log("\n💡 Note: WASM is only loaded when compression functions are used.");
    console.log("   Users who don't use compression won't download the WASM file.");
  }

  // Clean up
  rmSync(outDir, { recursive: true });

  console.log("\n" + "=".repeat(70));
  console.log("✅ Analysis complete\n");
}

main().catch(console.error);
