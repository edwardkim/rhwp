import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [#6902] 쪽맞춤에서 문서를 열면 화면이 한 프레임 위아래로 튀었다.
//
// 빈 쪽 자리표시자는 한 쪽 크기라 쪽맞춤에서 세로 스크롤바 없이 딱 맞는데, 문서가 실려
// 여러 쪽이 되면 스크롤바가 생기며 컨테이너 폭이 줄고(1380→1365) ResizeObserver 가 뜬다.
// `onViewportResize` 의 중심 앵커는 **이전 문서/이전 기하** 기준이라 갓 연 문서에는 보존할
// 읽던 자리가 없는데도 스크롤을 옮겼다.
//
//   dev 서버 · 쪽맞춤 · 1400×1000 실측 (rAF 표본)
//     수정 전   firstTop {140, 147}   scrollTop {0, 7}   ← i=24 에서 7px 튀고 i=25 에 복귀
//     수정 후   firstTop {147}        scrollTop {0}      ← 스크롤바 전이(scW 1380→1365)와
//                                                          쪽맞춤 재계산(432→429)은 그대로

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

function section(text: string, startMarker: string, endMarker: string): string {
  const start = text.indexOf(startMarker);
  const end = text.indexOf(endMarker, start);
  assert.ok(start >= 0 && end > start, `${startMarker} 범위를 찾을 수 있어야 한다`);
  return text.slice(start, end);
}

test('문서를 실은 직후 리사이즈 앵커를 끈다', () => {
  const view = source('src/view/canvas-view.ts');
  const load = section(view, '  async loadDocument(): Promise<void> {', '\n  /** WASM 문서 교체');

  const scrollTopIndex = load.indexOf('this.container.scrollTop = 0;');
  const suppressIndex = load.indexOf('this.suppressResizeScrollAnchor = true;');
  assert.ok(scrollTopIndex >= 0, '문서 로드는 맨 위로 맞춰야 한다');
  assert.ok(
    suppressIndex > scrollTopIndex,
    '맨 위로 맞춘 뒤 그 값을 지킬 수 있도록 앵커를 꺼야 한다',
  );
});

test('자리표시자로 갈아타는 두 진입점도 앵커를 끈다', () => {
  const view = source('src/view/canvas-view.ts');
  for (const [head, tail] of [
    ['  prepareDocumentLoad(): void {', '\n  /**'],
    ['  showBlankPage(): void {', '\n  /**'],
  ] as const) {
    const body = section(view, head, tail);
    assert.match(
      body,
      /this\.suppressResizeScrollAnchor = true;/,
      `${head.trim()} 도 앵커를 꺼야 한다 — 자리표시자 전이가 스크롤바 유무를 뒤집는다`,
    );
  }
});

test('리사이즈 앵커는 억제 플래그를 한 번 쓰고 스스로 거둔다', () => {
  const view = source('src/view/canvas-view.ts');
  const resize = section(view, '  private onViewportResize(): void {', '\n  /**');

  assert.match(
    resize,
    /const suppressAnchor = this\.suppressResizeScrollAnchor;\s*\n\s*this\.suppressResizeScrollAnchor = false;/,
    '읽은 뒤 곧바로 내려 다음 리사이즈까지 남지 않아야 한다',
  );
  assert.match(
    resize,
    /const canPreserveCenter =\s*\n?\s*!suppressAnchor &&/,
    '억제 중에는 중심 앵커를 계산하지 않아야 한다',
  );
});

test('사용자가 스크롤하면 억제를 거둔다', () => {
  const view = source('src/view/canvas-view.ts');
  const subscription = section(
    view,
    "eventBus.on('viewport-scroll', () => {",
    '}),',
  );

  assert.match(
    subscription,
    /this\.suppressResizeScrollAnchor = false;/,
    '한 번이라도 스크롤했으면 보존할 읽던 자리가 생기므로 앵커를 되살려야 한다',
  );
});
