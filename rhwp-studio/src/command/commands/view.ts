import type { CommandDef } from '../types';
import { setThemeMode, syncThemeMenu, type EffectiveTheme } from '../../core/theme';
import { userSettings, type ThemeMode } from '../../core/user-settings';
import { GridSettingsDialog } from '../../ui/grid-settings-dialog';
import {
  type GridOffsetMm,
  type GridViewSettings,
  getGridViewSettings,
  setGridViewSettings,
  toggleGridVisibility,
} from '../../view/grid-settings';
import { HWPUNIT_PER_MM } from '../../core/hwp-constants';

const PX_TO_MM = 25.4 / 96;

/** 배율 고정값 커맨드 생성 헬퍼 */
function zoomLevel(pct: number, shortcutLabel?: string): CommandDef {
  return {
    id: `view:zoom-${pct}`,
    label: `${pct}%`,
    shortcutLabel,
    execute(services) {
      services.getViewportManager()?.setZoom(pct / 100);
    },
  };
}

function themeModeCommand(mode: ThemeMode, label: string): CommandDef {
  return {
    id: `view:theme-${mode}`,
    label,
    execute(services) {
      const effective: EffectiveTheme = setThemeMode(mode);
      syncThemeMenu(mode);
      services.eventBus.emit('theme-changed', { mode, effective });
      services.eventBus.emit('document-view-changed');
    },
  };
}

export function syncTextMarkMenu(showControlCodes: boolean, showParagraphMarks: boolean): void {
  document.querySelectorAll('[data-cmd="view:ctrl-mark"]').forEach(el => {
    el.classList.toggle('active', showControlCodes);
  });
  document.querySelectorAll('[data-cmd="view:para-mark"]').forEach(el => {
    el.classList.toggle('active', showParagraphMarks);
  });
}

function refreshCaretAfterViewChange(services: Parameters<CommandDef['execute']>[0]): void {
  const inputHandler = services.getInputHandler() as any;
  inputHandler?.updateCaret?.(true);
  requestAnimationFrame(() => inputHandler?.updateCaret?.(true));
}

interface GridOriginMetrics {
  defaults: Record<'page' | 'paper', GridOffsetMm>;
  bases: Record<'page' | 'paper', GridOffsetMm>;
}

function getGridOriginMetrics(services: Parameters<CommandDef['execute']>[0]): GridOriginMetrics {
  let pageIndex = 0;
  const ih = services.getInputHandler();
  const cursor = ih ? (ih as any).cursor : null;
  if (typeof cursor?.rect?.pageIndex === 'number') {
    pageIndex = cursor.rect.pageIndex;
  }

  const pageInfo = services.wasm.getPageInfo(pageIndex);
  const documentInfo = services.wasm.getDocumentInfo();
  const sectionDef = services.wasm.getSectionDef(pageInfo.sectionIndex ?? 0);
  const pageBorderFill = services.wasm.getPageBorderFill(pageInfo.sectionIndex ?? 0);
  const rawPageDef = services.wasm.getPageDef(pageInfo.sectionIndex ?? 0);
  const paperX = roundMm(rawPageDef.marginLeft / HWPUNIT_PER_MM);
  const paperBaseY = roundMm((rawPageDef.marginTop + rawPageDef.marginHeader) / HWPUNIT_PER_MM);
  const hwp3PageYOffsetY = documentInfo.hwp3Variant && pageBorderFill.basis === 'page'
    ? roundMm(sectionDef.columnSpacing / HWPUNIT_PER_MM)
    : 0;
  const paperDefaultY = hwp3PageYOffsetY > 0
    ? roundMm(
      roundMm(rawPageDef.marginTop / HWPUNIT_PER_MM)
      + roundMm(rawPageDef.marginHeader / HWPUNIT_PER_MM)
      + hwp3PageYOffsetY,
    )
    : paperBaseY;
  const pageDefaultY = roundMm(paperDefaultY - paperBaseY);

  return {
    defaults: {
      page: { x: 0, y: pageDefaultY },
      paper: {
        x: paperX,
        y: paperDefaultY,
      },
    },
    bases: {
      page: {
        x: paperX,
        y: paperBaseY,
      },
      paper: { x: 0, y: 0 },
    },
  };
}

