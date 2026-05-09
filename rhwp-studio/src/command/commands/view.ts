import type { CommandDef } from '../types';
import { GridSettingsDialog } from '../../ui/grid-settings-dialog';

/** 배율 고정값 커맨드 생성 헬퍼 */
function zoomLevel(pct: number): CommandDef {
  return {
    id: `view:zoom-${pct}`,
    label: `${pct}%`,
    execute(services) {
      services.getViewportManager()?.setZoom(pct / 100);
    },
  };
}

export const viewCommands: CommandDef[] = [
  {
    id: 'view:zoom-in',
    label: '확대',
    icon: 'icon-zoom-menu-in',
    shortcutLabel: 'Shift+Num +',
    execute(services) {
      const vm = services.getViewportManager();
      if (vm) vm.setZoom(vm.getZoom() + 0.1);
    },
  },
  {
    id: 'view:zoom-out',
    label: '축소',
    icon: 'icon-zoom-menu-out',
    shortcutLabel: 'Shift+Num -',
    execute(services) {
      const vm = services.getViewportManager();
      if (vm) vm.setZoom(vm.getZoom() - 0.1);
    },
  },
  {
    id: 'view:zoom-fit-page',
    label: '쪽 맞춤',
    execute(services) {
      const vm = services.getViewportManager();
      if (!vm || services.wasm.pageCount === 0) return;
      const container = document.getElementById('scroll-container')!;
      const containerH = container.clientHeight - 40;
      const containerW = container.clientWidth - 40;
      const pi = services.wasm.getPageInfo(0);
      // pi.width/height는 이미 px 단위 (96dpi 기준)
      vm.setZoom(Math.max(0.1, Math.min(containerW / pi.width, containerH / pi.height, 4.0)));
    },
  },
  {
    id: 'view:zoom-fit-width',
    label: '폭 맞춤',
    execute(services) {
      const vm = services.getViewportManager();
      if (!vm || services.wasm.pageCount === 0) return;
      const container = document.getElementById('scroll-container')!;
      const containerW = container.clientWidth - 40;
      const pi = services.wasm.getPageInfo(0);
      // pi.width는 이미 px 단위 (96dpi 기준)
      vm.setZoom(Math.max(0.1, Math.min(containerW / pi.width, 4.0)));
    },
  },
  zoomLevel(50),
  zoomLevel(75),
  zoomLevel(100),
  zoomLevel(125),
  zoomLevel(150),
  zoomLevel(200),
  zoomLevel(300),
  // ─── 보기 메뉴: 표시/숨기기 ─────────────────────────
  {
    id: 'view:ctrl-mark',
    label: '조판 부호',
    icon: 'icon-ctrl-mark',
    shortcutLabel: 'Ctrl+G,C',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ctx = services.getContext();
      const next = !ctx.showControlCodes;
      // 조판부호 ON → 문단부호도 ON (한컴 기준: 조판부호는 문단부호를 포함)
      services.wasm.setShowControlCodes(next);
      services.wasm.setShowParagraphMarks(next);
      document.querySelectorAll('[data-cmd="view:ctrl-mark"]').forEach(el => {
        el.classList.toggle('active', next);
      });
      // 문단부호 버튼도 연동
      document.querySelectorAll('[data-cmd="view:para-mark"]').forEach(el => {
        el.classList.toggle('active', next);
      });
      services.eventBus.emit('document-changed');
    },
  },
  (() => {
    let showParaMarks = false;
    return {
      id: 'view:para-mark',
      label: '문단 부호',
      icon: 'icon-para-mark',
      canExecute: (ctx) => ctx.hasDocument,
      execute(services) {
        showParaMarks = !showParaMarks;
        services.wasm.setShowParagraphMarks(showParaMarks);
        document.querySelectorAll('[data-cmd="view:para-mark"]').forEach(el => {
          el.classList.toggle('active', showParaMarks);
        });
        services.eventBus.emit('document-changed');
      },
    } satisfies CommandDef;
  })(),
  {
    id: 'view:border-transparent',
    label: '투명 선',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      // WASM 실제 상태를 읽어 토글 — 셀 진입 자동 ON 등으로 인한 초기값 불일치 방지
      const next = !services.wasm.getShowTransparentBorders();
      services.wasm.setShowTransparentBorders(next);
      document.querySelectorAll('[data-cmd="view:border-transparent"]').forEach(el => {
        el.classList.toggle('active', next);
      });
      services.eventBus.emit('transparent-borders-changed', next);
      services.eventBus.emit('document-changed');
    },
  },
  (() => {
    let clipEnabled = true; // 기본: 잘림 적용 (편집용지 클립)
    return {
      id: 'view:toggle-clip',
      label: '잘림 보기',
      canExecute: (ctx) => ctx.hasDocument,
      execute(services) {
        clipEnabled = !clipEnabled;
        services.wasm.setClipEnabled(clipEnabled);
        document.querySelectorAll('[data-cmd="view:toggle-clip"]').forEach(el => {
          el.classList.toggle('active', !clipEnabled);
        });
        services.eventBus.emit('document-changed');
      },
    } satisfies CommandDef;
  })(),
  {
    id: 'view:grid-settings',
    label: '격자 설정',
    icon: 'icon-grid',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      if (!ih) return;
      new GridSettingsDialog(ih.getGridStepMm(), (mm) => ih.setGridStep(mm)).show();
    },
  },
  (() => {
    let visible: boolean | null = null;
    return {
      id: 'view:toolbox-basic',
      label: '기본',
      execute() {
        const el = document.getElementById('icon-toolbar');
        if (!el) return;
        if (visible === null) visible = getComputedStyle(el).display !== 'none';
        visible = !visible;
        el.style.display = visible ? '' : 'none';
        document.querySelectorAll('[data-cmd="view:toolbox-basic"]').forEach(btn => {
          btn.classList.toggle('active', visible!);
        });
      },
    } satisfies CommandDef;
  })(),
  (() => {
    let visible: boolean | null = null;
    return {
      id: 'view:toolbox-format',
      label: '서식',
      execute() {
        const el = document.getElementById('style-bar');
        if (!el) return;
        if (visible === null) visible = getComputedStyle(el).display !== 'none';
        visible = !visible;
        el.style.display = visible ? '' : 'none';
        document.querySelectorAll('[data-cmd="view:toolbox-format"]').forEach(btn => {
          btn.classList.toggle('active', visible!);
        });
      },
    } satisfies CommandDef;
  })(),
];
