/**
 * 메뉴 모델의 DOM 파생과 `ext:` 항목 삽입.
 *
 * 메뉴 **구조의 선언은 `index.html`** 에 있다(`.menu-item` 최상위, `.md-sub` 서브메뉴,
 * `.md-item[data-cmd]` 항목). `MenuBar`(`@/ui/menu-bar`)는 그 마크업에 이벤트를 걸고 레이블·활성을
 * 레지스트리에서 채우는 **컨트롤러**다. 따라서 자동화의 메뉴 질의도 같은 자리에서 읽는다 —
 * 별도 모델을 세워 두면 마크업과 두 벌이 되어 어긋난다.
 *
 * 마크업을 선언 데이터로 옮기는 일은 별개 작업이다(계획서 §3.2, P6 선택).
 */
import type { MenuItemNode, MenuNode } from './types';

const MENU_ITEM = '.menu-item';
const MENU_TITLE = '.menu-title';
/** 드롭다운 본문. 죽어 있던 `extension-api.ts` 는 `.md-body` 를 찾았는데 그런 요소는 없다. */
const DROPDOWN = '.menu-dropdown';
const ITEM = '.md-item';
const SUB = '.md-sub';
/** 서브메뉴의 실제 항목 컨테이너 — `.md-sub` 안에 따로 있다. */
const SUB_PANEL = '.md-sub-panel';
const LABEL = '.md-label';

/** 항목 하나의 표시 이름. `.md-label` 이 없으면 요소 텍스트로 떨어진다. */
function labelOf(el: Element): string {
  const label = el.querySelector(LABEL);
  return (label?.textContent ?? el.textContent ?? '').trim();
}

function readItems(container: Element, isEnabled: (id: string) => boolean): MenuItemNode[] {
  const nodes: MenuItemNode[] = [];
  for (const el of Array.from(container.children)) {
    if (el.classList.contains(SUB.slice(1))) {
      const body = el.querySelector(SUB_PANEL);
      nodes.push({
        label: labelOf(el),
        // 서브메뉴 컨테이너 자체는 커맨드가 아니다. 활성은 하위에 살아 있는 항목이 있는지로 본다.
        enabled: !el.classList.contains('disabled'),
        submenu: body ? readItems(body, isEnabled) : [],
      });
      continue;
    }
    if (!el.classList.contains(ITEM.slice(1))) continue;

    const commandId = (el as HTMLElement).dataset.cmd;
    nodes.push({
      commandId,
      label: labelOf(el),
      // 마크업의 `disabled` 클래스는 드롭다운이 열릴 때만 갱신된다. 커맨드가 있으면 레지스트리
      // 판정을 그대로 쓰고, 없으면(구분선·안내 문구) 마크업을 따른다.
      enabled: commandId ? isEnabled(commandId) : !el.classList.contains('disabled'),
    });
  }
  return nodes;
}

/** 메뉴바 컨테이너에서 현재 메뉴 구조를 읽는다. */
export function readMenuModel(
  container: HTMLElement,
  isEnabled: (id: string) => boolean,
): MenuNode[] {
  const menus: MenuNode[] = [];
  for (const menu of Array.from(container.querySelectorAll(MENU_ITEM))) {
    const menuId = (menu as HTMLElement).dataset.menu ?? '';
    const body = menu.querySelector(DROPDOWN);
    menus.push({
      menuId,
      label: (menu.querySelector(MENU_TITLE)?.textContent ?? menuId).trim(),
      items: body ? readItems(body, isEnabled) : [],
    });
  }
  return menus;
}

/** 지정 메뉴의 드롭다운 본문. 없으면 `null`. */
export function findMenuBody(container: HTMLElement, menuId: string): HTMLElement | null {
  return container.querySelector<HTMLElement>(`${MENU_ITEM}[data-menu="${menuId}"] ${DROPDOWN}`);
}

/**
 * 메뉴에 커맨드 항목을 심는다.
 *
 * 만들어진 요소는 기존 항목과 **같은 마크업 계약**(`.md-item[data-cmd]` + `.md-label`)을 따른다.
 * 그래야 `MenuBar` 의 클릭 위임·활성 갱신·단축키 매칭이 그대로 걸린다.
 */
export function insertMenuItem(
  body: HTMLElement,
  commandId: string,
  label: string,
  shortcutLabel: string | undefined,
  position: 'top' | 'bottom' = 'bottom',
): HTMLElement {
  const item = document.createElement('div');
  item.className = 'md-item';
  item.dataset.cmd = commandId;

  const labelEl = document.createElement('span');
  labelEl.className = 'md-label';
  labelEl.textContent = label;
  item.appendChild(labelEl);

  if (shortcutLabel) {
    const shortcut = document.createElement('span');
    shortcut.className = 'md-shortcut';
    shortcut.textContent = shortcutLabel;
    item.appendChild(shortcut);
  }

  if (position === 'top') body.prepend(item);
  else body.appendChild(item);
  return item;
}

/** 심어 둔 항목을 걷어낸다. 없으면 아무 일도 하지 않는다. */
export function removeMenuItemEl(container: HTMLElement, commandId: string): void {
  container.querySelectorAll(`${ITEM}[data-cmd="${commandId}"]`).forEach((el) => el.remove());
}

/** 마크업에 실제로 있는 `data-cmd` 전부 — 레지스트리 대조(드리프트 가드)에 쓴다. */
export function collectMarkupCommandIds(root: ParentNode = document): string[] {
  const ids = new Set<string>();
  root.querySelectorAll('[data-cmd]').forEach((el) => {
    const id = (el as HTMLElement).dataset.cmd;
    if (id) ids.add(id);
  });
  return Array.from(ids).sort();
}
