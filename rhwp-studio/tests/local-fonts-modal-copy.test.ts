import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { assertShowsText, assertDoesNotShowText } from './support/i18n-text.ts';
const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

test('로컬 글꼴 감지 모달은 사용자에게 대체 글꼴 표현을 사용한다', () => {
  const modal = source('src/ui/local-fonts-modal.ts');

  assertShowsText(modal, '대체 글꼴로 보기');
  assertShowsText(modal, '대체 글꼴 사용');
  assert.doesNotMatch(modal, /웹 대체로 보기/);
  assert.doesNotMatch(modal, /웹 대체 사용/);
});

test('외부 웹폰트 비활성 상태는 로컬 글꼴 감지 모달에 표시된다', () => {
  const modal = source('src/ui/local-fonts-modal.ts');

  assertShowsText(modal, '외부 웹폰트 사용 안 함: 켜짐');
  assertShowsText(modal, '외부 CDN 폰트를 요청하지 않고');
});

test('저장된 감지 결과가 없을 때 문서 글꼴 감지 여부를 묻는다', () => {
  const main = source('src/main.ts');

  assert.match(main, /if \(getLocalFontState\(\)\.stored\) return/);
  assert.match(main, /await showLocalFontsModalIfNeeded\(report, extensionViewerSettings\)/);
  assert.match(main, /if \(choice !== 'detect'\) return/);
});
