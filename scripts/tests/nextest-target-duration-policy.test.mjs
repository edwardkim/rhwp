import assert from "node:assert/strict";
import test from "node:test";

import {
  assignIntegrationTargets,
  integrationTargetsFromMetadata,
} from "../select-nextest-archive-targets.mjs";
import { collectTargetDurations } from "../collect-nextest-target-durations.mjs";
import { refreshDurationPolicy } from "../refresh-nextest-target-duration-policy.mjs";

const policy = {
  schema_version: 1,
  fallback_seconds: 1,
  targets: {
    slow: 9,
    medium: 6,
    fast: 1,
  },
};

test("duration-aware assignment is deterministic and balances estimated load", () => {
  const first = assignIntegrationTargets(["fast", "slow", "medium", "new"], policy);
  const second = assignIntegrationTargets(["new", "medium", "fast", "slow"], policy);

  assert.deepEqual(first, second);
  assert.deepEqual(first["integration-b"].targets, ["slow"]);
  assert.deepEqual(first["integration-c"].targets, ["fast", "medium", "new"]);
  assert.equal(first["integration-b"].estimatedSeconds, 9);
  assert.equal(first["integration-c"].estimatedSeconds, 8);
});

test("empty duration profile preserves stable alternating bootstrap assignment", () => {
  const assignments = assignIntegrationTargets(["delta", "alpha", "charlie", "bravo"], {
    schema_version: 1,
    fallback_seconds: 1,
    targets: {},
  });

  assert.deepEqual(assignments["integration-b"].targets, ["alpha", "charlie"]);
  assert.deepEqual(assignments["integration-c"].targets, ["bravo", "delta"]);
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

test("JUnit collection aggregates one duration per test binary and skips setup suites", () => {
  const durations = collectTargetDurations([
    '<testsuite name="rhwp::issue_1" time="1.25">',
    '<testsuite name="@setup-script:seed" time="5.00">',
    '<testsuite name="rhwp::issue_1" time="0.75">',
    '<testsuite name="rhwp::issue_2" time="2.00">',
  ].join("\n"));

  assert.deepEqual(durations, { issue_1: 2, issue_2: 2 });
});

test("policy refresh accepts one successful B and C measurement", () => {
  const refreshed = refreshDurationPolicy({
    schema_version: 1,
    fallback_seconds: 1,
    targets: { existing: 3 },
  }, [
    { schema_version: 1, archive_label: "b", run_id: "10", ref: "refs/heads/devel", sha: "b-sha", targets: { beta: 4 } },
    { schema_version: 1, archive_label: "c", run_id: "10", ref: "refs/heads/devel", sha: "c-sha", targets: { alpha: 2 } },
  ]);

  assert.deepEqual(refreshed.targets, { alpha: 2, beta: 4, existing: 3 });
  assert.deepEqual(refreshed.measurement_sources, {
    b: { run_id: "10", ref: "refs/heads/devel", sha: "b-sha" },
    c: { run_id: "10", ref: "refs/heads/devel", sha: "c-sha" },
  });
});
