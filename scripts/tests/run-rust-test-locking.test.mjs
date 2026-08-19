import assert from 'node:assert/strict';
import test from 'node:test';

import {
  cargoTestArguments,
  nextestArguments,
} from '../run-rust-test.mjs';

const GROUPED_PLAN = {
  target: 'regression_suite_001',
  grouped: true,
  caseName: 'issue_1035_alignment',
};

test('run-rust-test nextest 명령은 Cargo.lock을 갱신하지 않는다', () => {
  assert.deepEqual(
    nextestArguments(GROUPED_PLAN, ['--cargo-profile', 'release-test']),
    [
      'nextest',
      'run',
      '--locked',
      '--test',
      'regression_suite_001',
      '-E',
      'test(/(^|::)issue_1035_alignment::/)',
      '--cargo-profile',
      'release-test',
    ],
  );
});

test('run-rust-test cargo test 명령은 중복 없이 --locked를 전달한다', () => {
  assert.deepEqual(
    cargoTestArguments(GROUPED_PLAN, ['--locked', '--features', 'native-skia']),
    [
      'test',
      '--locked',
      '--features',
      'native-skia',
      '--test',
      'regression_suite_001',
      'issue_1035_alignment::',
    ],
  );
});
