import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

// 탭은 화면 글자가 아니라 ID(data-tab)로 구분한다. 화면 글자는 표시에만 쓰이므로
// 나중에 표시 문자열을 바꿔도(번역 등) 패널 연결·탭 선택 로직이 흔들리지 않는다. (#5852)
const source = (name: string) => readFileSync(new URL(`../src/ui/${name}`, import.meta.url), 'utf8');

const TAB_DIALOGS: Record<string, string[]> = {
  'picture-props-dialog.ts': ['기본', '여백/캡션', '선', '채우기', '글상자', '그림', '그림자', '반사', '네온', '열은 테두리'],
  'equation-props-dialog.ts': ['기본', '여백/캡션', '수식'],
  'char-shape-dialog.ts': ['기본', '확장', '테두리/배경'],
  'para-shape-dialog.ts': ['기본', '확장', '탭 설정', '테두리/배경'],
  'cell-border-bg-dialog.ts': ['테두리', '배경', '대각선'],
  'table-cell-props-dialog.ts': ['기본', '여백/캡션', '테두리', '배경', '표', '셀'],
  'page-border-dialog.ts': ['테두리', '배경'],
};

test('탭이 있는 대화상자는 탭 단추와 패널에 같은 탭 ID(data-tab)를 붙인다', () => {
  for (const file of Object.keys(TAB_DIALOGS)) {
    const src = source(file);
    assert.match(src, /(?:btn|button)\.dataset\.tab = /, `${file}: 탭 단추에 data-tab 이 있어야 한다`);
    assert.match(src, /(?:panel|p)\.dataset\.tab = /, `${file}: 탭 패널에 data-tab 이 있어야 한다`);
  }
});

test('탭 로직은 한국어 탭 이름을 비교하거나 키로 쓰지 않는다', () => {
  for (const file of Object.keys(TAB_DIALOGS)) {
    const src = source(file);
    assert.doesNotMatch(src, /\b(?:name|id|tabName|tabId)\s*===\s*'[가-힣]/, `${file}: 탭 이름 비교가 남아 있다`);
    assert.doesNotMatch(src, /^\s*'[가-힣][^']*':\s*\(\)\s*=>\s*this\.build/m, `${file}: 패널 생성 표가 탭 이름을 키로 쓴다`);
    assert.doesNotMatch(src, /\['border', 'background', 'diagonal'\]\[idx\]/, `${file}: 탭 ID 를 순서 배열에서 따로 꺼낸다`);
  }
});

test('탭 화면 글자는 그대로다(한국어 동작 유지)', () => {
  for (const [file, labels] of Object.entries(TAB_DIALOGS)) {
    const src = source(file);
    for (const label of labels) {
      assert.ok(src.includes(`'${label}'`), `${file}: 탭 글자 '${label}' 이 있어야 한다`);
    }
  }
});

test('E2E 는 탭을 화면 글자가 아니라 탭 ID 로 찾는다', () => {
  const ole = readFileSync(new URL('../e2e/issue-2069-ole-object-selection.test.mjs', import.meta.url), 'utf8');
  const undo = readFileSync(new URL('../e2e/undo-contracts.test.mjs', import.meta.url), 'utf8');
  assert.match(ole, /\.dialog-tab\[data-tab="margin"\]/);
  assert.doesNotMatch(ole, /textContent\?\.trim\(\) === '여백\/캡션'/);
  assert.match(undo, /clickDialogTab\(page, 'equation'\)/);
});
