import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

// 수식 편집기는 commands/insert.ts 에서 모듈 전역 싱글턴으로 재사용되고,
// build() 는 built 플래그로 최초 1회만 실행되어 DOM 이 인스턴스에 남는다.
// open() 은 scriptArea/fontSize/color/mode 는 리셋하지만 기호 검색 입력
// (searchInput)과 검색 결과 드롭다운(searchResults)은 건드리지 않아,
// 닫았다 다시 열면 이전 검색어와 펼쳐진 결과 목록이 그대로 남는다.
test('수식 편집기 대화상자는 열릴 때마다 기호 검색 입력과 결과 드롭다운을 초기화한다', () => {
  const dialog = source('src/ui/equation-editor-dialog.ts');
  const openStart = dialog.indexOf('open(sec: number');
  assert.notEqual(openStart, -1, 'open() 을 찾을 수 있어야 한다');
  const openEnd = dialog.indexOf('hide(): void', openStart);
  const openBody = dialog.slice(openStart, openEnd === -1 ? undefined : openEnd);

  assert.match(
    openBody,
    /this\.searchInput\.value\s*=\s*''/,
    'open() 은 이전 검색어를 비워야 한다',
  );
  assert.match(
    openBody,
    /this\.searchResults\.style\.display\s*=\s*'none'/,
    'open() 은 검색 결과 드롭다운을 닫아야 한다',
  );
});
