/**
 * 대화 조립 — 설정·키(AiContext), 문서(PluginHost), 엔진(runAgent)을 잇는다.
 *
 * 한 번에 한 질문만 처리한다. 문서가 바뀌면 진행 중인 질문을 끊고 대화를 새로 시작한다 —
 * 이전 문서의 내용이 새 문서의 대화에 섞이지 않게 하기 위해서다.
 */
import type { PluginHost } from '@/plugin/types';
import type { AiContext } from '../ai-context.ts';
import type { AiProvider } from '../provider/types.ts';
import { AiProviderError } from '../provider/types.ts';
import { READ_TOOL_DEFS, executeReadTool, type ToolHost } from '../tools/read-tools.ts';
import { createBoundaryId } from '../tools/untrusted.ts';
import { runAgent, type AgentCallbacks, type AgentRunResult } from './agent-loop.ts';
import { ChatSession } from './chat-session.ts';
import { AI_SYSTEM_PROMPT } from './system-prompt.ts';

export type ProviderFactory = (apiKey: string) => Promise<AiProvider>;

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

  constructor(ctx: AiContext, host: PluginHost, providerFactory: ProviderFactory = defaultProviderFactory) {
    this.ctx = ctx;
    this.host = host;
    this.providerFactory = providerFactory;
    host.onDocumentSwap(() => this.reset());
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
        tools: READ_TOOL_DEFS,
        system: AI_SYSTEM_PROMPT,
        model: settings.model,
        effort: settings.effort,
        executeTool: (call, boundaryId) => executeReadTool(toolHost, call, boundaryId),
        createBoundaryId: () => createBoundaryId(),
      }, text, abort.signal, callbacks);
    } finally {
      if (this.running === abort) this.running = null;
    }
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
