import fs from "node:fs";

export const COST_MODEL_VERSION = 1;
const MAX_SAMPLES = 20;
const CURRENT_SAMPLE_WEIGHT = 0.3;

function finitePositive(value) {
  return typeof value === "number" && Number.isFinite(value) && value > 0;
}

function finiteNonNegative(value) {
  return typeof value === "number" && Number.isFinite(value) && value >= 0;
}

function targetKindPrefix(kind) {
  const normalized = String(kind ?? "").toLowerCase();
  if (normalized === "lib" || normalized.includes("rlib")) return "lib";
  if (normalized === "bin" || normalized.includes("bin")) return "bin";
  if (normalized.includes("test")) return "test";
  return null;
}

export function targetIdentityFromNextest(nextest) {
  if (!nextest || typeof nextest !== "object") return null;
  const prefix = targetKindPrefix(nextest.kind);
  const name = typeof nextest.test_binary === "string" ? nextest.test_binary : "";
  return prefix && name ? `${prefix}:${name}` : null;
}

export function readCostModel(filePath) {
  try {
    return normalizeCostModel(JSON.parse(fs.readFileSync(filePath, "utf8")));
  } catch {
    return null;
  }
}

export function normalizeCostModel(value) {
  if (!value || value.version !== COST_MODEL_VERSION || typeof value.targets !== "object") {
    return null;
  }

  const targets = new Map();
  for (const [identity, entry] of Object.entries(value.targets)) {
    if (
      !/^(test|lib|bin):[^\s:]+$/.test(identity)
      || !entry
      || !finiteNonNegative(entry.run_ms)
      || !Number.isInteger(entry.samples)
      || entry.samples < 1
    ) {
      return null;
    }
    targets.set(identity, {
      runMs: entry.run_ms,
      samples: Math.min(entry.samples, MAX_SAMPLES),
    });
  }

  if (targets.size === 0) return null;
  const fallbackRunMs = finitePositive(value.fallback_run_ms)
    ? value.fallback_run_ms
    : median([...targets.values()].map((entry) => entry.runMs)) ?? 1;
  return { targets, fallbackRunMs };
}

export function estimateTargetRunMs(target, costModel) {
  const observed = costModel?.targets.get(target.identity);
  return observed?.runMs ?? costModel?.fallbackRunMs ?? null;
}

export function summarizeNextestEvents(text) {
  const targets = new Map();
  for (const [index, line] of text.split(/\r?\n/).entries()) {
    if (!line.trim()) continue;
    let event;
    try {
      event = JSON.parse(line);
    } catch (error) {
      throw new Error(`invalid libtest-json-plus line ${index + 1}: ${error.message}`);
    }
    if (event.type !== "suite" || event.event !== "ok") continue;
    const identity = targetIdentityFromNextest(event.nextest);
    const runMs = Number(event.exec_time) * 1000;
    if (!identity || !finiteNonNegative(runMs)) continue;
    const current = targets.get(identity) ?? { runMs: 0, testCount: 0 };
    current.runMs += runMs;
    current.testCount += Number.isInteger(event.passed) ? event.passed : 0;
    targets.set(identity, current);
  }
  if (targets.size === 0) {
    throw new Error("libtest-json-plus output did not contain a successful suite with nextest metadata");
  }
  return targets;
}

export function makeCostSlice({ archiveLabel, targets }) {
  if (!archiveLabel || !targets || targets.size === 0) {
    throw new Error("cost slice requires an archive label and at least one target");
  }
  return {
    version: COST_MODEL_VERSION,
    archive_label: archiveLabel,
    targets: Object.fromEntries([...targets.entries()].sort(([left], [right]) => left.localeCompare(right)).map(
      ([identity, entry]) => [identity, {
        run_ms: Number(entry.runMs.toFixed(3)),
        test_count: entry.testCount,
      }],
    )),
  };
}

export function mergeCostSlices({ priorModel, slices }) {
  const prior = priorModel ?? { targets: new Map(), fallbackRunMs: 1 };
  const current = new Map();
  for (const slice of slices) {
    const normalized = normalizeCostSlice(slice);
    for (const [identity, entry] of normalized.targets) {
      if (current.has(identity)) {
        throw new Error(`duplicate target cost in current run: ${identity}`);
      }
      current.set(identity, entry);
    }
  }
  if (current.size === 0) throw new Error("no nextest cost slices were supplied");

  const targets = {};
  for (const [identity, currentEntry] of [...current.entries()].sort(([left], [right]) => left.localeCompare(right))) {
    const previous = prior.targets.get(identity);
    const runMs = previous
      ? previous.runMs * (1 - CURRENT_SAMPLE_WEIGHT) + currentEntry.runMs * CURRENT_SAMPLE_WEIGHT
      : currentEntry.runMs;
    targets[identity] = {
      run_ms: Number(runMs.toFixed(3)),
      samples: Math.min((previous?.samples ?? 0) + 1, MAX_SAMPLES),
    };
  }

  const fallbackRunMs = median(Object.values(targets).map((entry) => entry.run_ms)) ?? prior.fallbackRunMs;
  return {
    version: COST_MODEL_VERSION,
    fallback_run_ms: Number(fallbackRunMs.toFixed(3)),
    targets,
  };
}

function normalizeCostSlice(value) {
  if (!value || value.version !== COST_MODEL_VERSION || typeof value.archive_label !== "string" || typeof value.targets !== "object") {
    throw new Error("invalid nextest cost slice");
  }
  const targets = new Map();
  for (const [identity, entry] of Object.entries(value.targets)) {
    if (!/^(test|lib|bin):[^\s:]+$/.test(identity) || !entry || !finiteNonNegative(entry.run_ms)) {
      throw new Error(`invalid nextest cost entry: ${identity}`);
    }
    targets.set(identity, {
      runMs: entry.run_ms,
      testCount: Number.isInteger(entry.test_count) && entry.test_count >= 0 ? entry.test_count : 0,
    });
  }
  return { archiveLabel: value.archive_label, targets };
}

function median(values) {
  const sorted = values.filter(finitePositive).sort((left, right) => left - right);
  if (sorted.length === 0) return null;
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
}
