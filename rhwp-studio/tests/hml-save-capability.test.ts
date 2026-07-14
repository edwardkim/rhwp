import test from 'node:test';
import assert from 'node:assert/strict';

import {
  normalizeHmlSaveState,
  parseHmlSaveState,
  readHmlSaveContext,
  resolveHmlSaveCapability,
} from '../src/core/hml-save-capability.ts';

test('HML 저장은 명시적 savable 메타데이터와 exporter가 모두 있을 때만 활성화된다', () => {
  assert.deepEqual(resolveHmlSaveCapability({ hmlSavable: true, saveBlockers: [] }, true), {
    hmlEnabled: true,
    diagnostic: null,
  });
  assert.equal(resolveHmlSaveCapability({ hmlSavable: true, saveBlockers: [] }, false).hmlEnabled, false);
  assert.equal(resolveHmlSaveCapability({ hmlSavable: false, saveBlockers: [] }, true).hmlEnabled, false);
  assert.equal(resolveHmlSaveCapability({}, true).hmlEnabled, false);
  assert.equal(resolveHmlSaveCapability(null, true).hmlEnabled, false);
});

test('누락되거나 잘못된 HML savability 필드는 저장 불가로 정규화된다', () => {
  assert.deepEqual(normalizeHmlSaveState({}), {
    hmlSavable: false,
    saveBlockers: [],
  });
  assert.deepEqual(normalizeHmlSaveState({ hmlSavable: true, saveBlockers: 'bad' }), {
    hmlSavable: false,
    saveBlockers: [],
  });
  assert.deepEqual(normalizeHmlSaveState({
    hmlSavable: true,
    saveBlockers: [{ code: 7, xmlPath: '/HWPML', message: 'bad' }],
  }), {
    hmlSavable: false,
    saveBlockers: [],
  });
  assert.deepEqual(normalizeHmlSaveState({
    hmlSavable: true,
    saveBlockers: [{ code: 'LOSS', xmlPath: '/HWPML', message: 'blocked' }],
  }), {
    hmlSavable: false,
    saveBlockers: [{
      code: 'LOSS', xmlPath: '/HWPML', message: 'blocked', preserved: false,
    }],
  });
  assert.equal(normalizeHmlSaveState(null), null);
});

test('canonical HML save state는 source와 blocker 종류에 관계없이 exact wire DTO를 유지한다', () => {
  const cases = [
    {
      input: { sourceFormat: 'hml', hmlSavable: true, blockers: [] },
      expected: { sourceFormat: 'hml', hmlSavable: true, blockers: [] },
    },
    {
      input: {
        sourceFormat: 'hwp',
        hmlSavable: false,
        blockers: [{
          code: 'HML_SOURCE_REQUIRED',
          xmlPath: '/HWPML',
          message: 'HML source metadata is required',
          preserved: false,
        }],
      },
      expected: {
        sourceFormat: 'hwp',
        hmlSavable: false,
        blockers: [{
          code: 'HML_SOURCE_REQUIRED',
          xmlPath: '/HWPML',
          message: 'HML source metadata is required',
          preserved: false,
        }],
      },
    },
    {
      input: {
        sourceFormat: 'hml',
        hmlSavable: false,
        blockers: [{
          code: 'HML_UNSUPPORTED_EQUATION_SEMANTICS',
          xmlPath: '/HWPML/BODY/SECTION/P/TEXT/EQUATION/@Unknown',
          message: 'Unknown equation attribute cannot be preserved',
          preserved: false,
        }],
      },
      expected: {
        sourceFormat: 'hml',
        hmlSavable: false,
        blockers: [{
          code: 'HML_UNSUPPORTED_EQUATION_SEMANTICS',
          xmlPath: '/HWPML/BODY/SECTION/P/TEXT/EQUATION/@Unknown',
          message: 'Unknown equation attribute cannot be preserved',
          preserved: false,
        }],
      },
    },
  ];

  for (const { input, expected } of cases) {
    assert.deepEqual(parseHmlSaveState(input), expected);
  }
  assert.equal(parseHmlSaveState({ hmlSavable: true, blockers: [] }), null);
  assert.equal(parseHmlSaveState({
    sourceFormat: 'hml', hmlSavable: false,
    blockers: [{ code: 'LOSS', xmlPath: '/HWPML', message: 'blocked' }],
  }), null);
});

test('HML 저장 비활성 사유는 capability와 metadata 문제를 구분한다', () => {
  assert.match(resolveHmlSaveCapability({ hmlSavable: true, saveBlockers: [] }, false).diagnostic ?? '', /WASM/);
  assert.match(resolveHmlSaveCapability(null, true).diagnostic ?? '', /저장 정보를 확인/);
  assert.match(resolveHmlSaveCapability({ hmlSavable: false, saveBlockers: [] }, true).diagnostic ?? '', /보존할 수 없는/);
});

test('metadata 또는 exporter 조회가 던져도 HML 저장은 진단 정보와 함께 fail-closed된다', () => {
  const metadataFailure = readHmlSaveContext(
    () => { throw new Error('metadata unavailable'); },
    () => true,
  );
  assert.deepEqual(metadataFailure, { metadata: null, exporterAvailable: true });
  assert.equal(resolveHmlSaveCapability(
    metadataFailure.metadata,
    metadataFailure.exporterAvailable,
  ).hmlEnabled, false);

  const exporterFailure = readHmlSaveContext(
    () => ({
      hmlSavable: false,
      saveBlockers: [{
        code: 'Loss', xmlPath: '/HWPML/BODY', message: 'blocked', preserved: false,
      }],
    }),
    () => { throw new Error('exporter unavailable'); },
  );
  assert.equal(exporterFailure.exporterAvailable, false);
  assert.deepEqual(exporterFailure.metadata?.saveBlockers, [
    { code: 'Loss', xmlPath: '/HWPML/BODY', message: 'blocked', preserved: false },
  ]);
});
