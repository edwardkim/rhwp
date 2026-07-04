import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

test('로컬 글꼴 감지 모달은 사용자에게 대체 글꼴 표현을 사용한다', () => {
  const modal = source('src/ui/local-fonts-modal.ts');

  assert.match(modal, /대체 글꼴로 보기/);
  assert.match(modal, /대체 글꼴 사용/);
  assert.doesNotMatch(modal, /웹 대체로 보기/);
  assert.doesNotMatch(modal, /웹 대체 사용/);
});

test('외부 웹폰트 비활성 상태는 로컬 글꼴 감지 모달에 표시된다', () => {
  const modal = source('src/ui/local-fonts-modal.ts');
  const main = source('src/main.ts');

  assert.match(modal, /외부 웹폰트 사용 안 함: 켜짐/);
  assert.match(modal, /외부 CDN 폰트를 요청하지 않고/);
  assert.match(main, /disableExternalWebFonts:\s*extensionViewerSettings\.disableExternalWebFonts/);
});
