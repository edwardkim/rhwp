import assert from 'node:assert/strict';
import test from 'node:test';

import { FONT_RULE_WEBFONT_ENTRIES } from '../src/core/font-rule-runtime.ts';

test('한양중고딕 문서 요청명은 Canvas2D 대체 웹폰트 face로 등록한다', () => {
  assert.deepEqual(
    FONT_RULE_WEBFONT_ENTRIES.find(entry => entry.name === '한양중고딕'),
    {
      name: '한양중고딕',
      file: 'fonts/NotoSansKR-Regular.woff2',
      format: 'woff2',
      ruleId: 'rule.studio-supply.856cdf2bb6bf547fb7da.canvas2d',
    },
  );
});
