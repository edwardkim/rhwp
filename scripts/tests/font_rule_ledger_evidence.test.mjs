import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  buildEvidenceLedger,
  validateEvidenceLedger,
} from '../font_rule_ledger_evidence.mjs';
import { canonicalJson } from '../font_rule_ledger.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const CANDIDATES = JSON.parse(fs.readFileSync(path.join(
  ROOT,
  'mydocs/tech/investigations/issue-4939/font_rule_candidates.json',
), 'utf8'));

function freshLedger() {
  return buildEvidenceLedger(CANDIDATES);
}

test('all 1,352 candidates close to one row or an approved profile split', () => {
  const ledger = freshLedger();
  const audit = validateEvidenceLedger(CANDIDATES, ledger, ROOT);

  assert.deepEqual(audit.errors, []);
  assert.equal(ledger.rules.length, 1507);
  assert.equal(ledger.rules.filter(rule => rule.conditions.profile === 'canvas2d').length > 0, true);
  assert.equal(ledger.rules.filter(rule => rule.conditions.profile === 'canvaskit-sfnt').length, 153);
  assert.equal(ledger.rules.filter(rule => rule.conditions.profile === 'source-exact').length, 1);
  assert.equal(ledger.rules.filter(rule => rule.conditions.profile === 'hancom-missing-font').length, 1);
});

test('evidence adjudication is byte deterministic', () => {
  assert.equal(canonicalJson(freshLedger()), canonicalJson(freshLedger()));
});

test('missing candidate coverage fails closed', () => {
  const ledger = freshLedger();
  const removed = ledger.rules.shift();
  const errors = validateEvidenceLedger(CANDIDATES, ledger, ROOT).errors.join('\n');

  assert.match(errors, new RegExp(`${removed.evidence[0].reference.split('#')[1]}: ledger coverage 0`));
});

test('an identity promotion without byte evidence fails closed', () => {
  const ledger = freshLedger();
  const rule = ledger.rules.find(entry => entry.relationType === 'style-fallback');
  rule.relationType = 'identity-alias';
  rule.evidenceStatus = 'inferred';

  assert.match(
    validateEvidenceLedger(CANDIDATES, ledger, ROOT).errors.join('\n'),
    /identity-alias requires verified-by-bytes and font-digest evidence/,
  );
});

test('a conflicting target without explicit precedence fails closed', () => {
  const ledger = freshLedger();
  const rule = ledger.rules.find(entry => entry.sourceOwner === 'studio-substitution'
    && entry.sourceFace === '명조'
    && entry.conditions.languageSlot === '0'
    && entry.conditions.altType === 'source:2->target:1'
    && entry.order === 0);
  assert.ok(rule);
  rule.order = null;

  assert.match(
    validateEvidenceLedger(CANDIDATES, ledger, ROOT).errors.join('\n'),
    /conflicting targets require unique explicit order/,
  );
});

test('a duplicate target for one decision key fails closed', () => {
  const ledger = freshLedger();
  const rows = ledger.rules.filter(entry => entry.sourceOwner === 'studio-substitution'
    && entry.sourceFace === '명조'
    && entry.conditions.languageSlot === '0'
    && entry.conditions.altType === 'source:2->target:1');
  assert.equal(rows.length, 2);
  rows[1].targetFaceOrPolicy = rows[0].targetFaceOrPolicy;

  assert.match(
    validateEvidenceLedger(CANDIDATES, ledger, ROOT).errors.join('\n'),
    /duplicate target for the same decision key/,
  );
});

test('orphan evidence and orphan tests fail closed', () => {
  const ledger = freshLedger();
  ledger.rules[0].evidence.push({ kind: 'document', reference: 'mydocs/missing-font-evidence.md' });
  ledger.rules[0].tests.push('scripts/tests/missing-font-test.mjs');
  const errors = validateEvidenceLedger(CANDIDATES, ledger, ROOT).errors.join('\n');

  assert.match(errors, /orphan document evidence mydocs\/missing-font-evidence\.md/);
  assert.match(errors, /orphan test scripts\/tests\/missing-font-test\.mjs/);
});

test('a detected self-loop must retain its explicit non-identity explanation', () => {
  const ledger = freshLedger();
  const initial = validateEvidenceLedger(CANDIDATES, ledger, ROOT);
  const selfLoop = initial.cycles.find(cycle => cycle.members.length === 1);
  assert.ok(selfLoop);
  const rule = ledger.rules.find(entry => entry.ruleId === selfLoop.rules[0]);
  rule.knownLimitations = ['No disposition.'];

  assert.match(
    validateEvidenceLedger(CANDIDATES, ledger, ROOT).errors.join('\n'),
    /cycle is not documented in knownLimitations/,
  );
});
