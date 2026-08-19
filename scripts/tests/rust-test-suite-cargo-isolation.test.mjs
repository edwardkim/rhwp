import assert from 'node:assert/strict';
import test from 'node:test';

import { validateDerivedArtifactChanges } from '../rust-test-suite-manifest.mjs';

const CARGO_BLOCK_START = '# BEGIN RHWP GENERATED TEST TARGETS';
const CARGO_BLOCK_END = '# END RHWP GENERATED TEST TARGETS';

function cargoToml(targetName, beforeBlock = '') {
  return [
    '[package]',
    'name = "fixture"',
    beforeBlock,
    CARGO_BLOCK_START,
    '[[test]]',
    `name = "${targetName}"`,
    `path = "tests/${targetName}.rs"`,
    CARGO_BLOCK_END,
    '',
  ].join('\n');
}

test('명시적 Cargo test target registry 동기화만 허용한다', () => {
  const baseCargoToml = cargoToml('old_target');
  const headCargoToml = cargoToml('new_target');

  assert.equal(
    validateDerivedArtifactChanges([], { baseCargoToml, headCargoToml }).length,
    1,
  );
  assert.deepEqual(
    validateDerivedArtifactChanges([], {
      baseCargoToml,
      headCargoToml,
      allowCargoTargetRegistryChange: true,
    }),
    [],
  );
});

test('Cargo registry 동기화에 marker 블록 밖 변경을 섞지 못한다', () => {
  const baseCargoToml = cargoToml('old_target');
  const headCargoToml = cargoToml('new_target', 'version = "2"');
  const errors = validateDerivedArtifactChanges([], {
    baseCargoToml,
    headCargoToml,
    allowCargoTargetRegistryChange: true,
  });

  assert.equal(errors.length, 1);
  assert.match(errors[0], /marker 블록 밖/);
});
