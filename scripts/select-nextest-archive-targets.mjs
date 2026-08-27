#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const GROUPS = new Set(["integration-b", "integration-c"]);

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
    if (arg !== "--group" && arg !== "--policy") {
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
    "    --group integration-b|integration-c",
    "    --policy tests/suites/nextest-target-duration-policy.json",
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

export function parseDurationPolicy(policy) {
  if (!policy || policy.schema_version !== 1) {
    fail("duration policy schema_version must be 1");
  }
  if (!Number.isFinite(policy.fallback_seconds) || policy.fallback_seconds <= 0) {
    fail("duration policy fallback_seconds must be a positive number");
  }
  if (!policy.targets || typeof policy.targets !== "object" || Array.isArray(policy.targets)) {
    fail("duration policy targets must be an object");
  }

  const durations = new Map();
  for (const [target, seconds] of Object.entries(policy.targets)) {
    if (!target || !Number.isFinite(seconds) || seconds <= 0) {
      fail(`duration policy target must have a positive duration: ${target}`);
    }
    durations.set(target, seconds);
  }

  return {
    fallbackSeconds: policy.fallback_seconds,
    durations,
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

export function assignIntegrationTargets(targets, policy) {
  const parsedPolicy = parseDurationPolicy(policy);
  const assignments = {
    "integration-b": { estimatedSeconds: 0, targets: [] },
    "integration-c": { estimatedSeconds: 0, targets: [] },
  };

  const weightedTargets = [...new Set(targets)]
    .map((name) => ({
      name,
      seconds: parsedPolicy.durations.get(name) ?? parsedPolicy.fallbackSeconds,
    }))
    .sort((left, right) => right.seconds - left.seconds || left.name.localeCompare(right.name));

  for (const target of weightedTargets) {
    const destination = assignments["integration-b"].estimatedSeconds
      <= assignments["integration-c"].estimatedSeconds
      ? "integration-b"
      : "integration-c";
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
  if (!GROUPS.has(options.group) || !options.policy) {
    fail(usage());
  }

  const metadata = JSON.parse(await readStandardInput());
  const policy = readJson(options.policy, "duration policy");
  const targets = integrationTargetsFromMetadata(metadata, path.join(process.cwd(), "Cargo.toml"));
  const assignments = assignIntegrationTargets(targets, policy);
  const assignment = assignments[options.group];

  process.stderr.write(
    `[NextestTargetDuration] group=${options.group} integration_targets=${targets.length} `
      + `selected_targets=${assignment.targets.length} `
      + `estimated_seconds=${assignment.estimatedSeconds.toFixed(3)}\n`,
  );
  process.stdout.write(`${assignment.targets.join("\n")}\n`);
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    process.stderr.write(`${error.stack ?? error.message}\n`);
    process.exitCode = 1;
  });
}
