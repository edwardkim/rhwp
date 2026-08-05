import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {
  makeCostSlice,
  mergeCostSlices,
  normalizeCostModel,
  summarizeNextestEvents,
} from "../../.github/scripts/nextest_cost_model.mjs";
import { findNativeSkiaJob, nativeSkiaState } from "../../.github/scripts/watch_native_skia.mjs";

test("libtest-json-plus suite 시간을 Cargo target별 비용으로 요약한다", () => {
  const targets = summarizeNextestEvents([
    JSON.stringify({
      type: "suite",
      event: "ok",
      passed: 2,
      exec_time: 0.25,
      nextest: { crate: "rhwp", test_binary: "fast_case", kind: "test" },
    }),
    JSON.stringify({
      type: "suite",
      event: "ok",
      passed: 1,
      exec_time: 0.75,
      nextest: { crate: "rhwp", test_binary: "fast_case", kind: "test" },
    }),
    JSON.stringify({
      type: "test",
      event: "ok",
      name: "ignored-per-test-event",
    }),
    "",
  ].join("\n"));

  assert.deepEqual(targets.get("test:fast_case"), { runMs: 1000, testCount: 3 });
  assert.equal(targets.size, 1);
});

test("현재 성공 실행만 남기고 EWMA 비용 모델을 갱신한다", () => {
  const prior = normalizeCostModel({
    version: 1,
    fallback_run_ms: 100,
    targets: {
      "test:current": { run_ms: 100, samples: 2 },
      "test:removed": { run_ms: 900, samples: 8 },
    },
  });
  const current = makeCostSlice({
    archiveLabel: "1",
    targets: new Map([
      ["test:current", { runMs: 300, testCount: 4 }],
      ["bin:new", { runMs: 50, testCount: 1 }],
    ]),
  });

  const merged = mergeCostSlices({ priorModel: prior, slices: [current] });
  assert.deepEqual(merged, {
    version: 1,
    fallback_run_ms: 105,
    targets: {
      "bin:new": { run_ms: 50, samples: 1 },
      "test:current": { run_ms: 160, samples: 3 },
    },
  });
});

test("0밀리초 suite도 모델 전체를 무효화하지 않는다", () => {
  const model = normalizeCostModel({
    version: 1,
    fallback_run_ms: 100,
    targets: {
      "test:instant": { run_ms: 0, samples: 1 },
    },
  });
  assert.equal(model?.targets.get("test:instant")?.runMs, 0);
});

test("비용 모델이 있으면 regular archive를 실행 시간 기준으로 나눈다", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "rhwp-nextest-cost-model-"));
  try {
    const sourceDir = path.join(directory, "sources");
    fs.mkdirSync(sourceDir);
    const names = ["slow", "a", "b", "c", "d", "e", "f"];
    const targets = names.map((name) => {
      const source = path.join(sourceDir, name + ".rs");
      fs.writeFileSync(source, "x".repeat(100));
      return { name, kind: ["test"], test: true, src_path: source };
    });
    fs.writeFileSync(path.join(directory, "metadata.json"), JSON.stringify({
      packages: [{ name: "rhwp", targets }],
    }) + "\n");
    fs.writeFileSync(path.join(directory, "model.json"), JSON.stringify({
      version: 1,
      fallback_run_ms: 1,
      targets: {
        "test:a": { run_ms: 600, samples: 1 },
        "test:b": { run_ms: 500, samples: 1 },
        "test:c": { run_ms: 400, samples: 1 },
        "test:d": { run_ms: 50, samples: 1 },
        "test:e": { run_ms: 40, samples: 1 },
        "test:f": { run_ms: 30, samples: 1 },
      },
    }) + "\n");

    execFileSync(process.execPath, [
      ".github/scripts/plan_nextest_target_archives.mjs",
      "--input", path.join(directory, "metadata.json"),
      "--output-dir", path.join(directory, "plan"),
      "--package", "rhwp",
      "--slow-test-target", "slow",
      "--cost-model", path.join(directory, "model.json"),
    ], { cwd: path.resolve("."), stdio: "pipe" });

    const plan = JSON.parse(fs.readFileSync(path.join(directory, "plan", "assignment.json"), "utf8"));
    assert.equal(plan.assignment_strategy, "historical-run-time-source-tiebreak");
    const regular = ["1", "2", "3"].map((label) => plan.archives[label]);
    const assigned = regular.flatMap((archive) => archive.targets.map((target) => target.identity));
    assert.equal(new Set(assigned).size, 6);
    const costs = regular.map((archive) => archive.estimated_run_ms);
    assert.ok(Math.max(...costs) - Math.min(...costs) <= 130, JSON.stringify(costs));
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("Native Skia 상태는 성공 외 완료 상태를 shard 중단 대상으로 구분한다", () => {
  assert.equal(nativeSkiaState(null), "pending");
  assert.equal(nativeSkiaState({ status: "in_progress" }), "pending");
  assert.equal(nativeSkiaState({ status: "completed", conclusion: "success" }), "success");
  assert.equal(nativeSkiaState({ status: "completed", conclusion: "failure" }), "failure");
  assert.deepEqual(
    findNativeSkiaJob([{ name: "Lint" }, { name: "Native Skia tests", status: "completed" }]),
    { name: "Native Skia tests", status: "completed" },
  );
});
