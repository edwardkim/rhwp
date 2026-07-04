import type { EventBus } from '@/core/event-bus';
import type { CommandRegistry } from './registry';
import type { CommandServices, EditorContext } from './types';

const FORM_MODE_BLOCKED_IDS = new Set([
  'edit:cut',
  'edit:paste',
  'edit:delete',
  'field:edit',
  'field:remove',
]);

const FORM_MODE_BLOCKED_PREFIXES = [
  'format:',
  'insert:',
  'table:',
  'page:',
];

function isBlockedInFormMode(commandId: string, ctx: EditorContext): boolean {
  if (!ctx.isFormMode) return false;
  if (FORM_MODE_BLOCKED_IDS.has(commandId)) return true;
  return FORM_MODE_BLOCKED_PREFIXES.some(prefix => commandId.startsWith(prefix));
}

/** 통합 커맨드 디스패처: 메뉴/툴바/키보드 모든 입력의 단일 실행 경로 */
export class CommandDispatcher {
  constructor(
    private registry: CommandRegistry,
    private services: CommandServices,
    private eventBus: EventBus,
  ) {}

  /**
   * 커맨드 실행.
   * @returns true: 실행됨, false: 미등록 또는 비활성
   */
  dispatch(commandId: string, params?: Record<string, unknown>): boolean {
    const def = this.registry.get(commandId);
    if (!def) {
      console.warn(`[CommandDispatcher] 미등록 커맨드: ${commandId}`);
      return false;
    }

    const ctx = this.services.getContext();
    if (isBlockedInFormMode(commandId, ctx)) {
      return false;
    }
    if (def.canExecute && !def.canExecute(ctx)) {
      // canExecute 실패 — 비활성 상태
      return false;
    }

    try {
      def.execute(this.services, params);
      // 커맨드 실행 후 UI 상태 갱신 알림
      this.eventBus.emit('command-state-changed');
      return true;
    } catch (err) {
      console.error(`[CommandDispatcher] 커맨드 실행 실패: ${commandId}`, err);
      return false;
    }
  }

  /**
   * 커맨드가 현재 활성(실행 가능)인지 확인.
   * 메뉴/툴바의 enabled/disabled 상태 갱신에 사용.
   */
  isEnabled(commandId: string): boolean {
    const def = this.registry.get(commandId);
    if (!def) return false;
    const ctx = this.services.getContext();
    if (isBlockedInFormMode(commandId, ctx)) return false;
    if (!def.canExecute) return true;
    return def.canExecute(ctx);
  }
}
