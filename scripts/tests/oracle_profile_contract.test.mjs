import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  applyPublicFixtureMutation,
  validateFrozenOracleInputs,
  validateOracleProfile,
  validateOracleProfileContract,
  validateOracleProfileSchema,
  validatePublicOracleFixtures,
} from '../oracle_profile_contract.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const INVESTIGATION = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4963',
);
const CONTRACT = JSON.parse(fs.readFileSync(
  path.join(INVESTIGATION, 'oracle_profile_contract.json'),
  'utf8',
));
const SCHEMA = JSON.parse(fs.readFileSync(
  path.join(INVESTIGATION, 'oracle_profile.schema.json'),
  'utf8',
));
const FIXTURES = JSON.parse(fs.readFileSync(
  path.join(INVESTIGATION, 'oracle_profile_public_fixtures.json'),
  'utf8',
));
const PROFILE_DIRECTORY = path.join(INVESTIGATION, 'profiles');

function clone(value) {
  return structuredClone(value);
}

test('W5 contract freezes the W4 queue and controlled ladder without product changes', () => {
  assert.deepEqual(validateOracleProfileContract(CONTRACT), []);
  assert.equal(CONTRACT.inputPreconditions.queueFaces.length, 17);
  assert.deepEqual(CONTRACT.profilePolicy.questionIds, [
    'exact-installed',
    'exact-removed',
    'document-subst-font-only',
    'curated-official-successor-only',
    'all-related-fonts-missing',
  ]);
  assert.deepEqual(CONTRACT.profilePolicy.observationStatuses, [
    'observed',
    'unavailable',
    'not-applicable',
    'blocked',
  ]);
  assert.equal(CONTRACT.profilePolicy.hmtxAndPdfAdvanceSeparated, true);
  assert.equal(CONTRACT.profilePolicy.historicalImportMissingProvenanceExplicit, true);
  assert.equal(CONTRACT.profilePolicy.unknownIdentityGuessing, false);
  assert.equal(CONTRACT.environmentPolicy.versionBranching, false);
  assert.equal(
    CONTRACT.environmentPolicy.mutableFontState,
    'disposable-snapshot-explicit-approval-only',
  );
  assert.ok(Object.values(CONTRACT.scope).every(value => value === false));
});

test('W5 contract reconciles the exact W4 public artifact and prior evidence hashes', () => {
  assert.deepEqual(validateFrozenOracleInputs(CONTRACT, ROOT), []);
});

test('JSON Schema enum inventories cannot drift from the executable contract', () => {
  assert.deepEqual(validateOracleProfileSchema(SCHEMA, CONTRACT), []);

  const changed = clone(SCHEMA);
  changed.$defs.relationType.enum = ['unknown'];
  assert.ok(validateOracleProfileSchema(changed, CONTRACT).includes(
    'schema relation inventory differs from the contract',
  ));
});

test('public synthetic fixture validates but is explicitly not Oracle evidence', () => {
  assert.equal(FIXTURES.validProfile.execution.evidenceClass, 'synthetic-contract-fixture');
  assert.equal(FIXTURES.validProfile.environment.oracleAuthority, 'contract-fixture');
  assert.match(FIXTURES.validProfile.environment.hancomVersion.value, /not-an-oracle/u);
  assert.deepEqual(validateOracleProfile(FIXTURES.validProfile, CONTRACT), []);
});

test('tracked historical and HWP 2020 Oracle Profiles satisfy the executable contract', () => {
  const names = fs.readdirSync(PROFILE_DIRECTORY)
    .filter(name => name.endsWith('_exact_installed.json'))
    .sort();
  assert.deepEqual(names, [
    'historical_hanyang_sinmyeongjo_exact_installed.json',
    'historical_human_myeongjo_exact_installed.json',
    'windows_hwp2020_malgun_gothic_exact_installed.json',
  ]);
  for (const name of names) {
    const profile = JSON.parse(fs.readFileSync(path.join(PROFILE_DIRECTORY, name), 'utf8'));
    assert.deepEqual(validateOracleProfile(profile, CONTRACT), [], name);
  }
});

test('all public negative fixtures fail closed with their declared reason', () => {
  assert.equal(FIXTURES.negativeCases.length, 9);
  assert.deepEqual(validatePublicOracleFixtures(FIXTURES, CONTRACT), []);
  for (const mutation of FIXTURES.negativeCases) {
    const profile = applyPublicFixtureMutation(FIXTURES.validProfile, mutation);
    const errors = validateOracleProfile(profile, CONTRACT);
    assert.ok(errors.includes(mutation.expectedError), `${mutation.id}: ${errors.join('; ')}`);
  }
});

