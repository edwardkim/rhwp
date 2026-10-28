/**
 * 오른쪽 도킹 패널 자리 — 플러그인이 화면을 얻는 유일한 통로.
 *
 * 패널은 `#editor-area` grid 의 셋째 열에 붙는다. 편집 영역(`#scroll-container`) 크기는
 * `ViewportManager` 의 `ResizeObserver` 가 이미 추종하므로 여기서 재배치를 부르지 않는다.
 *
 * 상태(어느 패널이 열렸나·폭)는 `SidePanelController` 가, DOM 은 `SidePanelView` 가 맡는다.
 * 둘을 나눈 이유는 상태 계약을 DOM 없는 단위 테스트로 고정하기 위해서다.
 */
import type { PluginSidePanelSpec } from '@/plugin/types';

export const SIDE_PANEL_MIN_WIDTH = 260;
export const SIDE_PANEL_MAX_WIDTH = 720;
const DEFAULT_WIDTH = 360;
const WIDTH_STORAGE_PREFIX = 'rhwp-side-panel-width:';

export interface SidePanelView {
  /** 패널 본문 컨테이너를 만든다. 패널마다 한 번. */
  createContainer(id: string): HTMLElement;
  removeContainer(id: string): void;
  show(id: string, title: string, width: number): void;
  hide(): void;
  /** 닫기 단추·Esc 로 닫으려 할 때 */
  onCloseRequest(cb: () => void): void;
  /** 사용자가 경계를 끌어 폭을 바꿨을 때 */
  onResize(cb: (width: number) => void): void;
}

export interface WidthStore {
  get(id: string): number | null;
  set(id: string, width: number): void;
}

/** 저장소 접근이 막힌 환경(사생활 보호 창 등)에서도 동작한다 — 폭만 기억하지 못할 뿐이다. */
export const localStorageWidthStore: WidthStore = {
  get(id) {
    try {
      const raw = localStorage.getItem(WIDTH_STORAGE_PREFIX + id);
      const value = raw === null ? NaN : Number(raw);
      return Number.isFinite(value) ? value : null;
    } catch {
      return null;
    }
  },
  set(id, width) {
    try { localStorage.setItem(WIDTH_STORAGE_PREFIX + id, String(Math.round(width))); }
    catch { /* 기억하지 못해도 패널은 동작한다 */ }
  },
};

export function clampPanelWidth(width: number): number {
  if (!Number.isFinite(width)) return DEFAULT_WIDTH;
  return Math.min(SIDE_PANEL_MAX_WIDTH, Math.max(SIDE_PANEL_MIN_WIDTH, Math.round(width)));
}

interface PanelEntry {
  spec: PluginSidePanelSpec;
  container: HTMLElement | null;
}

/** 플러그인 호스트가 쓰는 표면. */
export interface SidePanelRegistry {
  register(spec: PluginSidePanelSpec): () => void;
  toggle(id: string, open?: boolean): boolean;
  isOpen(id: string): boolean;
}

export class SidePanelController implements SidePanelRegistry {
  private readonly panels = new Map<string, PanelEntry>();
  private activeId: string | null = null;
  private readonly view: SidePanelView;
  private readonly widths: WidthStore;

  constructor(view: SidePanelView, widths: WidthStore = localStorageWidthStore) {
    this.view = view;
    this.widths = widths;
    view.onCloseRequest(() => {
      if (this.activeId) this.toggle(this.activeId, false);
    });
    view.onResize((width) => {
      if (this.activeId) this.widths.set(this.activeId, clampPanelWidth(width));
    });
  }

  register(spec: PluginSidePanelSpec): () => void {
    if (this.panels.has(spec.id)) throw new Error(`이미 등록된 패널입니다: ${spec.id}`);
    const entry: PanelEntry = { spec, container: null };
    this.panels.set(spec.id, entry);
    let disposed = false;
    return () => {
      if (disposed) return;
      disposed = true;
      if (this.activeId === spec.id) this.toggle(spec.id, false);
      this.panels.delete(spec.id);
      if (entry.container) {
        try { spec.unmount?.(); } catch (error) { console.error('[SidePanel] unmount 실패', error); }
        this.view.removeContainer(spec.id);
      }
    };
  }

