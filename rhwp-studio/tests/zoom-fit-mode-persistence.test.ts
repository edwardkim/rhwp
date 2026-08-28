import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import {
  normalizeZoomFitMode,
  resolveZoomFitZoom,
  type ZoomFitMode,
} from '../src/view/zoom-fit.ts';
import {
  resolveZoomDialogFitMode,
  zoomFitModeFromChoice,
} from '../src/view/zoom-dialog-state.ts';
import { ViewportManager } from '../src/view/viewport-manager.ts';
import type { EventBus } from '../src/core/event-bus.ts';
import { userSettings } from '../src/core/user-settings.ts';

class FakeEventBus {
  readonly events: Array<{ event: string; args: unknown[] }> = [];

  emit(event: string, ...args: unknown[]): void {
    this.events.push({ event, args });
  }

  fitModes(): unknown[] {
    return this.events
      .filter((entry) => entry.event === 'zoom-fit-mode-changed')
      .map((entry) => entry.args[0]);
  }
}

function newViewportManager(): { vm: ViewportManager; bus: FakeEventBus } {
  const bus = new FakeEventBus();
  return { vm: new ViewportManager(bus as unknown as EventBus), bus };
}

function withMockStorage(run: () => void): void {
  const original = (globalThis as { localStorage?: Storage }).localStorage;
  const store = new Map<string, string>();
  (globalThis as { localStorage?: Storage }).localStorage = {
    get length() {
      return store.size;
    },
    clear: () => store.clear(),
    getItem: (key: string) => store.get(key) ?? null,
    key: (index: number) => Array.from(store.keys())[index] ?? null,
    removeItem: (key: string) => void store.delete(key),
    setItem: (key: string, value: string) => void store.set(key, value),
  } as Storage;
  try {
    run();
  } finally {
    (globalThis as { localStorage?: Storage }).localStorage = original;
  }
}

function storedZoomFitMode(): unknown {
  return JSON.parse(localStorage.getItem('rhwp-settings') ?? '{}').view?.zoomFitMode;
}

test('맞춤 배율은 수치가 아니라 규칙이라 문서마다 그 쪽 크기로 다시 계산한다', () => {
  const a4 = {
    containerWidth: 883,
    containerHeight: 683,
    pageWidth: 793.8,
    pageHeight: 1122.5,
    arrangement: { kind: 'auto' } as const,
  };
  const b5 = { ...a4, pageWidth: 665.2, pageHeight: 944.9 };

  assert.equal(resolveZoomFitZoom('fitWidth', a4), 843 / 793.8);
  assert.equal(resolveZoomFitZoom('fitWidth', b5), 843 / 665.2);
  assert.equal(resolveZoomFitZoom('fitPage', a4), 663 / 1122.5);
  assert.equal(resolveZoomFitZoom('fitPage', b5), 663 / 944.9);
  // 수치 배율은 되돌릴 규칙이 없다 — 지금 배율을 그대로 둔다.
  assert.equal(resolveZoomFitZoom('none', a4), null);
});

test('폭 맞춤은 저장된 쪽 배치의 한 행을 기준으로 되돌린다', () => {
  const metrics = {
    containerWidth: 883,
    containerHeight: 683,
    pageWidth: 800,
    pageHeight: 1000,
    pageGap: 10,
  };

  assert.equal(
    resolveZoomFitZoom('fitWidth', { ...metrics, arrangement: { kind: 'double' } }),
    833 / 1600,
  );
});

test('쪽 맞춤은 저장된 두 쪽·여러 쪽 배치의 전체 블록을 기준으로 되돌린다', () => {
  const metrics = {
    containerWidth: 1600,
    containerHeight: 900,
    pageWidth: 800,
    pageHeight: 1000,
    pageGap: 10,
  };

  assert.equal(
    resolveZoomFitZoom('fitPage', { ...metrics, arrangement: { kind: 'double' } }),
    0.88,
  );
  assert.equal(
    resolveZoomFitZoom('fitPage', {
      ...metrics,
      arrangement: { kind: 'multiple', columns: 4, rows: 1 },
    }),
    1530 / 3200,
  );
});

test('저장값 정규화는 맞춤 두 가지만 통과시킨다', () => {
  assert.equal(normalizeZoomFitMode('fitWidth'), 'fitWidth');
  assert.equal(normalizeZoomFitMode('fitPage'), 'fitPage');
  for (const broken of ['none', 'FITPAGE', '', 0, null, undefined, {}]) {
    assert.equal(normalizeZoomFitMode(broken), 'none');
  }
});

