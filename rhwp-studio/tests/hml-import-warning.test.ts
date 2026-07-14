import test from 'node:test';
import assert from 'node:assert/strict';

import { buildHmlImportWarningMessage } from '../src/ui/hml-import-warning-message.ts';

test('HML 저장 가능 문서는 재저장 가능 안내와 warning path를 함께 표시한다', () => {
  const message = buildHmlImportWarningMessage({
    format: 'hml',
    hwpmlVersion: '2.91',
    encoding: 'utf-8',
    resourceCount: 0,
    hmlSavable: true,
    saveBlockers: [],
    warnings: [{
      code: 'UnsupportedElement',
      xmlPath: '/HWPML/TAIL/SCRIPTCODE',
      message: '지원하지 않는 HML 요소를 건너뛰었습니다: SCRIPTCODE',
      preserved: true,
    }],
  });

  assert.match(message, /HML 2\.91/);
  assert.match(message, /의미를 보존해 저장/);
  assert.match(message, /바이트/);
  assert.doesNotMatch(message, /HML로는 저장할 수 없습니다/);
  assert.match(message, /\/HWPML\/TAIL\/SCRIPTCODE/);
  assert.match(message, /1건/);
});

test('HML 저장 불가 문서는 저장 차단 안내와 HWP/HWPX 대안을 표시한다', () => {
  const message = buildHmlImportWarningMessage({
    format: 'hml',
    hwpmlVersion: '2.91',
    encoding: 'utf-8',
    resourceCount: 0,
    hmlSavable: false,
    saveBlockers: [{
      code: 'UnsupportedElement',
      xmlPath: '/HWPML/BODY/P/CONTROL/UNKNOWN',
      message: '보존할 수 없는 요소입니다: UNKNOWN',
      preserved: false,
    }],
    warnings: [],
  });

  assert.match(message, /HML 2\.91/);
  assert.match(message, /HML로는 저장할 수 없습니다/);
  assert.match(message, /HWP 또는 HWPX/);
});
