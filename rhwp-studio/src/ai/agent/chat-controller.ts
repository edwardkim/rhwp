/**
 * 대화 조립 — 설정·키(AiContext), 문서(PluginHost), 엔진(runAgent)을 잇는다.
 *
 * 한 번에 한 질문만 처리한다. 문서가 바뀌면 진행 중인 질문을 끊고 대화를 새로 시작한다 —
 * 이전 문서의 내용이 새 문서의 대화에 섞이지 않게 하기 위해서다.
 */
import type { PluginHost } from '@/plugin/types';
import { StudioCommandError } from '../../core/command-error.ts';
import type { AiContext } from '../ai-context.ts';
import type { AiProvider, AiToolCall, AiToolResult } from '../provider/types.ts';
import { AiProviderError } from '../provider/types.ts';
import { READ_TOOL_DEFS, executeReadTool, type ToolHost } from '../tools/read-tools.ts';
import { createBoundaryId, wrapDocumentContent } from '../tools/untrusted.ts';
import {
  COMMAND_TOOL_DEFS,
  isCommandTool,
  listAllowedCommands,
  prepareCommand,
  selectText,
  type CommandHost,
} from '../tools/command-tools.ts';
import {
  WRITE_TOOL_DEFS,
  WriteRefused,
  isWriteTool,
  prepareWrite,
  type WriteHost,
  type WriteProposal,
} from '../tools/write-tools.ts';
import { runAgent, type AgentCallbacks, type AgentRunResult } from './agent-loop.ts';
import { ChatSession } from './chat-session.ts';
import { AI_SYSTEM_PROMPT } from './system-prompt.ts';

export type ProviderFactory = (apiKey: string) => Promise<AiProvider>;

/** 읽기 → 쓰기 → 명령 통로 순서를 고정한다(요청 접두가 같아야 캐시가 유지된다). */
export const AI_TOOL_DEFS = [...READ_TOOL_DEFS, ...WRITE_TOOL_DEFS, ...COMMAND_TOOL_DEFS];

export type ProposalOutcome = 'applied' | 'rejected' | 'stopped' | 'failed';

/** 화면이 제안을 보이고 사용자의 선택을 받는 자리. */
export interface ProposalPresenter {
  /**
   * 제안 카드를 띄운다. `autoApply` 면 묻지 않고 곧바로 승인으로 끝난다.
   * `decision` 은 승인이면 true. `signal` 이 끊기면 false 로 끝나야 한다.
   */
  present(proposal: WriteProposal, options: { autoApply: boolean; signal: AbortSignal }): {
    decision: Promise<boolean>;
    settle(outcome: ProposalOutcome, message?: string): void;
  };
}

/** 문서 해제·편집 모드 차단 — 플러그인 호스트가 `DOCUMENT_RELEASED` 로 던진다. */
function isDocumentReleased(error: unknown): boolean {
  return (error as { code?: unknown } | null)?.code === 'DOCUMENT_RELEASED';
}

/** SDK 는 처음 질문할 때 불러온다. */
const defaultProviderFactory: ProviderFactory = async (apiKey) => {
  const { ClaudeProvider } = await import('../provider/claude.ts');
  return new ClaudeProvider(apiKey);
};

export class ChatController {
  /**
   * 대화를 새로 시작할 때 이력을 비우지 않고 **새 인스턴스로 바꾼다**. 끊긴 질문이 뒤늦게
   * 덧붙이는 칸은 옛 이력으로 가므로 새 대화를 어지럽히지 않는다.
   */
  private current = new ChatSession();
  private readonly ctx: AiContext;
  private readonly host: PluginHost;
  private readonly providerFactory: ProviderFactory;
  private running: AbortController | null = null;
  private resetListeners = new Set<() => void>();
  private presenter: ProposalPresenter | null = null;
  /** AI 가 스스로 새 문서를 만드는 중인가 — 그 교체로는 대화를 끊지 않는다. */
  private ownSwap = false;
  /** 사용자가 카드에서 「이번 응답 모두 적용」을 골랐는가. 질문 하나가 끝나면 풀린다. */
  private applyRestOfRun = false;

  constructor(ctx: AiContext, host: PluginHost, providerFactory: ProviderFactory = defaultProviderFactory) {
    this.ctx = ctx;
    this.host = host;
    this.providerFactory = providerFactory;
    host.onDocumentSwap(() => {
      if (!this.ownSwap) this.reset();
    });
  }

  /** 쓰기 제안을 보일 화면을 붙인다. 없으면 쓰기 도구는 거절된다. */
  setPresenter(presenter: ProposalPresenter | null): void {
    this.presenter = presenter;
  }

  get session(): ChatSession {
    return this.current;
  }

  get busy(): boolean {
    return this.running !== null;
  }

  /** 질문 하나를 끝까지 처리한다. 진행 중이면 거절한다. */
  async ask(text: string, callbacks: AgentCallbacks = {}): Promise<AgentRunResult> {
    if (this.running) throw new Error('이미 답변을 만드는 중입니다.');
    const settings = this.ctx.settings;
    const apiKey = this.ctx.keyStore.getKey(settings.provider);
    if (!apiKey) throw new AiProviderError({ reason: 'auth' });

    const abort = new AbortController();
    this.running = abort;
    this.applyRestOfRun = false;
    try {
      const provider = await this.providerFactory(apiKey);
      const toolHost: ToolHost = this.host;
      return await runAgent({
        provider,
        session: this.session,
        tools: AI_TOOL_DEFS,
        system: AI_SYSTEM_PROMPT,
        model: settings.model,
        effort: settings.effort,
        executeTool: (call, boundaryId, signal) => {
          if (isWriteTool(call.name)) return this.executeWrite(call, boundaryId, signal, (host) => prepareWrite(host, call));
          if (isCommandTool(call.name)) return this.executeCommandTool(call, boundaryId, signal);
          return executeReadTool(toolHost, call, boundaryId);
        },
        createBoundaryId: () => createBoundaryId(),
      }, text, abort.signal, callbacks);
    } finally {
      if (this.running === abort) this.running = null;
    }
  }

