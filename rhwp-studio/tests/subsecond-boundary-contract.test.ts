import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

test('개발용 런타임 import 실패는 Studio 초기화를 중단시키지 않는다', () => {
  const main = source('src/main.ts');

  assert.match(main, /if \(!import\.meta\.env\.DEV \|\| stopDevelopmentRenderRuntime \|\| !canvasView\) return;/);
  assert.match(main, /try \{[\s\S]*await import\('@\/core\/subsecond-runtime'\)/);
  assert.match(
    main,
    /catch \(error\) \{[\s\S]*console\.warn\('\[main\] 개발용 렌더 코드 교체를 시작하지 못했습니다:'/,
  );
  const canvasViewCreatedAt = main.indexOf('canvasView = new CanvasView(');
  const runtimeStartedAt = main.indexOf('await startDevelopmentRenderRuntime();', canvasViewCreatedAt);
  assert.ok(canvasViewCreatedAt >= 0 && runtimeStartedAt > canvasViewCreatedAt);
});

test('일반 Studio 클래스는 개발용 소켓과 감시자를 소유하지 않는다', () => {
  const bridge = source('src/core/wasm-bridge.ts');
  const canvasView = source('src/view/canvas-view.ts');

  assert.doesNotMatch(bridge, /subsecond-runtime|startRenderCodeReload|getRenderCodeReload|disconnectSubsecondDevtools/);
  assert.doesNotMatch(canvasView, /subsecond-runtime|startRenderCodeReloadWatch|renderCodeReloadWatcher|render-code-reloaded/);
  assert.match(source('src/main.ts'), /\(\) => canvasView\?\.refreshPages\(\)/);
});

test('비활성화된 자동 투명선 경로의 고아 이벤트를 남기지 않는다', () => {
  assert.doesNotMatch(source('src/engine/input-handler.ts'), /transparent-borders-changed/);
  assert.doesNotMatch(source('src/command/commands/view.ts'), /transparent-borders-changed/);
});
