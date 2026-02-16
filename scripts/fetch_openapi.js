#!/usr/bin/env node

/**
 * Fetches the OpenAPI specification from the running backend.
 * Falls back to the local openapi.json if the backend is unreachable.
 *
 * Environment variables:
 *   BACKEND_OPENAPI_URL - URL to fetch the spec from
 *                         (default: http://127.0.0.1:3000/docs/openapi.json)
 */

import { writeFileSync, existsSync, readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const OUTPUT_PATH = resolve(__dirname, "../frontend/openapi.json");

const BACKEND_OPENAPI_URL =
  process.env.BACKEND_OPENAPI_URL ||
  "http://127.0.0.1:3000/api-doc/openapi.json";

async function fetchFromBackend() {
  console.log(`⬇  Fetching OpenAPI spec from ${BACKEND_OPENAPI_URL} …`);
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 5000);

  try {
    const res = await fetch(BACKEND_OPENAPI_URL, {
      signal: controller.signal,
    });
    clearTimeout(timeout);

    if (!res.ok) {
      throw new Error(`HTTP ${res.status} ${res.statusText}`);
    }

    const json = await res.json();
    const pretty = JSON.stringify(json, null, 4);
    writeFileSync(OUTPUT_PATH, pretty + "\n", "utf-8");
    console.log(`✅ OpenAPI spec saved to ${OUTPUT_PATH}`);
    return true;
  } catch (err) {
    clearTimeout(timeout);
    console.warn(`⚠  Could not fetch from backend: ${err.message}`);
    return false;
  }
}

function fallbackToLocal() {
  if (existsSync(OUTPUT_PATH)) {
    console.log(`ℹ  Using existing local ${OUTPUT_PATH} as fallback.`);
    // Validate it's parseable JSON
    try {
      JSON.parse(readFileSync(OUTPUT_PATH, "utf-8"));
      return true;
    } catch {
      console.error(`❌ Local ${OUTPUT_PATH} is not valid JSON.`);
      return false;
    }
  }
  console.error(`❌ No local fallback found at ${OUTPUT_PATH}.`);
  return false;
}

async function main() {
  const fetched = await fetchFromBackend();
  if (!fetched) {
    const ok = fallbackToLocal();
    if (!ok) {
      process.exit(1);
    }
  }
}

main();
