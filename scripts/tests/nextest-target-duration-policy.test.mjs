import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

import {
  assignIntegrationTargets,
  integrationTargetsFromMetadata,
} from "../select-nextest-archive-targets.mjs";
import { collectTargetDurations } from "../collect-nextest-target-durations.mjs";
import { refreshDurationPolicy } from "../refresh-nextest-target-duration-policy.mjs";

const selectorPath = fileURLToPath(new URL("../select-nextest-archive-targets.mjs", import.meta.url));
const policyPath = fileURLToPath(new URL("../../tests/suites/nextest-target-duration-policy.json", import.meta.url));
const repoRoot = fileURLToPath(new URL("../../", import.meta.url));

const policy = {
  schema_version: 1,
  fallback_seconds: 1,
  targets: {
    slow: 9,
    medium: 6,
    fast: 1,
  },
};

test("duration-aware assignment is deterministic and balances three estimated loads", () => {
  const first = assignIntegrationTargets(["fast", "slow", "medium", "new"], policy);
  const second = assignIntegrationTargets(["new", "medium", "fast", "slow"], policy);

  assert.deepEqual(first, second);
  assert.deepEqual(first["integration-b"].targets, ["slow"]);
  assert.deepEqual(first["integration-c"].targets, ["medium"]);
  assert.deepEqual(first["integration-d"].targets, ["fast", "new"]);
  assert.equal(first["integration-b"].estimatedSeconds, 9);
  assert.equal(first["integration-c"].estimatedSeconds, 6);
  assert.equal(first["integration-d"].estimatedSeconds, 2);
  assert.deepEqual(
    Object.values(first).flatMap((assignment) => assignment.targets).sort(),
    ["fast", "medium", "new", "slow"],
  );
});

test("empty duration profile preserves stable alternating bootstrap assignment", () => {
  const assignments = assignIntegrationTargets(["delta", "alpha", "charlie", "bravo"], {
    schema_version: 1,
    fallback_seconds: 1,
    targets: {},
  });

  assert.deepEqual(assignments["integration-b"].targets, ["alpha", "delta"]);
  assert.deepEqual(assignments["integration-c"].targets, ["bravo"]);
  assert.deepEqual(assignments["integration-d"].targets, ["charlie"]);
});

test("metadata selection uses only root integration targets", () => {
  const targets = integrationTargetsFromMetadata({
    packages: [
      {
        manifest_path: "/repo/Cargo.toml",
        targets: [
          { name: "root-lib", kind: ["lib"] },
          { name: "case-b", kind: ["test"] },
          { name: "case-a", kind: ["test"] },
        ],
      },
      {
        manifest_path: "/repo/crates/helper/Cargo.toml",
        targets: [{ name: "helper-case", kind: ["test"] }],
      },
    ],
  }, "/repo/Cargo.toml");

  assert.deepEqual(targets, ["case-a", "case-b"]);
});

test("CLI consumes streamed cargo metadata without synchronous stdin reads", () => {
  const result = spawnSync(process.execPath, [
    selectorPath,
    "--group", "integration-b",
    "--policy", policyPath,
  ], {
    cwd: repoRoot,
    encoding: "utf8",
    input: JSON.stringify({
      packages: [{
        manifest_path: path.join(repoRoot, "Cargo.toml"),
        targets: [
          { name: "case-b", kind: ["test"] },
          { name: "case-a", kind: ["test"] },
        ],
      }],
    }),
  });

  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, "case-a\n");
  assert.match(result.stderr, /integration_targets=2 selected_targets=1/);
});

test("JUnit collection aggregates testcase durations per binary and skips setup suites", () => {
  const durations = collectTargetDurations([
    '<testsuite name="rhwp::issue_1">',
    '<testcase name="first" classname="rhwp::issue_1" time="1.25" />',
    '<testcase name="setup" classname="@setup-script:seed" time="5.00" />',
    '<testcase name="second" classname="rhwp::issue_1" time="0.75" />',
    '<testcase name="third" classname="rhwp::issue_2" time="2.00" />',
  ].join("\n"));

  assert.deepEqual(durations, { issue_1: 2, issue_2: 2 });
});

