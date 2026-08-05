#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { mergeCostSlices, readCostModel } from "./nextest_cost_model.mjs";

function fail(message) {
  throw new Error(message);
}

const args = new Map();
for (let index = 2; index < process.argv.length; index += 2) {
  const key = process.argv[index];
  const value = process.argv[index + 1];
  if (!key?.startsWith("--") || value === undefined) {
    fail("usage: merge_nextest_cost_model.mjs --input-dir DIR --prior FILE --output FILE");
  }
  args.set(key, value);
}

const inputDir = args.get("--input-dir");
const priorPath = args.get("--prior");
const outputPath = args.get("--output");
if (!inputDir || !priorPath || !outputPath || args.size !== 3) {
  fail("usage: merge_nextest_cost_model.mjs --input-dir DIR --prior FILE --output FILE");
}

const slices = fs.readdirSync(inputDir)
  .filter((name) => name.endsWith(".json"))
  .sort()
  .map((name) => JSON.parse(fs.readFileSync(path.join(inputDir, name), "utf8")));
const labels = new Set(slices.map((slice) => slice.archive_label));
for (const label of ["slow", "1", "2", "3"]) {
  if (!labels.has(label)) fail(`missing nextest cost slice: ${label}`);
}
if (labels.size !== 4 || slices.length !== 4) fail("expected exactly four nextest cost slices");

const model = mergeCostSlices({ priorModel: readCostModel(priorPath), slices });
fs.mkdirSync(path.dirname(outputPath), { recursive: true });
fs.writeFileSync(outputPath, `${JSON.stringify(model, null, 2)}\n`);
console.log(`model_targets=${Object.keys(model.targets).length} fallback_run_ms=${model.fallback_run_ms}`);
