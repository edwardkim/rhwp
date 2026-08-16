import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  collectRuleCandidates,
  validateCandidateSnapshot,
} from '../font_rule_candidates.mjs';
import { canonicalJson, sha256Text } from '../font_rule_ledger.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const CANDIDATE_PATH = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4939',
  'font_rule_candidates.json',
);

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

test('all 30 source boundaries close with extracted candidates', () => {
  const snapshot = collectRuleCandidates(readJson(CANDIDATE_PATH), ROOT);

  assert.equal(snapshot.candidates.length, 30);
  assert.equal(snapshot.dispositions.length, 30);
  assert.equal(snapshot.dispositions.every(entry => entry.status === 'extracted'), true);
  assert.equal(snapshot.dispositions.every(entry => entry.candidateCount > 0), true);
  assert.equal(snapshot.summary.unrecognizedMappingBlockCount, 0);
  assert.deepEqual(validateCandidateSnapshot(snapshot, ROOT), []);
});

test('finite inventories preserve metric, Studio substitution and font supply populations', () => {
  const snapshot = collectRuleCandidates(readJson(CANDIDATE_PATH), ROOT);
  const count = boundaryId => snapshot.ruleCandidates
    .filter(candidate => candidate.sourceBoundaryId === boundaryId).length;

  assert.equal(count('rust-metric.metric-table'), 600);
  assert.equal(count('studio-substitution.substitution-tables'), 265);
  assert.equal(count('studio-supply.font-list'), 153);
  assert.equal(snapshot.ruleCandidates.some(candidate => candidate.candidateKind === 'finite-mapping'), true);
  assert.equal(snapshot.ruleCandidates.some(candidate => candidate.candidateKind === 'ordered-chain'), true);
  assert.equal(snapshot.ruleCandidates.some(candidate => candidate.candidateKind === 'predicate'), true);
});

test('supply, detection and runtime fallback candidates remain on separate decision planes', () => {
  const snapshot = collectRuleCandidates(readJson(CANDIDATE_PATH), ROOT);
  const byOwner = owner => snapshot.ruleCandidates.filter(candidate => candidate.ownerId === owner);

  assert.equal(byOwner('studio-supply').every(candidate => candidate.decisionPlane === 'supply'), true);
  assert.equal(byOwner('asset-authority').every(candidate => candidate.decisionPlane === 'supply'), true);
  assert.equal(byOwner('studio-detection').every(candidate => candidate.decisionPlane === 'detection'), true);
  assert.equal(byOwner('studio-substitution').every(candidate => candidate.decisionPlane === 'paint'), true);
});

test('candidate IDs are unique and every row carries the current source digest', () => {
  const snapshot = collectRuleCandidates(readJson(CANDIDATE_PATH), ROOT);
  const ids = snapshot.ruleCandidates.map(candidate => candidate.candidateId);

  assert.equal(new Set(ids).size, ids.length);
  for (const candidate of snapshot.ruleCandidates) {
    assert.match(candidate.sourceLocation.sourceSha256, /^[0-9a-f]{64}$/);
    const boundary = snapshot.candidates.find(
      entry => `${entry.ownerId}.${entry.selectorId}` === candidate.sourceBoundaryId,
    );
    assert.equal(candidate.sourceLocation.sourceSha256, boundary.sourceSha256);
  }
});

test('candidate collection is byte deterministic', () => {
  const input = readJson(CANDIDATE_PATH);
  const first = collectRuleCandidates(input, ROOT);
  const second = collectRuleCandidates(input, ROOT);

  assert.equal(canonicalJson(first), canonicalJson(second));
  assert.equal(sha256Text(canonicalJson(first)), sha256Text(canonicalJson(second)));
});

test('a zero-count disposition fails closed', () => {
  const snapshot = collectRuleCandidates(readJson(CANDIDATE_PATH), ROOT);
  snapshot.dispositions[0].candidateCount = 0;

  assert.match(validateCandidateSnapshot(snapshot, ROOT).join('\n'), /candidateCount must be positive/);
});

test('an orphan candidate source boundary fails closed', () => {
  const snapshot = collectRuleCandidates(readJson(CANDIDATE_PATH), ROOT);
  snapshot.ruleCandidates[0].sourceBoundaryId = 'missing.owner';

  assert.match(validateCandidateSnapshot(snapshot, ROOT).join('\n'), /unknown sourceBoundaryId/);
});
