#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";

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
    if (!["--archive-label", "--input", "--output"].includes(arg)) {
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

function decodeXml(value) {
  return value
    .replaceAll("&quot;", "\"")
    .replaceAll("&apos;", "'")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&amp;", "&");
}

function parseAttributes(raw) {
  const attributes = new Map();
  for (const match of raw.matchAll(/([A-Za-z_:][-A-Za-z0-9_.:]*)="([^"]*)"/g)) {
    attributes.set(match[1], decodeXml(match[2]));
  }
  return attributes;
}

export function collectTargetDurations(junitXml) {
  const durations = new Map();
  for (const match of junitXml.matchAll(/<testcase\b([^>]*)>/g)) {
    const attributes = parseAttributes(match[1]);
    const suiteName = attributes.get("classname");
    const seconds = Number(attributes.get("time"));
    if (!suiteName || suiteName.startsWith("@setup-script:") || !Number.isFinite(seconds) || seconds < 0) {
      continue;
    }

    const separator = suiteName.indexOf("::");
    const target = separator === -1 ? suiteName : suiteName.slice(separator + 2);
    if (!target) {
      continue;
    }
    durations.set(target, (durations.get(target) ?? 0) + seconds);
  }
  return Object.fromEntries([...durations.entries()].sort(([left], [right]) => left.localeCompare(right)));
}

function targetForClassName(className) {
  const separator = className.indexOf("::");
  return separator === -1 ? className : className.slice(separator + 2);
}

function caseNameForTest(target, testName) {
  if (!target.startsWith("regression_suite_")) {
    return target;
  }
  const separator = testName.indexOf("::");
  if (separator <= 0) {
    fail(`grouped target testcase must include its source module: ${target}::${testName}`);
  }
  return testName.slice(0, separator);
}

export function collectDurationMeasurement(junitXml) {
  const targets = new Map();
  const cases = new Map();
  const testCases = new Map();
  for (const match of junitXml.matchAll(/<testcase\b([^>]*)>/g)) {
    const attributes = parseAttributes(match[1]);
    const className = attributes.get("classname");
    const testName = attributes.get("name");
    const seconds = Number(attributes.get("time"));
    if (
      !className
      || !testName
      || className.startsWith("@setup-script:")
      || !Number.isFinite(seconds)
      || seconds < 0
    ) {
      continue;
    }
    const target = targetForClassName(className);
    if (!target) {
      continue;
    }
    const caseName = caseNameForTest(target, testName);
    const testCase = `${target}::${testName}`;
    targets.set(target, (targets.get(target) ?? 0) + seconds);
    cases.set(caseName, (cases.get(caseName) ?? 0) + seconds);
    testCases.set(testCase, (testCases.get(testCase) ?? 0) + seconds);
  }
  return {
    targets: Object.fromEntries([...targets.entries()].sort(([left], [right]) => left.localeCompare(right))),
    cases: Object.fromEntries([...cases.entries()].sort(([left], [right]) => left.localeCompare(right))),
    test_cases: Object.fromEntries([...testCases.entries()].sort(([left], [right]) => left.localeCompare(right))),
  };
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  if (options.help) {
    process.stdout.write("Usage: node scripts/collect-nextest-target-durations.mjs --archive-label b|c|d --input junit.xml --output target-durations.json\n");
    return;
  }
  if (!["b", "c", "d"].includes(options["archive-label"]) || !options.input || !options.output) {
    fail("--archive-label b|c|d, --input, and --output are required");
  }

  const measurement = collectDurationMeasurement(fs.readFileSync(options.input, "utf8"));
  if (Object.keys(measurement.targets).length === 0) {
    fail("JUnit report contains no target durations");
  }
  const report = {
    schema_version: 2,
    archive_label: options["archive-label"],
    run_id: process.env.GITHUB_RUN_ID ?? null,
    ref: process.env.GITHUB_REF ?? null,
    sha: process.env.GITHUB_SHA ?? null,
    ...measurement,
  };
  fs.mkdirSync(path.dirname(options.output), { recursive: true });
  fs.writeFileSync(options.output, `${JSON.stringify(report, null, 2)}\n`);
  process.stderr.write(
    `[NextestTargetDuration] archive=${report.archive_label} target_durations=${Object.keys(report.targets).length} case_durations=${Object.keys(report.cases).length}\n`,
  );
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}
