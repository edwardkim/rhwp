#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const GROUPS = ["integration-b", "integration-c", "integration-d"];
const TEST_ATTRIBUTE = /^\s*#\s*\[(?:test|case)\b/gm;

function fail(message) {
  throw new Error(`[NextestTargetDuration] ${message}`);
}

function parseArgs(args) {
  const options = {};
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--help") {
      options.help = true;
      continue;
    }
    if (!["--group", "--policy", "--manifest"].includes(arg)) {
      fail(`unknown argument: ${arg}`);
    }
    const value = args[index + 1];
    if (!value || value.startsWith("--")) {
      fail(`${arg} requires a value`);
    }
    options[arg.slice(2)] = value;
    index += 1;
  }
  return options;
}

function usage() {
  return [
    "Usage: cargo metadata --no-deps --format-version 1 |",
    "  node scripts/select-nextest-archive-targets.mjs",
    "    --group integration-b|integration-c|integration-d",
    "    --policy tests/suites/nextest-target-duration-policy.json",
    "    [--manifest tests/suites/manifest.json]",
  ].join(" ");
}

function readJson(filePath, description) {
  try {
    return JSON.parse(fs.readFileSync(filePath, "utf8"));
  } catch (error) {
    fail(`${description} cannot be read: ${filePath}: ${error.message}`);
  }
}

async function readStandardInput() {
  let source = "";
  process.stdin.setEncoding("utf8");
  for await (const chunk of process.stdin) {
    source += chunk;
  }
  return source;
}

function parseDurations(entries, field) {
  const durations = new Map();
  for (const [name, seconds] of Object.entries(entries)) {
    if (!name || !Number.isFinite(seconds) || seconds <= 0) {
      fail(`duration policy ${field} must have a positive duration: ${name}`);
    }
    durations.set(name, seconds);
  }
  return durations;
}

export function parseDurationPolicy(policy) {
  if (policy?.schema_version === 2) {
    if (!Number.isFinite(policy.fallback_seconds_per_test) || policy.fallback_seconds_per_test <= 0) {
      fail("duration policy fallback_seconds_per_test must be a positive number");
    }
    for (const field of ["targets", "cases", "test_cases"]) {
      if (!policy[field] || typeof policy[field] !== "object" || Array.isArray(policy[field])) {
        fail(`duration policy ${field} must be an object`);
      }
    }
    return {
      schemaVersion: 2,
      fallbackSeconds: policy.fallback_seconds_per_test,
      durations: parseDurations(policy.targets, "target"),
      caseDurations: parseDurations(policy.cases, "case"),
    };
  }
  if (!policy || policy.schema_version !== 1) {
    fail("duration policy schema_version must be 1 or 2");
  }
  if (!Number.isFinite(policy.fallback_seconds) || policy.fallback_seconds <= 0) {
    fail("duration policy fallback_seconds must be a positive number");
  }
  if (!policy.targets || typeof policy.targets !== "object" || Array.isArray(policy.targets)) {
    fail("duration policy targets must be an object");
  }
  return {
    schemaVersion: 1,
    // The v1 selector remains byte-for-byte compatible until v2 measurements land.
    // Mutable suite names are excluded by the current-manifest estimate path.
    fallbackSeconds: policy.fallback_seconds,
    durations: parseDurations(policy.targets, "target"),
    caseDurations: new Map(),
  };
}

export function integrationTargetsFromMetadata(metadata, workspaceManifestPath) {
  if (!metadata || !Array.isArray(metadata.packages)) {
    fail("cargo metadata packages are missing");
  }
  const normalizedManifestPath = path.resolve(workspaceManifestPath);
  const workspacePackage = metadata.packages.find(
    (pkg) => path.resolve(pkg.manifest_path) === normalizedManifestPath,
  );
  if (!workspacePackage) {
    fail(`workspace package is missing: ${workspaceManifestPath}`);
  }
  const targets = workspacePackage.targets
    .filter((target) => Array.isArray(target.kind) && target.kind.includes("test"))
    .map((target) => target.name)
    .sort((left, right) => left.localeCompare(right));
  if (new Set(targets).size !== targets.length) {
    fail("cargo metadata contains duplicate integration target names");
  }
  return targets;
}

