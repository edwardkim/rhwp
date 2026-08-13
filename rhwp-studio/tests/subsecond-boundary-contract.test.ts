import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

function method(sourceText: string, startToken: string, endToken: string): string {
  const start = sourceText.indexOf(startToken);
  const end = sourceText.indexOf(endToken, start);
  assert.ok(start >= 0 && end > start, `${startToken} 범위를 찾을 수 있어야 한다`);
  return sourceText.slice(start, end);
}

test('개발용 런타임 import 실패는 Studio WASM 초기화를 중단시키지 않는다', () => {
  const bridge = method(
    source('src/core/wasm-bridge.ts'),
    '  private async startRenderCodeReload(',
    '\n  /**\n   * 렌더 코드 교체 능력.',
  );

  assert.match(bridge, /try \{[\s\S]*await import\('\.\/subsecond-runtime'\)/);
  assert.match(
    bridge,
    /catch \(error\) \{[\s\S]*console\.warn\('\[WasmBridge\] 개발용 렌더 코드 교체를 시작하지 못했습니다:'/,
  );
});

test('CanvasView는 초기화 순서가 틀린 개발용 교체 구성을 조용히 끄지 않는다', () => {
  const canvasView = method(
    source('src/view/canvas-view.ts'),
    '  private startRenderCodeReloadWatch(): void {',
    '\n  /** 문서 로드 후 호출',
  );

  assert.match(
    canvasView,
    /if \(!renderCodeReload\) \{[\s\S]*console\.warn\([\s\S]*await wasm\.initialize\(\)[\s\S]*return;/,
  );
});

test('비활성화된 자동 투명선 경로의 고아 이벤트를 남기지 않는다', () => {
  assert.doesNotMatch(source('src/engine/input-handler.ts'), /transparent-borders-changed/);
  assert.doesNotMatch(source('src/command/commands/view.ts'), /transparent-borders-changed/);
});
