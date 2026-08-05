#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { estimateTargetRunMs, readCostModel } from "./nextest_cost_model.mjs";

const SLOW_LABEL = "slow";
const REGULAR_LABELS = ["1", "2", "3"];
const ARCHIVE_BUILDERS = new Map([
  [SLOW_LABEL, "slow"],
  ["1", "a"],
  ["2", "slow"],
  ["3", "b"],
]);

function fail(message) {
  throw new Error(message);
}

function parseArgs(argv) {
  const values = new Map();
  for (let index = 0; index < argv.length; index += 2) {
    const key = argv[index];
    const value = argv[index + 1];
    if (!key?.startsWith("--") || value === undefined) {
      fail("usage: plan_nextest_target_archives.mjs --input FILE --output-dir DIR --package NAME --slow-test-target NAME");
    }
    values.set(key, value);
  }
  return values;
}

function sourceBytes(sourcePath) {
  try {
    return fs.statSync(sourcePath).size;
  } catch (error) {
    fail(`cannot stat Cargo test target source ${sourcePath}: ${error.message}`);
  }
}

function targetSelector(target) {
  if (target.kind.includes("test")) {
    return { identity: `test:${target.name}`, args: ["--test", target.name] };
  }
  if (target.kind.includes("rlib")) {
    return { identity: `lib:${target.name}`, args: ["--lib"] };
  }
  if (target.kind.includes("bin")) {
    return { identity: `bin:${target.name}`, args: ["--bin", target.name] };
  }
  fail(`unsupported test-enabled Cargo target ${target.name} (${target.kind.join(",")})`);
}

function splitCapacity(total, groupCount) {
  const base = Math.floor(total / groupCount);
  const remainder = total % groupCount;
  return Array.from({ length: groupCount }, (_, index) => base + (index < remainder ? 1 : 0));
}

function selectLeastLoaded(groups) {
  return groups.reduce((best, group) => (
    group.sourceBytes < best.sourceBytes
      || (group.sourceBytes === best.sourceBytes && group.label < best.label)
      ? group
      : best
  ));
}

function assignSourceBalancedTargets(targets, groups) {
  for (const target of targets) {
    const available = groups.filter((group) => group.targets.length < group.capacity);
    if (available.length === 0) {
      fail("target assignment exhausted every group capacity");
    }
    const group = selectLeastLoaded(available);
    group.targets.push(target);
    group.sourceBytes += target.sourceBytes;
    group.runMs += target.runMs;
  }
}

function selectCostAwareGroup(groups, target, totalSourceBytes, totalRunMs) {
  return groups.reduce((best, group) => {
    const projected = (candidate) => ({
      sourceBytes: candidate.sourceBytes + target.sourceBytes,
      runMs: candidate.runMs + target.runMs,
    });
    const projectedGroup = projected(group);
    const projectedBest = projected(best);
    const runScore = projectedGroup.runMs / totalRunMs;
    const bestRunScore = projectedBest.runMs / totalRunMs;
    const sourceScore = projectedGroup.sourceBytes / totalSourceBytes;
    const bestSourceScore = projectedBest.sourceBytes / totalSourceBytes;
    if (
      runScore < bestRunScore
      || (runScore === bestRunScore && sourceScore < bestSourceScore)
      || (runScore === bestRunScore && sourceScore === bestSourceScore && group.label < best.label)
    ) {
      return group;
    }
    return best;
  });
}

function assignCostAwareTargets(targets, groups) {
  const totalSourceBytes = Math.max(targets.reduce((sum, target) => sum + target.sourceBytes, 0), 1);
  const totalRunMs = targets.reduce((sum, target) => sum + target.runMs, 0);
  for (const target of targets) {
    const group = selectCostAwareGroup(groups, target, totalSourceBytes, totalRunMs);
    group.targets.push(target);
    group.sourceBytes += target.sourceBytes;
    group.runMs += target.runMs;
  }
}

const args = parseArgs(process.argv.slice(2));
const inputPath = args.get("--input");
const outputDir = args.get("--output-dir");
const packageName = args.get("--package");
const slowTestTarget = args.get("--slow-test-target");
const costModelPath = args.get("--cost-model");

if (!inputPath || !outputDir || !packageName || !slowTestTarget) {
  fail("all arguments are required");
}
if (path.resolve(outputDir) === path.parse(path.resolve(outputDir)).root) {
  fail("refusing to use the filesystem root as --output-dir");
}

const metadata = JSON.parse(fs.readFileSync(inputPath, "utf8"));
const packages = (metadata.packages ?? []).filter((candidate) => candidate.name === packageName);
if (packages.length !== 1) {
  fail(`expected exactly one Cargo package named ${packageName}, found ${packages.length}`);
}