test('대화상자에서 고른 수치 배율은 맞춤을 푼다', () => {
  assert.equal(zoomFitModeFromChoice({ kind: 'fitWidth' }), 'fitWidth');
  assert.equal(zoomFitModeFromChoice({ kind: 'fitPage' }), 'fitPage');
  assert.equal(zoomFitModeFromChoice({ kind: 'preset', percent: 150 }), 'none');
  assert.equal(zoomFitModeFromChoice({ kind: 'custom', percent: 137 }), 'none');
});

test('여러 쪽은 비활성 비율 선택과 무관하게 전체 배열 쪽 맞춤을 저장한다', () => {
  assert.equal(resolveZoomDialogFitMode({
    zoomChoice: { kind: 'preset', percent: 100 },
    arrangement: { kind: 'multiple', columns: 2, rows: 2 },
  }), 'fitPage');
  assert.equal(resolveZoomDialogFitMode({
    zoomChoice: { kind: 'custom', percent: 137 },
    arrangement: { kind: 'multiple', columns: 4, rows: 1 },
  }), 'fitPage');
  assert.equal(resolveZoomDialogFitMode({
    zoomChoice: { kind: 'custom', percent: 137 },
    arrangement: { kind: 'single' },
  }), 'none');
});

test('맞춤으로 정한 배율만 맞춤으로 남고, 수치 배율은 맞춤을 푼다', () => {
  const { vm, bus } = newViewportManager();
  assert.equal(vm.getZoomFitMode(), 'none');

  vm.setZoom(1.4, undefined, 'fitPage');
  assert.equal(vm.getZoomFitMode(), 'fitPage');

  // 가로바·수치 명령은 맞춤 인자를 주지 않는다 — 기본값이 맞춤을 푼다.
  vm.setZoom(2);
  assert.equal(vm.getZoomFitMode(), 'none');
  assert.deepEqual(bus.fitModes(), ['fitPage', 'none']);
});

test('휠 확대처럼 애니메이션으로 가는 배율도 맞춤을 푼다', () => {
  const globals = globalThis as {
    requestAnimationFrame?: unknown;
    cancelAnimationFrame?: unknown;
  };
  const originalRequest = globals.requestAnimationFrame;
  const originalCancel = globals.cancelAnimationFrame;
  globals.requestAnimationFrame = () => 1;
  globals.cancelAnimationFrame = () => {};
  try {
    assertSmoothZoomClearsFitMode();
  } finally {
    globals.requestAnimationFrame = originalRequest;
    globals.cancelAnimationFrame = originalCancel;
  }
});

function assertSmoothZoomClearsFitMode(): void {
  const { vm, bus } = newViewportManager();
  vm.setZoom(1.4, undefined, 'fitWidth');
  assert.equal(vm.getZoomFitMode(), 'fitWidth');

  vm.smoothZoomTo(2.2);
  assert.equal(vm.getZoomFitMode(), 'none');
  assert.deepEqual(bus.fitModes(), ['fitWidth', 'none']);
}

test('같은 맞춤을 다시 골라도 알림은 한 번뿐이다', () => {
  const { vm, bus } = newViewportManager();
  vm.setZoom(1.4, undefined, 'fitPage');
  vm.setZoom(1.5, undefined, 'fitPage');
  assert.deepEqual(bus.fitModes(), ['fitPage']);
});

test('쪽 맞춤/폭 맞춤 선택은 rhwp-settings에 저장된다', () => {
  withMockStorage(() => {
    for (const mode of ['fitWidth', 'fitPage', 'none'] as ZoomFitMode[]) {
      userSettings.setZoomFitMode(mode);
      assert.equal(userSettings.getViewSettings().zoomFitMode, mode);
      assert.equal(storedZoomFitMode(), mode);
    }
  });
});

test('문서를 열면 저장된 맞춤을 그 문서 쪽 크기로 되돌린다', () => {
  const main = readFileSync(new URL('../src/main.ts', import.meta.url), 'utf8');

  // 쪽 크기를 알 수 있는 첫 시점 = canvasView.loadDocument() 직후다.
  assert.match(
    main,
    /await canvasView\?\.loadDocument\(\);[\s\S]{0,200}?applySavedZoomFitMode\(savedZoomFitMode\);/,
  );
  // 되돌릴 맞춤은 로드 전에 읽는다 — 좁은 창 자동 폭 맞춤이 로드 중에 저장값을 지운다.
  assert.match(
    main,
    /const savedZoomFitMode = userSettings\.getViewSettings\(\)\.zoomFitMode;\n\s*await canvasView\?\.loadDocument\(\);/,
  );
  // 뷰포트가 알려주는 맞춤 변화는 그대로 저장으로 흘러야 다음 문서에서 되살아난다.
  assert.match(main, /eventBus\.on\('zoom-fit-mode-changed'/);
  assert.match(main, /userSettings\.setZoomFitMode\(/);
});
