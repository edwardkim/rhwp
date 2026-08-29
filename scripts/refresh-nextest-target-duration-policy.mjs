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
  if (policy.schema_version === 2) {
    if (
      !Number.isFinite(policy.fallback_seconds_per_test)
      || policy.fallback_seconds_per_test <= 0
      || !policy.cases
      || typeof policy.cases !== "object"
      || Array.isArray(policy.cases)
      || !policy.test_cases
      || typeof policy.test_cases !== "object"
      || Array.isArray(policy.test_cases)
    ) {
      fail("schema_version 2 policy requires fallback_seconds_per_test, cases, and test_cases");
    }
    // A trusted review-tail may have completed before this schema migration.
    // Its artifact provenance is still valid for post-merge reuse, but it has
    // no source-case detail. Keep the v2 policy unchanged rather than making
    // post-merge processing fail or reviving mutable suite-name measurements.
    if (measurements.every((measurement) => measurement?.schema_version === 1)) {
      return policy;
    }
    const parsedPolicy = parseDurationPolicy(policy);
    const targets = {};
    const cases = {};
    const testCases = {};
    const sourceKeys = new Set();
    for (const measurement of measurements) {
      if (!measurement || measurement.schema_version !== 2 || !["b", "c", "d"].includes(measurement.archive_label)) {
        fail("measurement must be a B, C, or D schema_version 2 report");
      }
      for (const [field, destination] of [["targets", targets], ["cases", cases], ["test_cases", testCases]]) {
        if (!measurement[field] || typeof measurement[field] !== "object" || Array.isArray(measurement[field])) {
          fail(`measurement ${field} must be an object: ${measurement.archive_label}`);
        }
        for (const [name, seconds] of Object.entries(measurement[field])) {
          if (!name || !Number.isFinite(seconds) || seconds <= 0) {
            fail(`measurement ${field} must have a positive duration: ${name}`);
          }
          destination[name] = seconds;
        }
      }
      if (Object.keys(measurement.targets).length === 0 || Object.keys(measurement.cases).length === 0) {
        fail(`measurement must contain target and case durations: ${measurement.archive_label}`);
      }
      const source = [measurement.run_id, measurement.ref, measurement.sha];
      if (!source.every((value) => typeof value === "string" && value.length > 0)) {
        fail(`measurement must identify its run, ref, and sha: ${measurement.archive_label}`);
      }
      sourceKeys.add(JSON.stringify(source));
    }
    if (sourceKeys.size !== 1) {
      fail("B, C, and D measurements must have identical run, ref, and sha provenance");
    }
    return {
      schema_version: 2,
      fallback_seconds_per_test: policy.fallback_seconds_per_test,
      parallelism_factor: parsedPolicy.parallelismFactor,
      measurement_sources: Object.fromEntries(measurements
        .map((measurement) => [measurement.archive_label, {
          run_id: measurement.run_id,
          ref: measurement.ref,
          sha: measurement.sha,
        }])
        .sort(([left], [right]) => left.localeCompare(right))),
      targets: Object.fromEntries(Object.entries(targets).sort(([left], [right]) => left.localeCompare(right))),
      cases: Object.fromEntries(Object.entries(cases).sort(([left], [right]) => left.localeCompare(right))),
      test_cases: Object.fromEntries(Object.entries(testCases).sort(([left], [right]) => left.localeCompare(right))),
    };
  }
  parseDurationPolicy(policy);
  const durations = { ...policy.targets };
  const sourceKeys = new Set();
  for (const measurement of measurements) {
    if (!measurement || measurement.schema_version !== 1 || !["b", "c", "d"].includes(measurement.archive_label)) {
      fail("measurement must be a B, C, or D schema_version 1 report");
    }
    if (!measurement.targets || typeof measurement.targets !== "object" || Array.isArray(measurement.targets)) {
      fail("measurement targets must be an object");
    }
    const entries = Object.entries(measurement.targets);
    if (entries.length === 0) {
      fail(`measurement must contain target durations: ${measurement.archive_label}`);
    }
    const source = [measurement.run_id, measurement.ref, measurement.sha];
    if (!source.every((value) => typeof value === "string" && value.length > 0)) {
      fail(`measurement must identify its run, ref, and sha: ${measurement.archive_label}`);
    }
    sourceKeys.add(JSON.stringify(source));
    for (const [target, seconds] of entries) {
      if (!target || !Number.isFinite(seconds) || seconds <= 0) {
        fail(`measurement target must have a positive duration: ${target}`);
      }
      durations[target] = seconds;
    }
  }
  if (sourceKeys.size !== 1) {
    fail("B, C, and D measurements must have identical run, ref, and sha provenance");
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
    process.stdout.write("Usage: node scripts/refresh-nextest-target-duration-policy.mjs --policy policy.json --measurement b.json --measurement c.json --measurement d.json --output policy.json\n");
    return;
  }
  if (!options.policy || !options.output || options.measurements.length !== 3) {
    fail("--policy, exactly three --measurement values, and --output are required");
  }
  const policy = JSON.parse(fs.readFileSync(options.policy, "utf8"));
  const measurements = options.measurements.map((filePath) => JSON.parse(fs.readFileSync(filePath, "utf8")));
  const labels = new Set(measurements.map((measurement) => measurement.archive_label));
  if (labels.size !== 3 || !labels.has("b") || !labels.has("c") || !labels.has("d")) {
    fail("measurements must contain exactly one B, C, and D report");
  }
  const refreshed = refreshDurationPolicy(policy, measurements);
  fs.mkdirSync(path.dirname(options.output), { recursive: true });
  fs.writeFileSync(options.output, `${JSON.stringify(refreshed, null, 2)}\n`);
  process.stderr.write(`[NextestTargetDuration] refreshed_targets=${Object.keys(refreshed.targets).length}\n`);
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}
