import test from 'node:test';
import assert from 'node:assert/strict';

import {
  fileNameForFormat,
  forgetConvertedHmlSaveHandle,
  markConvertedHmlSaveHandle,
  requiresSaveFormatChoice,
  resolveSaveTarget,
} from '../src/command/save-target.ts';

test('fallback 파일명은 선택한 HML/HWP/HWPX 확장자를 정확히 적용한다', () => {
  assert.equal(fileNameForFormat('document', 'hwp'), 'document.hwp');
  assert.equal(fileNameForFormat('document.hwp', 'hwpx'), 'document.hwpx');
  assert.equal(fileNameForFormat('document.hwpx', 'hwpx'), 'document.hwpx');
  assert.equal(fileNameForFormat('document.hml', 'hml'), 'document.hml');
  assert.equal(fileNameForFormat('document', 'hml'), 'document.hml');
});

test('원본 HML은 변환 저장 이력이 없으면 HML 이름으로 다른 이름 저장을 강제한다', () => {
  const originalHandle = { name: 'aligns.hml' };
  assert.deepEqual(resolveSaveTarget('hml', 'aligns.hml', originalHandle), {
    format: 'hml',
    forceSaveAs: true,
    suggestedName: 'aligns.hml',
  });
});

test('HWPX로 변환 저장한 HML은 이후 Ctrl+S에서 HWPX 대상을 유지한다', () => {
  const convertedHandle = { name: 'converted.hwpx' };
  markConvertedHmlSaveHandle(convertedHandle);
  assert.deepEqual(resolveSaveTarget('hml', 'converted.hwpx', convertedHandle), {
    format: 'hwpx',
    forceSaveAs: false,
    suggestedName: 'converted.hwpx',
  });
});

test('HML로 저장한 HML은 이후 Ctrl+S에서 같은 HML 대상을 유지한다', () => {
  const savedHandle = { name: 'saved.hml' };
  markConvertedHmlSaveHandle(savedHandle);
  assert.deepEqual(resolveSaveTarget('hml', 'saved.hml', savedHandle), {
    format: 'hml',
    forceSaveAs: false,
    suggestedName: 'saved.hml',
  });
});

test('저장된 HML은 capability가 사라지면 두 번째 Ctrl+S에서 형식을 다시 선택한다', () => {
  const savedHandle = { name: 'saved.hml' };
  markConvertedHmlSaveHandle(savedHandle);
  const target = resolveSaveTarget('hml', 'saved.hml', savedHandle);

  assert.equal(requiresSaveFormatChoice(target, true), false);
  assert.equal(requiresSaveFormatChoice(target, false), true);
});

test('확장자가 HWP여도 새로 연 HML 원본 handle은 변환 대상으로 오인하지 않는다', () => {
  const misleadingSourceHandle = { name: 'misleading.hwp' };
  assert.deepEqual(resolveSaveTarget('hml', 'misleading.hwp', misleadingSourceHandle), {
    format: 'hml',
    forceSaveAs: true,
    suggestedName: 'misleading.hml',
  });
});

test('새 문서로 다시 연 handle은 이전 HML 변환 저장 표식을 지운다', () => {
  const reusedHandle = { name: 'converted.hwp' };
  markConvertedHmlSaveHandle(reusedHandle);
  forgetConvertedHmlSaveHandle(reusedHandle);
  assert.equal(resolveSaveTarget('hml', 'converted.hwp', reusedHandle).forceSaveAs, true);
});

test('기존 HWP/HWPX 문서는 출처 포맷 저장 동작을 유지한다', () => {
  assert.deepEqual(resolveSaveTarget('hwp', 'sample.hwp', { name: 'sample.hwp' }), {
    format: 'hwp',
    forceSaveAs: false,
    suggestedName: 'sample.hwp',
  });
  assert.deepEqual(resolveSaveTarget('hwpx', 'sample.hwpx', { name: 'sample.hwpx' }), {
    format: 'hwpx',
    forceSaveAs: false,
    suggestedName: 'sample.hwpx',
  });
});
