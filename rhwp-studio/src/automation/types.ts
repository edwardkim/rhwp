/**
 * 자동화 표면의 타입 계약 — 외부 JavaScript 가 studio 의 **커맨드와 메뉴**를 다루는 자리.
 *
 * 경계: 여기는 문서를 바꾸지 않는다. 문서 조작은 플러그인(`@/plugin/types`)이 맡는다.
 * 자동화는 메뉴·툴바·키보드가 이미 지나가는 `CommandDispatcher` 를 **그대로** 태운다 —
 * 우회로를 하나 더 내면 양식 모드 차단·`canExecute` 게이트가 표면마다 갈린다.
 *
 * 계획: `mydocs/plans/rhwp_studio_hwpctrl_plugin_impl.md` §2.1
 */
import type { CommandDef, EditorContext } from '@/command/types';

export type { CommandFailure, CommandResult } from '@/command/types';

/** 확장 커맨드 ID 접두사. 이 접두사 없이는 등록할 수 없다. */
export const EXT_COMMAND_PREFIX = 'ext:';

/** `listCommands()` 가 돌려주는 커맨드 한 건. 레지스트리 + 현재 컨텍스트의 활성 여부다. */
export interface CommandInfo {
  id: string;
  label: string;
  shortcutLabel?: string;
  icon?: string;
  /** 지금 이 컨텍스트에서 실행 가능한가 (`CommandDispatcher.isEnabled` 와 같은 판정) */
  enabled: boolean;
  /** 실행하면 다이얼로그를 여는가 */
  opensDialog?: boolean;
}

/**
 * 메뉴 모델 한 갈래.
 *
 * **레지스트리가 아니라 DOM 에서 파생한다.** 메뉴 구조의 선언은 `index.html` 에 있고
 * (`.menu-item` 8 / `.md-item` 197 / `.md-sub` 32), 레이블·활성은 레지스트리에서 온다.
 * 그래서 `getMenuModel()` 은 `listCommands()` 의 부분집합이 아니다 — 레지스트리에는 있으나
 * 메뉴 마크업에 없는 커맨드가 41개 있다(컨텍스트 메뉴·단축키 전용).
 */
export interface MenuNode {
  /** `data-menu` 값 — file, edit, view, insert, format, page, table, tool */
  menuId: string;
  label: string;
  items: MenuItemNode[];
}

export interface MenuItemNode {
  /** `data-cmd` 값. 없으면 구분선이나 비커맨드 항목이다. */
  commandId?: string;
  label: string;
  enabled: boolean;
  /** `.md-sub` 하위 항목 */
  submenu?: MenuItemNode[];
}

/** 메뉴에 항목을 심을 자리. `commandId` 는 `ext:` 접두사여야 한다. */
export interface MenuItemSpec {
  menuId: string;
  commandId: string;
  position?: 'top' | 'bottom';
}

/**
 * 확장 커맨드 정의.
 *
 * `CommandDef` 와 달리 `id` 가 `ext:` 로 시작해야 하고, 이 제약은 등록 시점에 검사한다.
 * 함수(`execute`)를 담으므로 **iframe 안에서만** 등록할 수 있다 — 부모 페이지는 구조화 복제
 * 가능한 값만 보낼 수 있어 RPC 표면에 등록 메서드가 없다(명세서 §3.3).
 */
export type ExtCommandDef = CommandDef & { readonly id: `ext:${string}` };

/** 자동화 호스트의 공개 표면. `window.rhwpStudio.automation` 과 RPC 가 같은 것을 본다. */
export interface StudioAutomation {
  /** 레지스트리 전체 — 메뉴에 없는 커맨드도 포함한다. */
  listCommands(): CommandInfo[];
  /** 실제 메뉴 구조 — DOM 파생. */
  getMenuModel(): MenuNode[];
  isEnabled(id: string): boolean;
  /**
   * `CommandDispatcher.dispatchWithResult` 위임. 게이트는 UI 와 동일하다.
   *
   * 단 하나 더 걸린다 — `opensDialog` 커맨드는 기본 거절(`needs-dialog`)이다. 자동화가 대화상자를
   * 열면 사람이 누를 때까지 응답이 멈추기 때문이다. `{ allowDialog: true }` 로 풀 수 있다.
   */
  execute(
    id: string,
    params?: Record<string, unknown>,
    options?: { allowDialog?: boolean },
  ): CommandResultLike;
  getContext(): EditorContext;

  registerCommand(def: ExtCommandDef): void;
  unregisterCommand(id: string): void;
  addMenuItem(spec: MenuItemSpec): void;
  removeMenuItem(commandId: string): void;
}

/**
 * `execute` 반환 타입의 지역 별칭.
 *
 * `CommandResult` 를 그대로 쓰면 이 파일의 `export type` 재노출과 이름이 겹쳐 읽기 어렵다.
 * 값은 완전히 같다.
 */
export type CommandResultLike = import('@/command/types').CommandResult;

/** 자동화가 던지는 오류 코드. RPC 는 이 값을 그대로 실어 보낸다. */
export type AutomationErrorCode =
  | 'EXT_PREFIX_REQUIRED'
  | 'UNKNOWN_MENU'
  | 'UNKNOWN_COMMAND';

/** chrome(메뉴·툴바·상태표시줄) 표시 상태. 표시만 바꾸며 커맨드는 계속 살아 있다. */
export interface ChromeVisibility {
  menu: boolean;
  toolbar: boolean;
  statusbar: boolean;
}

/** `#studio-root` 에 얹는 클래스 — 실제 숨김은 CSS 가 한다. */
export const CHROME_HIDDEN_CLASS: Readonly<Record<keyof ChromeVisibility, string>> = {
  menu: 'rhwp-chrome-no-menu',
  toolbar: 'rhwp-chrome-no-toolbar',
  statusbar: 'rhwp-chrome-no-status',
};