  /**
   * 쓰기 도구 하나 — 제안을 만들고, 사용자가 승인하면 트랜잭션 한 번으로 적용한다.
   * 결과 문구는 모델이 읽는다. 문서에서 온 글은 경계로 감싼다.
   */
  /** 명령 통로: 목록·선택은 바로 하고, 실행은 쓰기와 같은 승인 흐름을 탄다. */
  private executeCommandTool(call: AiToolCall, boundaryId: string, signal: AbortSignal): AiToolResult | Promise<AiToolResult> {
    const commandHost: CommandHost = this.host;
    if (call.name === 'run_command') return this.executeWrite(call, boundaryId, signal, () => prepareCommand(commandHost, call));
    try {
      const value = call.name === 'list_commands' ? listAllowedCommands(commandHost) : selectText(commandHost, call);
      return { id: call.id, content: wrapDocumentContent(value, boundaryId), isError: false };
    } catch (error) {
      if (error instanceof WriteRefused) return { id: call.id, content: error.message, isError: true };
      if (isDocumentReleased(error)) return { id: call.id, content: 'No document is open.', isError: true };
      console.error('[AI] 명령 도구 실패', call.name, error);
      return { id: call.id, content: 'The command tool failed.', isError: true };
    }
  }

  private async executeWrite(
    call: AiToolCall,
    boundaryId: string,
    signal: AbortSignal,
    prepare: (host: WriteHost) => WriteProposal,
  ): Promise<AiToolResult> {
    const fail = (content: string): AiToolResult => ({ id: call.id, content, isError: true });
    const writeHost: WriteHost = {
      read: (fn) => this.host.read(fn),
      transaction: (label, fn) => this.host.transaction(label, fn),
      createBlankDocument: async () => {
        this.ownSwap = true;
        try {
          return await this.host.createBlankDocument({ confirmUnsaved: true });
        } finally {
          this.ownSwap = false;
        }
      },
    };
    let proposal: WriteProposal;
    try {
      proposal = prepare(writeHost);
    } catch (error) {
      if (error instanceof WriteRefused) return fail(error.message);
      if (isDocumentReleased(error)) return fail('No document is open.');
      console.error('[AI] 변경 제안 실패', call.name, error);
      return fail('Failed to prepare the change.');
    }
    if (!this.presenter) return fail('Editing is not available here.');

    const card = this.presenter.present(proposal, {
      autoApply: this.ctx.settings.autoApply || this.applyRestOfRun,
      signal,
    });
    const approved = await card.decision;
    if (!approved) {
      if (signal.aborted) {
        card.settle('stopped');
        return { id: call.id, content: 'Not applied: the user stopped the response.', isError: false };
      }
      card.settle('rejected');
      return {
        id: call.id,
        content: 'The user rejected this change. Do not propose the same change again unless the user asks.',
        isError: false,
      };
    }
    try {
      await proposal.apply();
    } catch (error) {
      // 명령 경계는 롤백·화면 복구를 비동기로 확인한다. 보고가 끝나기 전에
      // 다음 도구를 실행하면 복구 중인 세션이 같은 실패로 후속 편집까지 거절한다.
      if (error instanceof StudioCommandError) await error.report;
      // 편집 엔진이 지원하지 않는 경우(예: 개체가 든 문단의 하이퍼링크)는 엔진 사유를 그대로 전한다 —
      // 모델이 다른 방법을 고르고, 사용자는 카드에서 이유를 본다. 엔진 사유에는 키가 없다.
      const cause = error instanceof StudioCommandError ? error.cause : error;
      const engineReason = cause instanceof Error ? cause.message : typeof cause === 'string' ? cause : '';
      const message = error instanceof WriteRefused ? error.message
        : isDocumentReleased(error) ? 'Editing is not allowed right now (the document may be closed or in a mode that blocks editing).'
          : engineReason ? `Failed to apply the change: ${engineReason.slice(0, 300)}` : 'Failed to apply the change.';
      if (!(error instanceof WriteRefused)) console.error('[AI] 변경 적용 실패', call.name, error);
      card.settle('failed', message);
      return fail(message);
    }
    card.settle('applied');
    return {
      id: call.id,
      content: wrapDocumentContent({
        applied: true, location: proposal.location, newText: proposal.inserted,
        ...(proposal.note ? { note: proposal.note } : {}),
      }, boundaryId),
      isError: false,
    };
  }

  /** 이번 응답의 남은 변경은 묻지 않고 적용한다(카드의 「이번 응답 모두 적용」). */
  approveRestOfRun(): void {
    this.applyRestOfRun = true;
  }

  /** 진행 중인 질문을 끊는다. 이력은 엔진이 유효한 상태로 마무리한다. */
  stop(): void {
    this.running?.abort();
  }

  /** 대화를 새로 시작한다. 진행 중이면 먼저 끊는다. */
  reset(): void {
    this.stop();
    this.current = new ChatSession();
    for (const listener of [...this.resetListeners]) listener();
  }

  onReset(listener: () => void): () => void {
    this.resetListeners.add(listener);
    return () => { this.resetListeners.delete(listener); };
  }
}