function caseNameForSource(source) {
  return path.basename(source, ".rs").replaceAll("-", "_");
}

function sourceEstimate(source, parsedPolicy, root) {
  const measured = parsedPolicy.caseDurations.get(caseNameForSource(source));
  if (measured !== undefined) {
    return measured;
  }
  const text = fs.readFileSync(path.join(root, source), "utf8");
  const tests = (text.match(TEST_ATTRIBUTE) ?? []).length;
  return Math.max(1, tests) * parsedPolicy.fallbackSeconds;
}

export function estimateManifestTargetDurations(manifest, policy, root = process.cwd()) {
  const parsedPolicy = policy?.caseDurations ? policy : parseDurationPolicy(policy);
  const estimates = new Map();
  for (const [target, sources] of Object.entries(manifest.suites ?? {})) {
    estimates.set(target, sources.reduce(
      (total, source) => total + sourceEstimate(source, parsedPolicy, root),
      0,
    ));
  }
  for (const exception of manifest.exceptions ?? []) {
    estimates.set(exception.target, sourceEstimate(exception.path, parsedPolicy, root));
  }
  return estimates;
}

export function assignIntegrationTargets(targets, policy, targetEstimates = new Map()) {
  const parsedPolicy = policy?.caseDurations ? policy : parseDurationPolicy(policy);
  const assignments = Object.fromEntries(
    GROUPS.map((group) => [group, { estimatedSeconds: 0, targets: [] }]),
  );
  const weightedTargets = [...new Set(targets)]
    .map((name) => ({
      name,
      seconds: targetEstimates.get(name) ?? parsedPolicy.durations.get(name) ?? parsedPolicy.fallbackSeconds,
    }))
    .sort((left, right) => right.seconds - left.seconds || left.name.localeCompare(right.name));
  for (const target of weightedTargets) {
    const destination = GROUPS.reduce((shortest, group) => (
      assignments[group].estimatedSeconds < assignments[shortest].estimatedSeconds
      || (
        assignments[group].estimatedSeconds === assignments[shortest].estimatedSeconds
        && group.localeCompare(shortest) < 0
      )
        ? group
        : shortest
    ));
    assignments[destination].targets.push(target.name);
    assignments[destination].estimatedSeconds += target.seconds;
  }
  for (const assignment of Object.values(assignments)) {
    assignment.targets.sort((left, right) => left.localeCompare(right));
  }
  return assignments;
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  if (options.help) {
    process.stdout.write(`${usage()}\n`);
    return;
  }
  if (!GROUPS.includes(options.group) || !options.policy) {
    fail(usage());
  }
  const metadata = JSON.parse(await readStandardInput());
  const policy = parseDurationPolicy(readJson(options.policy, "duration policy"));
  const targets = integrationTargetsFromMetadata(metadata, path.join(process.cwd(), "Cargo.toml"));
  const targetEstimates = options.manifest
    ? estimateManifestTargetDurations(readJson(options.manifest, "derived suite manifest"), policy)
    : new Map();
  const assignment = assignIntegrationTargets(targets, policy, targetEstimates)[options.group];
  process.stderr.write(
    `[NextestTargetDuration] group=${options.group} integration_targets=${targets.length} `
      + `selected_targets=${assignment.targets.length} `
      + `estimated_seconds=${assignment.estimatedSeconds.toFixed(3)} `
      + `duration_model=${options.manifest ? `current-manifest-v${policy.schemaVersion}` : `target-v${policy.schemaVersion}`}\n`,
  );
  process.stdout.write(`${assignment.targets.join("\n")}\n`);
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    process.stderr.write(`${error.stack ?? error.message}\n`);
    process.exitCode = 1;
  });
}
