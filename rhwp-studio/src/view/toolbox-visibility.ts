/**
 * 도구 상자(보기 > 도구 상자 > 기본/서식) 표시 상태를 DOM 에 반영한다.
 *
 * 저장·복원은 `userSettings` 의 `view.toolbarBasic` / `view.toolbarFormat` 가 맡고,
 * 이 모듈은 "설정값 → 루트 표시 상태 + 메뉴 체크 표시" 한 방향만 책임진다.
 *
 * 숨김을 인라인 style 이 아니라 루트 data 속성으로 표현하는 이유는 첫 페인트다 —
 * 같은 속성을 `public/theme-init.js` 가 번들보다 먼저 찍어 숨긴 도구 모음이
 * 잠깐 보였다 사라지는 깜빡임을 없앤다. 규칙은 `src/style.css` 에 있다.
 * DOM 은 인자로 받아 전역 `document` 없이 검증할 수 있게 둔다.
 */

/** 도구 상자 표시 상태 (기본/서식) */
export interface ToolboxVisibility {
  basic: boolean;
  format: boolean;
}

/** 도구 상자 항목: 설정 키 ↔ 메뉴 커맨드 ↔ 루트 data 속성 ↔ 도구 모음 요소 id */
export const TOOLBOX_TARGETS = [
  {
    key: 'basic',
    cmd: 'view:toolbox-basic',
    name: '기본 도구 상자',
    shortcut: 'Ctrl+F1',
    datasetKey: 'toolboxBasic',
    attribute: 'data-toolbox-basic',
    elementId: 'icon-toolbar',
  },
  {
    key: 'format',
    cmd: 'view:toolbox-format',
    name: '서식 도구 상자',
    shortcut: undefined,
    datasetKey: 'toolboxFormat',
    attribute: 'data-toolbox-format',
    elementId: 'style-bar',
  },
] as const satisfies ReadonlyArray<{
  key: keyof ToolboxVisibility;
  cmd: string;
  name: string;
  shortcut?: string;
  datasetKey: string;
  attribute: string;
  elementId: string;
}>;

interface ToolboxMenuItem {
  classList: { toggle(token: string, force: boolean): void };
  getAttribute(name: string): string | null;
  setAttribute(name: string, value: string): void;
}

/** 이 모듈이 쓰는 `document` 표면만 좁혀 받는다. */
export interface ToolboxDom {
  documentElement: { dataset: Record<string, string | undefined> };
  querySelectorAll(selectors: string): Iterable<ToolboxMenuItem>;
}

/** 루트 data 속성 값 — CSS 는 'hidden' 일 때만 숨긴다. */
export function toolboxState(visible: boolean): 'shown' | 'hidden' {
  return visible ? 'shown' : 'hidden';
}

/** 도구 모음 표시 여부와 메뉴·버튼의 접근성 상태를 함께 맞춘다. */
export function applyToolboxVisibility(dom: ToolboxDom, visibility: ToolboxVisibility): void {
  for (const target of TOOLBOX_TARGETS) {
    const visible = visibility[target.key];
    dom.documentElement.dataset[target.datasetKey] = toolboxState(visible);
    for (const item of dom.querySelectorAll(`[data-cmd="${target.cmd}"]`)) {
      item.classList.toggle('active', visible);
      if (item.getAttribute('aria-controls') === target.elementId) {
        const action = visible ? '접기' : '펴기';
        const label = `${target.name} ${action}`;
        item.setAttribute('aria-expanded', String(visible));
        item.setAttribute('aria-label', label);
        item.setAttribute('title', target.shortcut ? `${label} (${target.shortcut})` : label);
      } else {
        item.setAttribute('aria-checked', String(visible));
      }
    }
  }
}
