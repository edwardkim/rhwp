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

test('KoPub바탕체 weight face는 고정폭이 아니라 비례폭 serif로 분류한다', () => {
  for (const fontName of [
    'KoPub바탕체 Light',
    'KoPub바탕체 Medium',
    'KoPub바탕체 Bold',
    'KoPubBatang Light',
    'KoPubBatang Medium',
    'KoPubBatang Bold',
  ]) {
    const chain = fontFamilyChainForDisplay(fontName, 0, 0);
    assert.match(chain, new RegExp(`^"${fontName}", "Batang"`, 'u'), fontName);
    assert.match(chain, /"Noto Serif KR"/u, fontName);
    assert.match(chain, /serif$/u, fontName);
    assert.doesNotMatch(chain, /monospace/u, fontName);
  }
});

test('정부상징 legacy face는 exact, ROKG successor, 문서 대체 face 순서를 따른다', () => {
  const legacy = '정부상징 부처명_16040911';
  const documentFallbackFamilies = ['한컴바탕'];

  const exact = fontFamilyChainForDisplay(legacy, 1, 0, {
    confirmedLocalFonts: [legacy, '대한민국정부상징체 R'],
    documentFallbackFamilies,
  });
  assert.match(exact, /^"정부상징 부처명_16040911", "대한민국정부상징체 R", "한컴바탕"/u);

  const successor = fontFamilyChainForDisplay(legacy, 1, 0, {
    confirmedLocalFonts: ['대한민국정부상징체 R'],
    documentFallbackFamilies,
  });
  assert.match(successor, /^"정부상징 부처명_16040911", "대한민국정부상징체 R", "한컴바탕"/u);

  const documentSubstitute = fontFamilyChainForDisplay(legacy, 1, 0, {
    confirmedLocalFonts: [],
    documentFallbackFamilies,
  });
  assert.match(documentSubstitute, /^"정부상징 부처명_16040911", "한컴바탕"/u);
});

test('ROKG successor는 정부상징 legacy 이름에만 적용한다', () => {
  const chain = fontFamilyChainForDisplay('일반 제목체', 1, 0, {
    confirmedLocalFonts: ['ROKG'],
  });

  assert.doesNotMatch(chain, /ROKG/u);
});

test('stand-in 웹폰트만 있는 legacy 이름은 설치된 치환 대상 face를 앞에 둔다', () => {
  // `한양중고딕`은 studio 공급 카탈로그에 있어 resolveFont가 그대로 돌려준다.
  // 그 공급은 번들 Noto Sans KR stand-in이므로 설치 face `HY중고딕`이 먼저 와야 한다.
  const installed = fontFamilyChainForDisplay('한양중고딕', 0, 0, {
    confirmedLocalFonts: ['HY중고딕'],
  });
  assert.match(installed, /^"HY중고딕", "한양중고딕", "Malgun Gothic"/u);
  assert.match(installed, /sans-serif$/u);
});

test('치환 대상이 설치돼 있지 않으면 legacy 이름의 기존 체인을 유지한다', () => {
  const chain = fontFamilyChainForDisplay('한양중고딕', 0, 0, {
    confirmedLocalFonts: [],
  });

  assert.equal(
    chain,
    '"한양중고딕", "Malgun Gothic", "Apple SD Gothic Neo", "Noto Sans KR", "Pretendard", sans-serif',
  );
});

test('요청 이름 자체가 설치돼 있으면 치환 대상을 앞에 두지 않는다', () => {
  const chain = fontFamilyChainForDisplay('굴림', 0, 0, {
    confirmedLocalFonts: ['굴림', '새굴림'],
  });

  assert.match(chain, /^"굴림"/u);
});
