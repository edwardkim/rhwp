import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  assertSourceBoundary,
  canonicalJson,
  expandFixtureSnippets,
  sha256Text,
  verifyHistoricalSourceCandidates,
  validateLedger,
} from '../font_rule_ledger.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const INVESTIGATION = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4939',
);
const FIXTURE = path.join(
  ROOT,
  'scripts',
  'tests',
  'fixtures',
  'font-rule-ledger',
  'source-snippets.json',
);

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

test('grouped mapping, ordered chain and predicate expand to stable ledger rows', () => {
  const ledger = expandFixtureSnippets(readJson(FIXTURE));

  assert.deepEqual(
    ledger.rules.map(rule => [rule.ruleId, rule.sourceFace, rule.targetFaceOrPolicy, rule.order]),
    [
      ['fixture.grouped.1', 'FixtureBatang', 'Fixture Serif', null],
      ['fixture.grouped.2', 'FixtureBatangEnglish', 'Fixture Serif', null],
      ['fixture.chain.1', 'FixtureMissing', 'FixtureA', 0],
      ['fixture.chain.2', 'FixtureMissing', 'FixtureB', 1],
      [
        'fixture.predicate.serif',
        null,
        'if normalized family contains a serif marker, choose serif',
        null,
      ],
    ],
  );
  assert.deepEqual(validateLedger(ledger), []);
});

test('unknown evidence is explicit, while an empty evidence status is rejected', () => {
  const ledger = expandFixtureSnippets(readJson(FIXTURE));
  ledger.rules[0].evidenceStatus = '';

  assert.match(validateLedger(ledger).join('\n'), /evidenceStatus/);
});

test('unknown enum values are rejected', () => {
  const ledger = expandFixtureSnippets(readJson(FIXTURE));
  ledger.rules[0].relationType = 'convenient-alias';

  assert.match(validateLedger(ledger).join('\n'), /relationType/);
});

test('duplicate ruleId values are rejected', () => {
  const ledger = expandFixtureSnippets(readJson(FIXTURE));
  ledger.rules[1].ruleId = ledger.rules[0].ruleId;

  assert.match(validateLedger(ledger).join('\n'), /duplicate ruleId/);
});

test('all declared source owners and selectors exist at the recorded W1 commit', () => {
  const snapshot = readJson(path.join(INVESTIGATION, 'font_rule_candidates.json'));
  assert.deepEqual(verifyHistoricalSourceCandidates(snapshot, ROOT), []);
});

test('a missing owner is rejected instead of being treated as zero candidates', () => {
  const sources = readJson(path.join(INVESTIGATION, 'font_rule_sources.json'));
  sources.owners = [];

  assert.match(assertSourceBoundary(sources, ROOT).join('\n'), /owners must not be empty/);
});

test('a disappeared symbol selector is rejected instead of matching zero candidates', () => {
  const snapshot = readJson(path.join(INVESTIGATION, 'font_rule_candidates.json'));
  snapshot.candidates[0].selector = '__removed_selector_for_red_test__';

  assert.match(
    verifyHistoricalSourceCandidates(snapshot, ROOT).join('\n'),
    /historical selector matched 0/,
  );
});

test('historical source verification is deterministic and closes every selector', () => {
  const first = readJson(path.join(INVESTIGATION, 'font_rule_candidates.json'));
  const second = readJson(path.join(INVESTIGATION, 'font_rule_candidates.json'));

  assert.equal(first.candidates.length, 30);
  assert.equal(first.candidates.every(candidate => candidate.matchCount >= candidate.minMatches), true);
  assert.deepEqual(verifyHistoricalSourceCandidates(first, ROOT), []);
  assert.deepEqual(verifyHistoricalSourceCandidates(second, ROOT), []);
  assert.equal(canonicalJson(first), canonicalJson(second));
  assert.equal(sha256Text(canonicalJson(first)), sha256Text(canonicalJson(second)));
});

test('W0 baseline preserves all metric entries and lookup fallback projection', () => {
  const baseline = readJson(path.join(INVESTIGATION, 'font_rule_baseline.json'));

  assert.equal(baseline.fontMetrics.entryCount, 600);
  assert.equal(baseline.fontMetrics.uniqueNameCount, 401);
  assert.deepEqual(baseline.lookupContract.exactOrder, [
    'name+bold+italic',
    'name+bold+italic=false',
    'name-first',
  ]);
  assert.equal(baseline.lookupContract.knownInputCount > 401, true);
  assert.match(baseline.fontMetrics.tableSha256, /^[0-9a-f]{64}$/);
  assert.match(baseline.lookupContract.projectionSha256, /^[0-9a-f]{64}$/);
});

test('canonical JSON sorts object keys, preserves array order and ends with one newline', () => {
  assert.equal(
    canonicalJson({ z: 1, a: { y: 2, b: 3 }, rows: [{ d: 4, c: 5 }, 6] }),
    '{\n  "a": {\n    "b": 3,\n    "y": 2\n  },\n  "rows": [\n    {\n      "c": 5,\n      "d": 4\n    },\n    6\n  ],\n  "z": 1\n}\n',
  );
});
