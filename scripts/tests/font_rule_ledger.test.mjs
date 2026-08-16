import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  assertSourceBoundary,
  expandFixtureSnippets,
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

test('all declared source owners and selectors exist in the current checkout', () => {
  const sources = readJson(path.join(INVESTIGATION, 'font_rule_sources.json'));
  assert.deepEqual(assertSourceBoundary(sources, ROOT), []);
});

test('a missing owner is rejected instead of being treated as zero candidates', () => {
  const sources = readJson(path.join(INVESTIGATION, 'font_rule_sources.json'));
  sources.owners = [];

  assert.match(assertSourceBoundary(sources, ROOT).join('\n'), /owners must not be empty/);
});

test('a disappeared symbol selector is rejected instead of matching zero candidates', () => {
  const sources = readJson(path.join(INVESTIGATION, 'font_rule_sources.json'));
  sources.owners[0].selectors[0].selector = '__removed_selector_for_red_test__';

  assert.match(assertSourceBoundary(sources, ROOT).join('\n'), /matched 0 time/);
});
