import test from 'node:test';
import assert from 'node:assert/strict';

import { buildHmlSaveFormatMessage } from '../src/ui/hml-save-format-message.ts';

test('HML 저장 차단 안내는 blocker path와 HWP/HWPX 대안을 표시한다', () => {
  const message = buildHmlSaveFormatMessage({
    hmlSavable: false,
    saveBlockers: [{
      code: 'UnsupportedElement',
      xmlPath: '/HWPML/BODY/SECTION/P/UNKNOWN',
      message: '보존할 수 없는 요소',
      preserved: false,
    }],
  }, true);

  assert.match(message, /\/HWPML\/BODY\/SECTION\/P\/UNKNOWN/);
  assert.match(message, /HWP 또는 HWPX/);
});

test('exporter가 없으면 HML 저장을 권하지 않고 capability 진단을 표시한다', () => {
  const message = buildHmlSaveFormatMessage({ hmlSavable: true, saveBlockers: [] }, false);
  assert.match(message, /WASM/);
  assert.doesNotMatch(message, /의미를 보존해 저장할 수 있지만/);
});
