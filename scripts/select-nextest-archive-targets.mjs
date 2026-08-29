#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const GROUPS = ["integration-b", "integration-c", "integration-d"];
const TEST_ATTRIBUTE = /^\s*#\s*\[(?:test|case)\b/gm;
const DEFAULT_V2_PARALLELISM_FACTOR = 4;

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

function caseNameForTestCaseKey(testCase) {
  const parts = String(testCase).split("::");
  if (parts.length === 0 || !parts[0]) {
    fail(`duration policy test_case must include a target name: ${testCase}`);
  }
  if (parts[0].startsWith("regression_suite_")) {
    if (!parts[1]) {
      fail(`duration policy grouped test_case must include its source module: ${testCase}`);
    }
    return parts[1];
  }
  return parts[0];
}

function deriveCaseMaxDurations(testCaseDurations) {
  const caseMaxDurations = new Map();
  for (const [testCase, seconds] of testCaseDurations.entries()) {
    const caseName = caseNameForTestCaseKey(testCase);
    caseMaxDurations.set(
      caseName,
      Math.max(caseMaxDurations.get(caseName) ?? 0, seconds),
    );
  }
  return caseMaxDurations;
}

function parseParallelismFactor(policy) {
  if (policy.parallelism_factor === undefined) {
    return DEFAULT_V2_PARALLELISM_FACTOR;
  }
  if (!Number.isFinite(policy.parallelism_factor) || policy.parallelism_factor <= 0) {
    fail("duration policy parallelism_factor must be a positive number");
  }
  return policy.parallelism_factor;
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
    const testCaseDurations = parseDurations(policy.test_cases, "test_case");
    return {
      schemaVersion: 2,
      fallbackSeconds: policy.fallback_seconds_per_test,
      parallelismFactor: parseParallelismFactor(policy),
      durations: parseDurations(policy.targets, "target"),
      caseDurations: parseDurations(policy.cases, "case"),
      testCaseDurations,
      caseMaxDurations: deriveCaseMaxDurations(testCaseDurations),
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
    parallelismFactor: 1,
    durations: parseDurations(policy.targets, "target"),
    caseDurations: new Map(),
    testCaseDurations: new Map(),
    caseMaxDurations: new Map(),
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

function sourceRuntimeProfile(source, parsedPolicy, root) {
  const caseName = caseNameForSource(source);
  const measured = parsedPolicy.caseDurations.get(caseName);
  if (measured !== undefined) {
    return {
      estimatedSeconds: measured,
      maxTestcaseSeconds: parsedPolicy.caseMaxDurations.get(caseName) ?? measured,
    };
  }
  const text = fs.readFileSync(path.join(root, source), "utf8");
  const tests = (text.match(TEST_ATTRIBUTE) ?? []).length;
  return {
    estimatedSeconds: Math.max(1, tests) * parsedPolicy.fallbackSeconds,
    maxTestcaseSeconds: parsedPolicy.fallbackSeconds,
  };
}

function combineRuntimeProfiles(profiles) {
  return profiles.reduce(
    (combined, profile) => ({
      estimatedSeconds: combined.estimatedSeconds + profile.estimatedSeconds,
      maxTestcaseSeconds: Math.max(combined.maxTestcaseSeconds, profile.maxTestcaseSeconds),
    }),
    { estimatedSeconds: 0, maxTestcaseSeconds: 0 },
  );
}

export function estimateManifestTargetDurations(manifest, policy, root = process.cwd()) {
  return new Map(
    [...estimateManifestTargetRuntimeProfiles(manifest, policy, root).entries()]
      .map(([target, profile]) => [target, profile.estimatedSeconds]),
  );
}

export function estimateManifestTargetRuntimeProfiles(manifest, policy, root = process.cwd()) {
  const parsedPolicy = policy?.caseDurations ? policy : parseDurationPolicy(policy);
  const estimates = new Map();
  for (const [target, sources] of Object.entries(manifest.suites ?? {})) {
    estimates.set(
      target,
      combineRuntimeProfiles(sources.map((source) => sourceRuntimeProfile(source, parsedPolicy, root))),
    );
  }
  for (const exception of manifest.exceptions ?? []) {
    estimates.set(exception.target, sourceRuntimeProfile(exception.path, parsedPolicy, root));
  }
  return estimates;
}

function normalizeTargetProfile(name, parsedPolicy, targetEstimates) {
  const estimate = targetEstimates.get(name);
  if (typeof estimate === "number") {
    return { estimatedSeconds: estimate, maxTestcaseSeconds: estimate };
  }
  if (estimate && typeof estimate === "object") {
    const estimatedSeconds = estimate.estimatedSeconds ?? estimate.seconds;
    const maxTestcaseSeconds = estimate.maxTestcaseSeconds ?? estimate.maxSeconds ?? estimatedSeconds;
    if (Number.isFinite(estimatedSeconds) && estimatedSeconds > 0) {
      return {
        estimatedSeconds,
        maxTestcaseSeconds: Number.isFinite(maxTestcaseSeconds) && maxTestcaseSeconds > 0
          ? maxTestcaseSeconds
          : estimatedSeconds,
      };
    }
  }
  const fallback = parsedPolicy.durations.get(name) ?? parsedPolicy.fallbackSeconds;
  return { estimatedSeconds: fallback, maxTestcaseSeconds: fallback };
}

function estimatedWallSeconds(estimatedSeconds, maxTestcaseSeconds, parallelismFactor) {
  return Math.max(estimatedSeconds / parallelismFactor, maxTestcaseSeconds);
}

function assignmentSpread(values) {
  return Math.max(...values) - Math.min(...values);
}

function projectedAssignmentScore(assignments, group, target, parsedPolicy) {
  const assignment = assignments[group];
  const candidateSeconds = assignment.estimatedSeconds + target.estimatedSeconds;
  const candidateMax = Math.max(assignment.maxTestcaseSeconds, target.maxTestcaseSeconds);
  const candidateWall = estimatedWallSeconds(
    candidateSeconds,
    candidateMax,
    parsedPolicy.parallelismFactor,
  );
  const projectedWallSeconds = GROUPS.map((current) => (
    current === group ? candidateWall : assignments[current].estimatedWallSeconds
  ));
  const projectedEstimatedSeconds = GROUPS.map((current) => (
    current === group ? candidateSeconds : assignments[current].estimatedSeconds
  ));
  return {
    candidateSeconds,
    candidateMax,
    candidateWall,
    projectedMaxWallSeconds: Math.max(...projectedWallSeconds),
    projectedEstimatedSecondsSpread: assignmentSpread(projectedEstimatedSeconds),
  };
}

export function assignIntegrationTargets(targets, policy, targetEstimates = new Map()) {
  const parsedPolicy = policy?.caseDurations ? policy : parseDurationPolicy(policy);
  const assignments = Object.fromEntries(
    GROUPS.map((group) => [group, {
      estimatedSeconds: 0,
      estimatedWallSeconds: 0,
      maxTestcaseSeconds: 0,
      targets: [],
    }]),
  );
  const weightedTargets = [...new Set(targets)]
    .map((name) => ({
      name,
      ...normalizeTargetProfile(name, parsedPolicy, targetEstimates),
    }))
    .sort((left, right) => (
      right.maxTestcaseSeconds - left.maxTestcaseSeconds
      || right.estimatedSeconds - left.estimatedSeconds
      || left.name.localeCompare(right.name)
    ));
  for (const target of weightedTargets) {
    const destination = GROUPS.reduce((best, group) => {
      const candidate = projectedAssignmentScore(assignments, group, target, parsedPolicy);
      const bestCandidate = projectedAssignmentScore(assignments, best, target, parsedPolicy);
      return (
        candidate.projectedMaxWallSeconds < bestCandidate.projectedMaxWallSeconds
        || (
          candidate.projectedMaxWallSeconds === bestCandidate.projectedMaxWallSeconds
          && (
            candidate.projectedEstimatedSecondsSpread < bestCandidate.projectedEstimatedSecondsSpread
            || (
              candidate.projectedEstimatedSecondsSpread === bestCandidate.projectedEstimatedSecondsSpread
              && (
                candidate.candidateWall < bestCandidate.candidateWall
                || (
                  candidate.candidateWall === bestCandidate.candidateWall
                  && group.localeCompare(best) < 0
                )
              )
            )
          )
        )
      )
        ? group
        : best;
    });
    assignments[destination].targets.push(target.name);
    assignments[destination].estimatedSeconds += target.estimatedSeconds;
    assignments[destination].maxTestcaseSeconds = Math.max(
      assignments[destination].maxTestcaseSeconds,
      target.maxTestcaseSeconds,
    );
    assignments[destination].estimatedWallSeconds = estimatedWallSeconds(
      assignments[destination].estimatedSeconds,
      assignments[destination].maxTestcaseSeconds,
      parsedPolicy.parallelismFactor,
    );
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
    ? estimateManifestTargetRuntimeProfiles(readJson(options.manifest, "derived suite manifest"), policy)
    : new Map();
  const assignment = assignIntegrationTargets(targets, policy, targetEstimates)[options.group];
  process.stderr.write(
    `[NextestTargetDuration] group=${options.group} integration_targets=${targets.length} `
      + `selected_targets=${assignment.targets.length} `
      + `estimated_seconds=${assignment.estimatedSeconds.toFixed(3)} `
      + `estimated_wall_seconds=${assignment.estimatedWallSeconds.toFixed(3)} `
      + `max_testcase_seconds=${assignment.maxTestcaseSeconds.toFixed(3)} `
      + `parallelism_factor=${policy.parallelismFactor} `
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
