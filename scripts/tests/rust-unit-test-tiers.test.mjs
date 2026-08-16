import assert from 'node:assert/strict';
import {
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import {
  buildTierManifest,
  classifyTestModule,
  inventorySourceTests,
  maskRustNonCode,
} from '../rust-unit-test-tiers.mjs';

function fixture(t, source) {
  const root = mkdtempSync(path.join(os.tmpdir(), 'rhwp-unit-tier-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  mkdirSync(path.join(root, 'src'), { recursive: true });
  writeFileSync(path.join(root, 'src', 'sample.rs'), source);
  return root;
}

test('private 의존성에 따라 세 tier를 구분한다', () => {
  assert.equal(classifyTestModule('use super::*;').tier, 'white_box');
  assert.equal(classifyTestModule('use crate::model::Document;').tier, 'test_support');
  assert.equal(classifyTestModule('use std::collections::HashMap;').tier, 'integration_ready');
});

test('문자열과 주석의 brace를 module 경계로 해석하지 않는다', () => {
  const source = 'const E: &str = "😀"; mod tests { const S: &str = "}"; /* } */ }';
  const masked = maskRustNonCode(source);
  assert.equal(masked.length, source.length);
  assert.equal(masked.split('{').length - 1, 1);
  assert.equal(masked.split('}').length - 1, 1);
});

test('repository source를 module과 support item으로 전수 분류한다', (t) => {
  const root = fixture(
    t,
    [
      '#[cfg(test)]',
      'mod ready { #[test] fn public_contract() {} }',
      '#[cfg(test)]',
      'mod support { use crate::model::Document; #[test] fn internal() {} }',
      '#[cfg(test)]',
      'mod white { use super::*; #[test] fn private_contract() {} }',
      '#[cfg(test)]',
      'const TEST_FLAG: bool = true;',
    ].join('\n'),
  );
  const inventory = inventorySourceTests(root);
  assert.equal(inventory.summary.cfgTestModules, 3);
  assert.equal(inventory.summary.cfgTestSupportItems, 1);
  assert.equal(inventory.summary.staticTestAttributes, 3);
  assert.equal(inventory.summary.tiers.integration_ready.testAttributes, 1);
  assert.equal(inventory.summary.tiers.test_support.testAttributes, 1);
  assert.equal(inventory.summary.tiers.white_box.testAttributes, 1);
});

test('내부 workspace crate의 source test도 기준선에 포함한다', (t) => {
  const root = fixture(t, 'pub fn root() {}');
  const crateSource = path.join(root, 'crates', 'leaf', 'src');
  mkdirSync(crateSource, { recursive: true });
  writeFileSync(
    path.join(crateSource, 'lib.rs'),
    '#[cfg(test)] mod tests { use super::*; #[test] fn leaf_contract() {} }',
  );
  const inventory = inventorySourceTests(root);
  assert.equal(inventory.summary.cfgTestModules, 1);
  assert.equal(inventory.modules[0].file, 'crates/leaf/src/lib.rs');
  assert.equal(inventory.modules[0].tier, 'white_box');
});

test('#[path] 외부 test module을 선언 파일 기준으로 해석한다', (t) => {
  const root = fixture(
    t,
    '#[cfg(test)]\n#[path = "sample_tests.rs"]\nmod tests;',
  );
  writeFileSync(
    path.join(root, 'src', 'sample_tests.rs'),
    'use super::*; #[test] fn private_contract() {}',
  );
  const inventory = inventorySourceTests(root);
  assert.equal(inventory.summary.cfgTestModules, 1);
  assert.equal(inventory.summary.staticTestAttributes, 1);
  assert.equal(inventory.modules[0].sourcePath, 'src/sample_tests.rs');
  assert.equal(inventory.modules[0].tier, 'white_box');
});

test('기존 module의 source-side test 증가는 거부한다', (t) => {
  const root = fixture(
    t,
    '#[cfg(test)] mod tests { use super::*; #[test] fn first() {} }',
  );
  const baseline = buildTierManifest(inventorySourceTests(root)).manifest;
  writeFileSync(
    path.join(root, 'src', 'sample.rs'),
    '#[cfg(test)] mod tests { use super::*; #[test] fn first() {} #[test] fn second() {} }',
  );
  const result = buildTierManifest(inventorySourceTests(root), baseline);
  assert.match(result.violations.join('\n'), /source unit test 증가 금지/);
});

test('신규 cfg(test) module과 support item은 명시적 baseline 승인 없이 거부한다', (t) => {
  const root = fixture(t, 'pub fn value() -> u32 { 1 }');
  const baseline = buildTierManifest(inventorySourceTests(root)).manifest;
  writeFileSync(
    path.join(root, 'src', 'sample.rs'),
    [
      '#[cfg(test)] mod tests { #[test] fn added() {} }',
      '#[cfg(test)] const TEST_FLAG: bool = true;',
    ].join('\n'),
  );
  const result = buildTierManifest(inventorySourceTests(root), baseline);
  assert.match(result.violations.join('\n'), /신규 cfg\(test\) module 금지/);
  assert.match(result.violations.join('\n'), /신규 cfg\(test\) support item 금지/);
});

test('기존 source-side test 감소는 허용하고 최대값은 보존한다', (t) => {
  const root = fixture(
    t,
    '#[cfg(test)] mod tests { use super::*; #[test] fn first() {} #[test] fn second() {} }',
  );
  const baseline = buildTierManifest(inventorySourceTests(root)).manifest;
  writeFileSync(
    path.join(root, 'src', 'sample.rs'),
    '#[cfg(test)] mod tests { use super::*; #[test] fn first() {} }',
  );
  const result = buildTierManifest(inventorySourceTests(root), baseline);
  assert.deepEqual(result.violations, []);
  assert.equal(result.manifest.modules[0].testAttributes, 1);
  assert.equal(result.manifest.modules[0].maximumTestAttributes, 2);
});

test('생성한 tier manifest를 다시 계산해도 동일하다', (t) => {
  const root = fixture(
    t,
    [
      '#[cfg(test)] static UPPER_FLAG: bool = true;',
      '#[cfg(test)] fn lower_support() {}',
      '#[cfg(test)] mod tests { use super::*; #[test] fn contract() {} }',
    ].join('\n'),
  );
  const inventory = inventorySourceTests(root);
  const baseline = buildTierManifest(inventory).manifest;
  const rebuilt = buildTierManifest(inventory, baseline);
  assert.deepEqual(rebuilt.violations, []);
  assert.deepEqual(rebuilt.manifest, baseline);
});