  toggle(id: string, open?: boolean): boolean {
    const entry = this.panels.get(id);
    if (!entry) throw new Error(`등록되지 않은 패널입니다: ${id}`);
    const wantOpen = open ?? this.activeId !== id;
    if (wantOpen === (this.activeId === id)) return wantOpen;

    if (!wantOpen) {
      this.activeId = null;
      this.view.hide();
      this.notify(entry, false);
      return false;
    }

    // 패널은 한 번에 하나다 — 다른 패널이 열려 있으면 먼저 닫는다.
    const previous = this.activeId ? this.panels.get(this.activeId) : undefined;
    if (previous) {
      this.activeId = null;
      this.notify(previous, false);
    }
    if (!entry.container) {
      entry.container = this.view.createContainer(id);
      entry.spec.mount(entry.container);
    }
    const width = clampPanelWidth(this.widths.get(id) ?? entry.spec.defaultWidth ?? DEFAULT_WIDTH);
    this.activeId = id;
    this.view.show(id, entry.spec.title, width);
    this.notify(entry, true);
    return true;
  }

  isOpen(id: string): boolean {
    return this.activeId === id;
  }

  private notify(entry: PanelEntry, open: boolean): void {
    try { entry.spec.onVisibilityChange?.(open); }
    catch (error) { console.error('[SidePanel] onVisibilityChange 실패', error); }
  }
}

/**
 * `#side-panel-host` 위의 DOM 구현.
 * `restoreFocus` 는 포커스를 쥔 채 패널이 닫힐 때 부른다 — 숨은 요소에 포커스가 남으면 키 입력이 사라진다.
 */
export function createDomSidePanelView(
  root: HTMLElement,
  closeLabel: string,
  restoreFocus: () => void,
): SidePanelView {
  root.replaceChildren();
  root.hidden = true;

  const resizer = document.createElement('div');
  resizer.className = 'spl-resizer';
  resizer.setAttribute('role', 'separator');
  resizer.setAttribute('aria-orientation', 'vertical');

  const header = document.createElement('div');
  header.className = 'spl-header';
  const title = document.createElement('span');
  title.className = 'spl-title';
  const close = document.createElement('button');
  close.type = 'button';
  close.className = 'spl-close';
  close.setAttribute('aria-label', closeLabel);
  close.title = closeLabel;
  close.textContent = '×';
  header.append(title, close);

  const body = document.createElement('div');
  body.className = 'spl-body';
  root.append(resizer, header, body);

  const containers = new Map<string, HTMLElement>();
  let closeCb: () => void = () => {};
  let resizeCb: (width: number) => void = () => {};

  close.addEventListener('click', () => closeCb());
  root.addEventListener('keydown', (event) => {
    if (event.key === 'Escape' && !event.defaultPrevented) {
      event.preventDefault();
      closeCb();
    }
  });

  // 왼쪽 경계를 끌어 폭을 바꾼다. 패널이 오른쪽에 있으므로 왼쪽으로 끌수록 넓어진다.
  resizer.addEventListener('pointerdown', (down) => {
    down.preventDefault();
    const startX = down.clientX;
    const startWidth = root.getBoundingClientRect().width;
    resizer.setPointerCapture(down.pointerId);
    const move = (event: PointerEvent) => {
      root.style.width = `${clampPanelWidth(startWidth + startX - event.clientX)}px`;
    };
    const up = (event: PointerEvent) => {
      resizer.releasePointerCapture(event.pointerId);
      resizer.removeEventListener('pointermove', move);
      resizer.removeEventListener('pointerup', up);
      resizeCb(root.getBoundingClientRect().width);
    };
    resizer.addEventListener('pointermove', move);
    resizer.addEventListener('pointerup', up);
  });

  return {
    createContainer(id) {
      const container = document.createElement('div');
      container.className = 'spl-panel';
      container.dataset.panelId = id;
      container.hidden = true;
      body.append(container);
      containers.set(id, container);
      return container;
    },
    removeContainer(id) {
      containers.get(id)?.remove();
      containers.delete(id);
    },
    show(id, panelTitle, width) {
      for (const [key, container] of containers) container.hidden = key !== id;
      title.textContent = panelTitle;
      root.setAttribute('aria-label', panelTitle);
      root.style.width = `${width}px`;
      root.hidden = false;
    },
    hide() {
      const hadFocus = root.contains(document.activeElement);
      root.hidden = true;
      if (hadFocus) restoreFocus();
      for (const container of containers.values()) container.hidden = true;
    },
    onCloseRequest(cb) { closeCb = cb; },
    onResize(cb) { resizeCb = cb; },
  };
}