function roundMm(value: number): number {
  return Math.round(value * 100) / 100;
}

function applyGridDefaults(settings: GridViewSettings, defaults: GridOriginMetrics['defaults']): GridViewSettings {
  if (!closeMm(settings.offsetXmm, 0) || !closeMm(settings.offsetYmm, 0)) {
    return settings;
  }
  const defaultOffset = defaults[settings.origin];
  return {
    ...settings,
    offsetXmm: defaultOffset.x,
    offsetYmm: defaultOffset.y,
  };
}

function closeMm(a: number, b: number): boolean {
  return Number.isFinite(a) && Math.abs(a - b) < 0.01;
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
    shortcutLabel: 'Ctrl+G,P',
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
    shortcutLabel: 'Ctrl+G,W',
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
  zoomLevel(100, 'Ctrl+G,Q'),
  zoomLevel(125),
  zoomLevel(150),
  zoomLevel(200),
  zoomLevel(300),
  themeModeCommand('system', '시스템 설정'),
  themeModeCommand('light', '밝게'),
  themeModeCommand('dark', '어둡게'),
  // ─── 보기 메뉴: 표시/숨기기 ─────────────────────────
  {
    id: 'view:form-mode',
    label: '양식 모드',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const next = services.getContext().isFormMode ? 'normal' : 'form';
      services.setEditMode(next);
    },
  },
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
      userSettings.setShowControlCodes(next);
      userSettings.setShowParagraphMarks(next);
      syncTextMarkMenu(next, next);
      refreshCaretAfterViewChange(services);
      services.eventBus.emit('document-view-changed');
    },
  },
  {
    id: 'view:para-mark',
    label: '문단 부호',
    icon: 'icon-para-mark',
    shortcutLabel: 'Ctrl+G,T',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ctx = services.getContext();
      const next = !ctx.showParagraphMarks;
      services.wasm.setShowParagraphMarks(next);
      userSettings.setShowParagraphMarks(next);
      syncTextMarkMenu(ctx.showControlCodes, next);
      refreshCaretAfterViewChange(services);
      services.eventBus.emit('document-view-changed');
    },
  },
  {
    id: 'view:border-transparent',
    label: '투명 선',
    shortcutLabel: 'Alt+V,T',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      // WASM 실제 상태를 읽어 토글 — 셀 진입 자동 ON 등으로 인한 초기값 불일치 방지
      const next = !services.wasm.getShowTransparentBorders();
      services.wasm.setShowTransparentBorders(next);
      document.querySelectorAll('[data-cmd="view:border-transparent"]').forEach(el => {
        el.classList.toggle('active', next);
      });
      services.eventBus.emit('transparent-borders-changed', next);
      services.eventBus.emit('document-view-changed');
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
        services.eventBus.emit('document-view-changed');
      },
    } satisfies CommandDef;
  })(),
  {
    id: 'view:toggle-grid',
    label: '격자 보기',
    icon: 'icon-grid',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const next = toggleGridVisibility();
      document.querySelectorAll('[data-cmd="view:toggle-grid"]').forEach(el => {
        el.classList.toggle('active', next.visible);
      });
      services.eventBus.emit('grid-view-changed', next);
    },
  },
  {
    id: 'view:grid-settings',
    label: '격자 설정',
    icon: 'icon-grid',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
      const ih = services.getInputHandler();
      const originMetrics = getGridOriginMetrics(services);
      new GridSettingsDialog(
        applyGridDefaults(getGridViewSettings(), originMetrics.defaults),
        originMetrics.bases,
        ih?.getGridStepMm() ?? 3,
        (settings, moveStepMm) => {
          const next = setGridViewSettings(settings);
          ih?.setGridStep(moveStepMm);
          document.querySelectorAll('[data-cmd="view:toggle-grid"]').forEach(el => {
            el.classList.toggle('active', next.visible);
          });
          services.eventBus.emit('grid-view-changed', next);
        },
      ).show();
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
