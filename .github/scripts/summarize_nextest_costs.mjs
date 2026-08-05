#!/usr/bin/env node

import fs from "node:fs";
import { makeCostSlice, summarizeNextestEvents } from "./nextest_cost_model.mjs";

function fail(message) {
  throw new Error(message);
}

const args = new Map();
for (let index = 2; index < process.argv.length; index += 2) {
  const key = process.argv[index];
  const value = process.argv[index + 1];
  if (!key?.startsWith("--") || value === undefined) {
    fail("usage: summarize_nextest_costs.mjs --events FILE --archive-label LABEL --output FILE");
  }
  args.set(key, value);
}

const eventsPath = args.get("--events");
const archiveLabel = args.get("--archive-label");
const outputPath = args.get("--output");
if (!eventsPath || !archiveLabel || !outputPath || args.size !== 3) {
  fail("usage: summarize_nextest_costs.mjs --events FILE --archive-label LABEL --output FILE");
}

const targets = summarizeNextestEvents(fs.readFileSync(eventsPath, "utf8"));
const slice = makeCostSlice({ archiveLabel, targets });
fs.writeFileSync(outputPath, `${JSON.stringify(slice, null, 2)}\n`);
console.log(`archive=${archiveLabel} cost_targets=${targets.size}`);
