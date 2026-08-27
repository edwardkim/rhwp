#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { parseDurationPolicy } from "./select-nextest-archive-targets.mjs";

function fail(message) {
  throw new Error(`[NextestTargetDuration] ${message}`);
}

function parseArgs(args) {
  const options = { measurements: [] };
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--help") {
      options.help = true;
      continue;
    }
    if (!["--policy", "--measurement", "--output"].includes(arg)) {
      fail(`unknown argument: ${arg}`);
    }
    const value = args[index + 1];
    if (!value || value.startsWith("--")) {
      fail(`${arg} requires a value`);
    }
    if (arg === "--measurement") {
      options.measurements.push(value);
    } else {
      options[arg.slice(2)] = value;
    }
    index += 1;
  }
  return options;
}

export function refreshDurationPolicy(policy, measurements) {
  parseDurationPolicy(policy);
  const durations = { ...policy.targets };
  for (const measurement of measurements) {
    if (!measurement || measurement.schema_version !== 1 || !["b", "c"].includes(measurement.archive_label)) {
      fail("measurement must be a B or C schema_version 1 report");
    }
    if (!measurement.targets || typeof measurement.targets !== "object" || Array.isArray(measurement.targets)) {
      fail("measurement targets must be an object");
    }
    for (const [target, seconds] of Object.entries(measurement.targets)) {
      if (!target || !Number.isFinite(seconds) || seconds <= 0) {
        fail(`measurement target must have a positive duration: ${target}`);
      }
      durations[target] = seconds;
    }
  }

  return {
    schema_version: 1,
    fallback_seconds: policy.fallback_seconds,
    measurement_sources: Object.fromEntries(measurements
      .map((measurement) => [measurement.archive_label, {
        run_id: measurement.run_id ?? null,
        ref: measurement.ref ?? null,
        sha: measurement.sha ?? null,
      }])
      .sort(([left], [right]) => left.localeCompare(right))),
    targets: Object.fromEntries(Object.entries(durations).sort(([left], [right]) => left.localeCompare(right))),
  };
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  if (options.help) {
    process.stdout.write("Usage: node scripts/refresh-nextest-target-duration-policy.mjs --policy policy.json --measurement b.json --measurement c.json --output policy.json\n");
    return;
  }
  if (!options.policy || !options.output || options.measurements.length !== 2) {
    fail("--policy, exactly two --measurement values, and --output are required");
  }
  const policy = JSON.parse(fs.readFileSync(options.policy, "utf8"));
  const measurements = options.measurements.map((filePath) => JSON.parse(fs.readFileSync(filePath, "utf8")));
  const labels = new Set(measurements.map((measurement) => measurement.archive_label));
  if (labels.size !== 2 || !labels.has("b") || !labels.has("c")) {
    fail("measurements must contain exactly one B report and one C report");
  }
  const refreshed = refreshDurationPolicy(policy, measurements);
  fs.mkdirSync(path.dirname(options.output), { recursive: true });
  fs.writeFileSync(options.output, `${JSON.stringify(refreshed, null, 2)}\n`);
  process.stderr.write(`[NextestTargetDuration] refreshed_targets=${Object.keys(refreshed.targets).length}\n`);
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}
