import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

// 책갈피 대화상자는 commands/insert.ts 에서 모듈 전역 싱글턴으로 재사용된다.
// show() 는 build() 로 DOM 을 매번 새로 만들기 때문에 정렬 라디오는 항상
// '위치(P)' 가 선택된 상태로 나타나지만, 인스턴스 필드 sortMode 는 이전 세션
// 값('name')이 남는다 → 라디오 표시와 실제 목록 정렬이 어긋난다.
// show() 는 refreshList() 전에 sortMode 를 초기값 'position' 으로 리셋해야 한다.
test('책갈피 대화상자는 열릴 때마다 sortMode를 라디오 초기 상태(position)로 리셋한다', () => {
  const dialog = source('src/ui/bookmark-dialog.ts');
  const showStart = dialog.indexOf('show(): void');
  assert.notEqual(showStart, -1, 'show() 를 찾을 수 있어야 한다');
  const hideStart = dialog.indexOf('hide(): void', showStart);
  const showBody = dialog.slice(showStart, hideStart === -1 ? undefined : hideStart);

  assert.match(
    showBody,
    /this\.sortMode\s*=\s*'position'/,
    'show() 는 refreshList() 전에 sortMode 를 position 으로 리셋해야 한다',
  );

  const resetIdx = showBody.search(/this\.sortMode\s*=\s*'position'/);
  const refreshIdx = showBody.indexOf('this.refreshList()');
  assert.ok(
    resetIdx !== -1 && refreshIdx !== -1 && resetIdx < refreshIdx,
    'sortMode 리셋은 refreshList() 호출보다 앞서야 한다',
  );
});
