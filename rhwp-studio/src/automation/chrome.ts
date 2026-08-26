/**
 * 창 제어 — 메뉴·툴바·상태표시줄 표시 토글.
 *
 * **표시만 끈다.** 커맨드 레지스트리와 단축키는 그대로 살아 있어 `automation.execute` 가 계속
 * 동작한다 — "UI 없는 헤드리스 편집기" 구성이 이것으로 선다.
 *
 * 편집 영역 크기는 `ViewportManager` 의 `ResizeObserver` 가 이미 추종하므로 여기서 재배치를
 * 지시하지 않는다.
 */
import { CHROME_HIDDEN_CLASS, type ChromeVisibility } from './types';

const ROOT_ID = 'studio-root';

function root(): HTMLElement | null {
  return document.getElementById(ROOT_ID);
}

/** 현재 표시 상태. 요소가 없으면 전부 보인다고 본다. */
export function getChromeVisibility(): ChromeVisibility {
  const el = root();
  const visible = (key: keyof ChromeVisibility) =>
    !el || !el.classList.contains(CHROME_HIDDEN_CLASS[key]);
  return { menu: visible('menu'), toolbar: visible('toolbar'), statusbar: visible('statusbar') };
}

/**
 * 표시 상태를 바꾼다. 넘기지 않은 항목은 그대로 둔다.
 * 반환값은 적용 후의 실제 상태다 — 호출자가 요청과 결과를 대조할 수 있어야 한다.
 */
export function setChromeVisibility(next: Partial<ChromeVisibility>): ChromeVisibility {
  const el = root();
  if (el) {
    for (const key of Object.keys(CHROME_HIDDEN_CLASS) as Array<keyof ChromeVisibility>) {
      const wanted = next[key];
      if (typeof wanted !== 'boolean') continue;
      el.classList.toggle(CHROME_HIDDEN_CLASS[key], !wanted);
    }
  }
  return getChromeVisibility();
}