const targetIdentities = new Set();
const costModel = costModelPath ? readCostModel(costModelPath) : null;
if (costModelPath && !costModel) {
  console.warn(`nextest cost model ignored: ${costModelPath}`);
}
const candidates = packages[0].targets
  .filter((target) => target.test === true)
  .map((target) => {
    const selector = targetSelector(target);
    if (targetIdentities.has(selector.identity)) {
      fail(`duplicate Cargo test target selector ${selector.identity}`);
    }
    targetIdentities.add(selector.identity);
    return {
      ...selector,
      name: target.name,
      kind: target.kind,
      sourceBytes: sourceBytes(target.src_path),
    };
  });
for (const target of candidates) {
  target.runMs = estimateTargetRunMs(target, costModel) ?? 0;
}

const slowTargets = candidates.filter((target) => target.identity === `test:${slowTestTarget}`);
if (slowTargets.length !== 1) {
  fail(`expected exactly one integration test target ${slowTestTarget}, found ${slowTargets.length}`);
}
const regularTargets = candidates.filter((target) => target !== slowTargets[0]);
if (regularTargets.length < REGULAR_LABELS.length) {
  fail(`need at least ${REGULAR_LABELS.length} regular targets, found ${regularTargets.length}`);
}

// 이력이 있으면 실제 nextest 실행 시간을 우선 최소화하고 source 크기는 동률 해소에만 쓰는 LPT 배정을 사용한다.
// 이력이 없거나 손상됐으면 기존 source-size + 동일 target 수 계약으로 자동 후퇴한다.
const regularCapacities = splitCapacity(regularTargets.length, REGULAR_LABELS.length);
const regularGroups = REGULAR_LABELS.map((label, index) => ({
  label,
  builder: ARCHIVE_BUILDERS.get(label),
  capacity: costModel ? Number.POSITIVE_INFINITY : regularCapacities[index],
  targets: [],
  sourceBytes: 0,
  runMs: 0,
}));
if (costModel) {
  regularTargets.sort((left, right) => (
    right.runMs - left.runMs
      || right.sourceBytes - left.sourceBytes
      || left.identity.localeCompare(right.identity)
  ));
  assignCostAwareTargets(regularTargets, regularGroups);
} else {
  regularTargets.sort((left, right) => (
    right.sourceBytes - left.sourceBytes || left.identity.localeCompare(right.identity)
  ));
  assignSourceBalancedTargets(regularTargets, regularGroups);
}

const archives = new Map();
archives.set(SLOW_LABEL, {
  label: SLOW_LABEL,
  builder: ARCHIVE_BUILDERS.get(SLOW_LABEL),
  capacity: 1,
  targets: slowTargets,
  sourceBytes: slowTargets[0].sourceBytes,
  runMs: slowTargets[0].runMs,
});
for (const group of regularGroups) {
  if (group.targets.length === 0) {
    fail(`archive ${group.label} has no Cargo test target`);
  }
  archives.set(group.label, group);
}

const orderedLabels = [SLOW_LABEL, ...REGULAR_LABELS];
if (archives.size !== orderedLabels.length || orderedLabels.some((label) => !archives.has(label))) {
  fail("archive labels are incomplete");
}
const assignedIdentities = orderedLabels.flatMap((label) => archives.get(label).targets.map((target) => target.identity));
if (new Set(assignedIdentities).size !== candidates.length || assignedIdentities.length !== candidates.length) {
  fail("Cargo test target assignment contains a duplicate or omission");
}

fs.rmSync(outputDir, { recursive: true, force: true });
fs.mkdirSync(outputDir, { recursive: true });
const plan = {
  package: packageName,
  assignment_strategy: costModel ? "historical-run-time-source-tiebreak" : "source-size-fallback",
  total_test_targets: candidates.length,
  builders: Object.fromEntries(["slow", "a", "b"].map((builder) => {
    const ownedArchives = orderedLabels
      .map((label) => archives.get(label))
      .filter((archive) => archive.builder === builder);
    return [builder, {
      archive_labels: ownedArchives.map((archive) => archive.label),
      total_target_count: ownedArchives.reduce((sum, archive) => sum + archive.targets.length, 0),
      total_source_bytes: ownedArchives.reduce((sum, archive) => sum + archive.sourceBytes, 0),
      total_estimated_run_ms: costModel
        ? Number(ownedArchives.reduce((sum, archive) => sum + archive.runMs, 0).toFixed(3))
        : null,
    }];
  })),
  archives: Object.fromEntries(orderedLabels.map((label) => {
    const archive = archives.get(label);
    fs.writeFileSync(path.join(outputDir, `${label}.args`), `${archive.targets.flatMap((target) => target.args).join("\n")}\n`);
    return [label, {
      builder: archive.builder,
      target_count: archive.targets.length,
      source_bytes: archive.sourceBytes,
      estimated_run_ms: costModel ? Number(archive.runMs.toFixed(3)) : null,
      targets: archive.targets.map(({ identity, name, kind, sourceBytes }) => ({
        identity,
        name,
        kind,
        source_bytes: sourceBytes,
      })),
    }];
  })),
};
fs.writeFileSync(path.join(outputDir, "assignment.json"), `${JSON.stringify(plan, null, 2)}\n`);
console.log(JSON.stringify(plan, null, 2));
