/**
 * 대화 조립 — 설정·키(AiContext), 문서(PluginHost), 엔진(runAgent)을 잇는다.
 *
 * 한 번에 한 질문만 처리한다. 문서가 바뀌면 진행 중인 질문을 끊고 대화를 새로 시작한다 —
 * 이전 문서의 내용이 새 문서의 대화에 섞이지 않게 하기 위해서다.
 */
import type { PluginHost } from '@/plugin/types';
import type { AiContext } from '../ai-context.ts';
import type { AiProvider, AiToolCall, AiToolResult } from '../provider/types.ts';
import { AiProviderError } from '../provider/types.ts';
import { READ_TOOL_DEFS, executeReadTool, type ToolHost } from '../tools/read-tools.ts';
import { createBoundaryId, wrapDocumentContent } from '../tools/untrusted.ts';
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

/** 읽기 다음에 쓰기 — 도구 순서를 고정한다(요청 접두가 같아야 캐시가 유지된다). */
export const AI_TOOL_DEFS = [...READ_TOOL_DEFS, ...WRITE_TOOL_DEFS];

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

  constructor(ctx: AiContext, host: PluginHost, providerFactory: ProviderFactory = defaultProviderFactory) {
    this.ctx = ctx;
    this.host = host;
    this.providerFactory = providerFactory;
    host.onDocumentSwap(() => this.reset());
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
        executeTool: (call, boundaryId, signal) => (isWriteTool(call.name)
          ? this.executeWrite(call, boundaryId, signal)
          : executeReadTool(toolHost, call, boundaryId)),
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
  private async executeWrite(call: AiToolCall, boundaryId: string, signal: AbortSignal): Promise<AiToolResult> {
    const fail = (content: string): AiToolResult => ({ id: call.id, content, isError: true });
    const writeHost: WriteHost = this.host;
    let proposal: WriteProposal;
    try {
      proposal = prepareWrite(writeHost, call);
    } catch (error) {
      if (error instanceof WriteRefused) return fail(error.message);
      if (isDocumentReleased(error)) return fail('No document is open.');
      console.error('[AI] 변경 제안 실패', call.name, error);
      return fail('Failed to prepare the change.');
    }
    if (!this.presenter) return fail('Editing is not available here.');

    const card = this.presenter.present(proposal, { autoApply: this.ctx.settings.autoApply, signal });
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
      proposal.apply();
    } catch (error) {
      const message = error instanceof WriteRefused ? error.message
        : isDocumentReleased(error) ? 'Editing is not allowed right now (the document may be closed or in a mode that blocks editing).'
          : 'Failed to apply the change.';
      if (!(error instanceof WriteRefused)) console.error('[AI] 변경 적용 실패', call.name, error);
      card.settle('failed', message);
      return fail(message);
    }
    card.settle('applied');
    return {
      id: call.id,
      content: wrapDocumentContent({ applied: true, location: proposal.location, newText: proposal.inserted }, boundaryId),
      isError: false,
    };
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
