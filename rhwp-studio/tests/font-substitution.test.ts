import test from 'node:test';
import assert from 'node:assert/strict';

import {
  fontFamilyChainForDisplay,
  fontFamilyWithFallback,
  resolveFont,
} from '../src/core/font-substitution.ts';

test('resolveFont는 기존 웹 대체 글꼴 해소를 유지한다', () => {
  assert.equal(resolveFont('휴먼명조', 0, 0), 'HY신명조');
  assert.equal(resolveFont('신명 중고딕', 0, 0), 'HY중고딕');
});

test('fontFamilyChainForDisplay는 미승인 로컬 글꼴명을 웹 대체 글꼴보다 앞에 두지 않는다', () => {
  const chain = fontFamilyChainForDisplay('휴먼명조', 0, 0);

  assert.match(chain, /^"HY신명조"/);
  assert.doesNotMatch(chain, /"휴먼명조"/);
  assert.match(chain, /"Noto Serif KR"/);
  assert.match(chain, /serif$/);
});

test('fontFamilyChainForDisplay는 확인된 로컬 글꼴을 웹 대체 글꼴보다 앞에 둔다', () => {
  const chain = fontFamilyChainForDisplay('휴먼명조', 0, 0, {
    confirmedLocalFonts: ['휴먼명조'],
  });

  assert.match(chain, /^"휴먼명조", "HY신명조"/);
  assert.match(chain, /"Noto Serif KR"/);
  assert.match(chain, /serif$/);
});

test('fontFamilyChainForDisplay는 등록 웹폰트도 시스템 fallback을 붙인다', () => {
  const chain = fontFamilyChainForDisplay('함초롬바탕', 0, 0);

  assert.match(chain, /^"함초롬바탕"/);
  assert.match(chain, /"Noto Serif KR"/);
  assert.match(chain, /serif$/);
});

test('fontFamilyChainForDisplay는 중복 없이 generic font를 그대로 처리한다', () => {
  assert.equal(fontFamilyChainForDisplay('serif', 0, 0), 'serif');
  assert.equal(
    fontFamilyChainForDisplay('없는글꼴', 0, 0),
    '"Malgun Gothic", "Apple SD Gothic Neo", "Noto Sans KR", "Pretendard", sans-serif',
  );
  assert.equal(
    fontFamilyChainForDisplay('없는글꼴', 0, 0, { confirmedLocalFonts: ['없는글꼴'] }),
    '"없는글꼴", "Malgun Gothic", "Apple SD Gothic Neo", "Noto Sans KR", "Pretendard", sans-serif',
  );
});

test('fontFamilyWithFallback 기존 helper는 동일한 fallback 계열을 사용한다', () => {
  assert.equal(
    fontFamilyWithFallback('굴림체'),
    '"굴림체", "GulimChe", "D2Coding", "Noto Sans Mono", monospace',
  );
});