test("JUnit measurement retains individual testcase and source-case durations", async () => {
  const { collectDurationMeasurement } = await import("../collect-nextest-target-durations.mjs");
  const measurement = collectDurationMeasurement([
    '<testcase name="security_corpus_regression::negative_sweep" classname="rhwp::regression_suite_015" time="9.25" />',
    '<testcase name="security_corpus_regression::positive_sweep" classname="rhwp::regression_suite_015" time="0.75" />',
    '<testcase name="standalone" classname="rhwp::issue_1" time="2.00" />',
  ].join("\n"));

  assert.deepEqual(measurement.targets, { issue_1: 2, regression_suite_015: 10 });
  assert.deepEqual(measurement.cases, { issue_1: 2, security_corpus_regression: 10 });
  assert.deepEqual(measurement.test_cases, {
    "issue_1::standalone": 2,
    "regression_suite_015::security_corpus_regression::negative_sweep": 9.25,
    "regression_suite_015::security_corpus_regression::positive_sweep": 0.75,
  });
});

test("v2 policy uses current suite source composition instead of a historical suite name", async () => {
  const { estimateManifestTargetDurations } = await import("../select-nextest-archive-targets.mjs");
  const fs = await import("node:fs");
  const os = await import("node:os");
  const path = await import("node:path");
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "rhwp-duration-manifest-"));
  try {
    fs.mkdirSync(path.join(root, "tests", "cases"), { recursive: true });
    fs.writeFileSync(path.join(root, "tests", "cases", "heavy.rs"), "#[test] fn one() {}\n");
    fs.writeFileSync(path.join(root, "tests", "cases", "light.rs"), "#[test] fn one() {}\n");
    const estimates = estimateManifestTargetDurations({
      suites: { regression_suite_001: ["tests/cases/heavy.rs", "tests/cases/light.rs"] },
      exceptions: [],
    }, {
      schema_version: 2,
      fallback_seconds_per_test: 60,
      targets: { regression_suite_001: 1 },
      cases: { heavy: 800, light: 5 },
      test_cases: {},
    }, root);
    assert.equal(estimates.get("regression_suite_001"), 805);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("policy refresh accepts one successful B, C, and D measurement", () => {
  const refreshed = refreshDurationPolicy({
    schema_version: 1,
    fallback_seconds: 1,
    targets: { existing: 3 },
  }, [
    { schema_version: 1, archive_label: "b", run_id: "10", ref: "refs/heads/devel", sha: "same-sha", targets: { beta: 4 } },
    { schema_version: 1, archive_label: "c", run_id: "10", ref: "refs/heads/devel", sha: "same-sha", targets: { alpha: 2 } },
    { schema_version: 1, archive_label: "d", run_id: "10", ref: "refs/heads/devel", sha: "same-sha", targets: { delta: 1 } },
  ]);

  assert.deepEqual(refreshed.targets, { alpha: 2, beta: 4, delta: 1, existing: 3 });
  assert.deepEqual(refreshed.measurement_sources, {
    b: { run_id: "10", ref: "refs/heads/devel", sha: "same-sha" },
    c: { run_id: "10", ref: "refs/heads/devel", sha: "same-sha" },
    d: { run_id: "10", ref: "refs/heads/devel", sha: "same-sha" },
  });
});

test("v2 policy keeps its source-case model when a trusted v1 artifact is reused", () => {
  const policy = {
    schema_version: 2,
    fallback_seconds_per_test: 60,
    targets: {},
    cases: {},
    test_cases: {},
  };
  const measurements = ["b", "c", "d"].map((archive_label) => ({
    schema_version: 1,
    archive_label,
    run_id: "10",
    ref: "refs/heads/devel",
    sha: "same-sha",
    targets: { regression_suite_001: 4 },
  }));
  assert.deepEqual(refreshDurationPolicy(policy, measurements), policy);
});

test("policy refresh rejects empty or mismatched B/C/D measurements", () => {
  const basePolicy = { schema_version: 1, fallback_seconds: 1, targets: {} };
  assert.throws(
    () => refreshDurationPolicy(basePolicy, [
      { schema_version: 1, archive_label: "b", run_id: "10", ref: "refs/heads/devel", sha: "same-sha", targets: {} },
      { schema_version: 1, archive_label: "c", run_id: "10", ref: "refs/heads/devel", sha: "same-sha", targets: { alpha: 2 } },
      { schema_version: 1, archive_label: "d", run_id: "10", ref: "refs/heads/devel", sha: "same-sha", targets: { delta: 1 } },
    ]),
    /must contain target durations: b/,
  );
  assert.throws(
    () => refreshDurationPolicy(basePolicy, [
      { schema_version: 1, archive_label: "b", run_id: "10", ref: "refs/heads/devel", sha: "first", targets: { beta: 4 } },
      { schema_version: 1, archive_label: "c", run_id: "10", ref: "refs/heads/devel", sha: "second", targets: { alpha: 2 } },
      { schema_version: 1, archive_label: "d", run_id: "10", ref: "refs/heads/devel", sha: "first", targets: { delta: 1 } },
    ]),
    /identical run, ref, and sha provenance/,
  );
});