test('unobserved evidence is explicit and never fabricated as a value', () => {
  for (const status of ['unavailable', 'not-applicable', 'blocked']) {
    const profile = clone(FIXTURES.validProfile);
    profile.observations.hmtxAdvance = {
      status,
      value: null,
      reason: `${status} public test reason`,
    };
    assert.deepEqual(validateOracleProfile(profile, CONTRACT), []);

    profile.observations.hmtxAdvance.value = 1000;
    assert.ok(validateOracleProfile(profile, CONTRACT).includes(
      `observations.hmtxAdvance ${status} evidence value must be null`,
    ));
  }
});

test('PDF observed widths cannot enter the SFNT hmtx envelope', () => {
  const profile = clone(FIXTURES.validProfile);
  profile.observations.hmtxAdvance.value = clone(
    profile.observations.pdfObservedAdvance.value,
  );
  const errors = validateOracleProfile(profile, CONTRACT);
  assert.ok(errors.includes('observations.hmtxAdvance.value schema drift'));

  const opposite = clone(FIXTURES.validProfile);
  opposite.observations.pdfObservedAdvance.value = clone(
    opposite.observations.hmtxAdvance.value,
  );
  assert.ok(validateOracleProfile(opposite, CONTRACT).includes(
    'observations.pdfObservedAdvance.value schema drift',
  ));
});

test('relation kinds remain distinct and a claimed relation needs direct evidence', () => {
  for (const type of CONTRACT.profilePolicy.relationTypes.filter(type => type !== 'unknown')) {
    const profile = clone(FIXTURES.validProfile);
    profile.relationEvidence.type = type;
    profile.relationEvidence.anchor.value = `direct-public-anchor-for-${type}`;
    assert.deepEqual(validateOracleProfile(profile, CONTRACT), []);

    profile.relationEvidence.anchor = {
      status: 'unavailable',
      value: null,
      reason: 'no direct evidence',
    };
    assert.ok(validateOracleProfile(profile, CONTRACT).includes(
      `${type} requires an observed direct anchor`,
    ));
  }

  const unknown = clone(FIXTURES.validProfile);
  unknown.relationEvidence = {
    type: 'unknown',
    anchor: {
      status: 'unavailable',
      value: null,
      reason: 'relation not established',
    },
  };
  assert.deepEqual(validateOracleProfile(unknown, CONTRACT), []);
});

test('evidence class fixes authority and primary Oracle runs require process reset', () => {
  const mismatch = clone(FIXTURES.validProfile);
  mismatch.execution.evidenceClass = 'oracle-run';
  assert.ok(validateOracleProfile(mismatch, CONTRACT).includes(
    'evidenceClass and oracleAuthority do not match',
  ));

  mismatch.environment.oracleAuthority = 'acceptance-primary';
  mismatch.environment.processReset.value = false;
  assert.ok(validateOracleProfile(mismatch, CONTRACT).includes(
    'an oracle-run requires an observed reset Hancom process',
  ));

  mismatch.environment.processReset.value = true;
  assert.deepEqual(validateOracleProfile(mismatch, CONTRACT), []);
});

test('first divergence preserves plane order and a concrete first location', () => {
  const profile = clone(FIXTURES.validProfile);
  profile.observations.firstTypesettingDivergence.value = {
    plane: 'advance',
    characterIndex: 42,
    lineIndex: 1,
    pageIndex: 0,
  };
  assert.deepEqual(validateOracleProfile(profile, CONTRACT), []);

  profile.observations.firstTypesettingDivergence.value = {
    plane: 'page',
    characterIndex: null,
    lineIndex: null,
    pageIndex: null,
  };
  assert.ok(validateOracleProfile(profile, CONTRACT).includes(
    'a non-none divergence must identify a location',
  ));
});

test('local absolute paths and privacy flags cannot enter a public profile', () => {
  const pathLeak = clone(FIXTURES.validProfile);
  pathLeak.relationEvidence.anchor.value = '/home/edward/mygithub/ttfs/private.ttf';
  assert.ok(validateOracleProfile(pathLeak, CONTRACT).some(error => (
    error.includes('exposes an absolute local path')
  )));

  const privacyLeak = clone(FIXTURES.validProfile);
  privacyLeak.privacy.privateDocumentIdentityIncluded = true;
  assert.ok(validateOracleProfile(privacyLeak, CONTRACT).includes(
    'privacy flags must all be false',
  ));
});

test('contract drift in frozen hashes, relations and mutation safety is rejected', () => {
  const changed = clone(CONTRACT);
  changed.inputPreconditions.ranking.fileSha256 = '0'.repeat(64);
  changed.profilePolicy.unknownIdentityGuessing = true;
  changed.environmentPolicy.mutableFontState = 'current-host';
  delete changed.resourcePolicy.rejectSymlinks;
  const errors = validateOracleProfileContract(changed);
  assert.ok(errors.includes('W4 ranking precondition has drifted'));
  assert.ok(errors.includes('profilePolicy.unknownIdentityGuessing must remain false'));
  assert.ok(errors.includes('environment authority or mutation boundary has drifted'));
  assert.ok(errors.includes('resourcePolicy schema drift'));
});
