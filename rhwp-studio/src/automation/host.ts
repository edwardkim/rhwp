/**
 * 자동화 호스트 — 외부 JavaScript 가 커맨드와 메뉴를 다루는 단일 표면.
 *
 * 설계 원칙 하나: **UI 가 못 하는 일을 자동화가 할 수 있게 만들지 않는다.** 실행은
 * `CommandDispatcher.dispatchWithResult` 를 그대로 태우므로 양식 모드 차단·`canExecute` 게이트가
 * 메뉴·툴바·키보드와 똑같이 걸린다. 우회로를 하나 더 내면 그것이 곧 버그의 출처가 된다.
 *
 * 계획: `mydocs/plans/rhwp_studio_hwpctrl_plugin_impl.md` §2.1, §7(P1)
 */
import type { CommandDispatcher } from '@/command/dispatcher';
import type { CommandRegistry } from '@/command/registry';
import type { CommandResult, EditorContext } from '@/command/types';
import { formatShortcutLabel } from '@/engine/navigation-keymap';
import {
  findMenuBody,
  insertMenuItem,
  readMenuModel,
  removeMenuItemEl,
} from './menu-dom';
import {
  EXT_COMMAND_PREFIX,
  type CommandInfo,
  type ExtCommandDef,
  type MenuItemSpec,
  type MenuNode,
  type StudioAutomation,
} from './types';

export interface AutomationHostOptions {
  registry: CommandRegistry;
  dispatcher: CommandDispatcher;
  getContext: () => EditorContext;
  /** 메뉴바 컨테이너. DOM 이 늦게 서므로 함수로 받는다. */
  getMenuContainer: () => HTMLElement | null;
}

/** `ext:` 접두사 위반은 등록 시점에 막는다. 네임스페이스가 섞이면 회수할 수 없다. */
function assertExtId(id: string): void {
  if (!id.startsWith(EXT_COMMAND_PREFIX)) {
    const error = new Error(`확장 커맨드 ID 는 "${EXT_COMMAND_PREFIX}" 접두사가 필요합니다: ${id}`);
    (error as Error & { code?: string }).code = 'EXT_PREFIX_REQUIRED';
    throw error;
  }
}

export class AutomationHost implements StudioAutomation {
  /** 이 호스트가 심은 메뉴 항목. 회수는 원장으로 한다 — DOM 을 훑어 지우지 않는다. */
  private readonly ownedMenuItems = new Set<string>();
  /** 이 호스트가 등록한 확장 커맨드. */
  private readonly ownedCommands = new Set<string>();

  constructor(private readonly options: AutomationHostOptions) {}

  listCommands(): CommandInfo[] {
    const { registry, dispatcher } = this.options;
    return registry.getAllIds().sort().map((id) => {
      const def = registry.get(id)!;
      return {
        id,
        label: def.label,
        shortcutLabel: def.shortcutLabel,
        icon: def.icon,
        enabled: dispatcher.isEnabled(id),
        opensDialog: def.opensDialog,
      };
    });
  }

  getMenuModel(): MenuNode[] {
    const container = this.options.getMenuContainer();
    if (!container) return [];
    return readMenuModel(container, (id) => this.options.dispatcher.isEnabled(id));
  }

  isEnabled(id: string): boolean {
    return this.options.dispatcher.isEnabled(id);
  }

  /**
   * 커맨드 실행.
   *
   * **다이얼로그를 여는 커맨드는 기본적으로 거절한다.** 자동화가 그것을 실행하면 사람이 누를
   * 때까지 응답이 멈춘다 — 호출자는 대기하는지 실패했는지 구분할 수 없다. 정말 띄우려면
   * `allowDialog` 를 명시한다(사용자가 앞에 있는 대화형 통합용).
   */
  execute(
    id: string,
    params?: Record<string, unknown>,
    options: { allowDialog?: boolean } = {},
  ): CommandResult {
    if (!options.allowDialog && this.options.registry.get(id)?.opensDialog) {
      return {
        ok: false,
        reason: 'needs-dialog',
        message: `대화상자를 여는 커맨드입니다. 띄우려면 allowDialog 를 지정하세요: ${id}`,
      };
    }
    return this.options.dispatcher.dispatchWithResult(id, params);
  }

  getContext(): EditorContext {
    return this.options.getContext();
  }

  registerCommand(def: ExtCommandDef): void {
    assertExtId(def.id);
    this.options.registry.register(def);
    this.ownedCommands.add(def.id);
  }

  unregisterCommand(id: string): void {
    // 내장 커맨드를 지우는 경로는 만들지 않는다 — studio 가 자기 기능을 잃는다.
    if (!id.startsWith(EXT_COMMAND_PREFIX)) return;
    this.removeMenuItem(id);
    this.options.registry.unregister(id);
    this.ownedCommands.delete(id);
  }

  addMenuItem(spec: MenuItemSpec): void {
    assertExtId(spec.commandId);

    const container = this.options.getMenuContainer();
    const body = container ? findMenuBody(container, spec.menuId) : null;
    if (!body) {
      const error = new Error(`메뉴를 찾을 수 없습니다: ${spec.menuId}`);
      (error as Error & { code?: string }).code = 'UNKNOWN_MENU';
      throw error;
    }

    const def = this.options.registry.get(spec.commandId);
    if (!def) {
      const error = new Error(`미등록 커맨드: ${spec.commandId}`);
      (error as Error & { code?: string }).code = 'UNKNOWN_COMMAND';
      throw error;
    }

    // 같은 커맨드를 두 번 심으면 항목이 겹친다. 먼저 걷어내고 새로 심는다.
    removeMenuItemEl(body, spec.commandId);
    insertMenuItem(
      body,
      spec.commandId,
      def.label,
      def.shortcutLabel ? formatShortcutLabel(def.shortcutLabel) : undefined,
      spec.position,
    );
    this.ownedMenuItems.add(spec.commandId);
  }

  removeMenuItem(commandId: string): void {
    const container = this.options.getMenuContainer();
    if (container) removeMenuItemEl(container, commandId);
    this.ownedMenuItems.delete(commandId);
  }

  /**
   * 이 호스트가 심은 것 전부 철거.
   *
   * 플러그인 unload(§4.4)와 브리지 destroy 가 부른다. 등록자가 정리 함수를 성실히 부를 것이라
   * 기대하지 않기 위한 자리다.
   */
  disposeOwned(): void {
    for (const id of Array.from(this.ownedMenuItems)) this.removeMenuItem(id);
    for (const id of Array.from(this.ownedCommands)) this.unregisterCommand(id);
  }
}
