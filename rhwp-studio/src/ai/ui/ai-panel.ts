/**
 * AI 패널 본문.
 *
 * 준비 상태(키 없음 / 잠김 / 전송 동의 필요)에서는 안내와 단추 하나를, 준비되면 대화 화면을 보인다.
 * 모델 응답은 `message-render.ts` 로만 그린다 — 이 파일은 HTML 문자열을 다루지 않는다.
 */
import { t } from '@/i18n/index.ts';
import { showConfirm } from '@/ui/confirm-dialog';
import type { AiContext } from '../ai-context.ts';
import type { AgentRunResult } from '../agent/agent-loop.ts';
import type { ChatController, ProposalOutcome, ProposalPresenter } from '../agent/chat-controller.ts';
import type { WriteProposal } from '../tools/write-tools.ts';
import { modelsOf } from '../provider/models.ts';
import { AiProviderError, EMPTY_USAGE, addUsage, type AiToolCall, type AiUsage } from '../provider/types.ts';
import { AiSettingsDialog, failureMessage } from './ai-settings-dialog.ts';
import { AiUnlockDialog } from './ai-unlock-dialog.ts';
import { renderMarkdown } from './message-render.ts';
import './ai-panel.css';

export type AiPanelState = 'none' | 'locked' | 'consent' | 'ready';

/** 패널이 쓰는 호스트 표면 — 선택 여부와 커서 이동 알림만. */
export interface AiPanelHost {
  hasSelection(): boolean;
  onCursorChange(listener: () => void): () => void;
}

/** 빠른 동작. 선택 영역이 있을 때만 보인다. 프롬프트는 사용자 말로 보낸다. */
const QUICK_ACTIONS = ['summarize', 'explain', 'proofread'] as const;

function el<K extends keyof HTMLElementTagNameMap>(tag: K, className: string, text?: string): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

function button(className: string, label: string, onClick: () => void): HTMLButtonElement {
  const node = el('button', className, label);
  node.type = 'button';
  node.addEventListener('click', onClick);
  return node;
}

/** 서식 변경 요약 — 「굵게 · 16pt · 글자색 #C00000」 */
function formatSummary(format: Record<string, string | number | boolean>): string {
  const parts: string[] = [];
  for (const [key, value] of Object.entries(format)) {
    switch (key) {
      case 'bold': case 'italic': case 'underline': case 'strikethrough':
        parts.push(t(`ai.format.${key}.${value ? 'on' : 'off'}`));
        break;
      case 'fontSize': parts.push(t('ai.format.fontSize', { v: value as number })); break;
      case 'alignment': parts.push(t('ai.format.alignment', { v: t(`ai.format.align.${value as string}`) })); break;
      case 'rows': break;
      case 'columns': parts.push(t('ai.format.table', { rows: format.rows as number, columns: value as number })); break;
      case 'column': parts.push(t('ai.format.cell', { row: format.row as number, column: value as number })); break;
      case 'row': break;
      case 'list': parts.push(t(`ai.format.list.${value as string}`)); break;
      case 'tableAction': parts.push(t(`ai.format.tableAction.${value as string}`)); break;
      case 'span': parts.push(t('ai.format.span', { v: value as number })); break;
      case 'kind': parts.push(t(`ai.format.kind.${value as string}`)); break;
      case 'keepsObjects': parts.push(t('ai.format.keepsObjects')); break;
      case 'command': parts.push(t('ai.format.command', { v: String(value) })); break;
      case 'scope': parts.push(t(`ai.format.scope.${value as string}`, { n: format.cells as number })); break;
      case 'cells': break;
      case 'verticalAlign': parts.push(t(`ai.format.valign.${value as string}`)); break;
      case 'orientation': parts.push(t(`ai.format.orientation.${value as string}`)); break;
      case 'borderStyle': parts.push(t(`ai.format.border.${value as string}`)); break;
      case 'backgroundColor':
        parts.push(value === 'none' ? t('ai.format.backgroundNone') : t('ai.format.backgroundColor', { v: String(value) }));
        break;
      default: parts.push(t(`ai.format.${key}`, { v: String(value) }));
    }
  }
  return parts.join(' · ');
}

/** 응답 하나의 화면 — 회차마다 글 조각, 도구 호출마다 표시. */
class AssistantView {
  readonly root = el('div', 'ai-msg ai-msg-assistant');
  private textPart: { node: HTMLElement; source: string } | null = null;
  private toolsPart: HTMLElement | null = null;
  private readonly dirty = new Set<{ node: HTMLElement; source: string }>();
  private frame = 0;
  private readonly chips = new Map<string, HTMLElement>();

  constructor() {
    this.root.setAttribute('aria-busy', 'true');
    const pending = el('div', 'ai-msg-pending', t('ai.chat.thinking'));
    this.root.append(pending);
  }

  turnStart(): void {
    this.textPart = null;
    this.toolsPart = null;
  }

  appendText(delta: string): void {
    this.root.querySelector('.ai-msg-pending')?.remove();
    if (!this.textPart) {
      this.textPart = { node: el('div', 'ai-msg-body'), source: '' };
      this.root.append(this.textPart.node);
    }
    this.textPart.source += delta;
    this.dirty.add(this.textPart);
    // 조각마다 다시 그리지 않고 화면 갱신 주기에 한 번 그린다.
    if (!this.frame) this.frame = requestAnimationFrame(() => this.flush());
  }

  toolStart(call: AiToolCall): void {
    this.root.querySelector('.ai-msg-pending')?.remove();
    if (!this.toolsPart) {
      this.toolsPart = el('div', 'ai-msg-tools');
      this.root.append(this.toolsPart);
    }
    // 다음 글은 도구 뒤에 새 조각으로 이어진다.
    this.textPart = null;
    const chip = el('span', 'ai-tool-chip', t(`ai.tool.${call.name}`));
    chip.dataset.state = 'running';
    this.toolsPart.append(chip);
    this.chips.set(call.id, chip);
  }

  toolEnd(call: AiToolCall, isError: boolean): void {
    const chip = this.chips.get(call.id);
    if (chip) chip.dataset.state = isError ? 'error' : 'done';
  }

  /** 변경 제안 카드. 사용자가 고르면(또는 자동 적용이면) `decision` 이 끝난다. */
  proposal(proposal: WriteProposal, autoApply: boolean, signal: AbortSignal) {
    this.root.querySelector('.ai-msg-pending')?.remove();
    this.textPart = null;
    this.toolsPart = null;
    const card = el('div', 'ai-change-card');
    card.dataset.state = 'pending';
    const loc = proposal.location;
    const coords = { section: (loc.section ?? 0) + 1, paragraph: (loc.paragraph ?? 0) + 1 };
    // 새 문서와 머리말·꼬리말은 본문 위치가 없다(머리말 종류는 요약에 보인다).
    const where = ['create_document', 'set_header_footer', 'run_command', 'replace_all', 'page_setup', 'set_columns']
      .includes(proposal.tool)
      ? null
      : loc.field !== undefined ? t('ai.change.field', { name: loc.field })
        : loc.position === 'start' ? t('ai.change.atStart')
          : loc.position === 'end' ? t('ai.change.atEnd')
            : loc.position === 'after' ? t('ai.change.after', coords)
              : t(loc.inTable ? 'ai.change.locationTable' : 'ai.change.location', coords);
    const title = t(`ai.change.${proposal.tool}`);
    card.append(el('div', 'ai-change-title', where ? `${title} · ${where}` : title));
    const line = (className: string, label: string, mark: 'del' | 'ins', text: string) => {
      const row = el('div', `ai-change-line ${className}`);
      row.append(el('span', 'ai-change-label', label), document.createTextNode(proposal.prefix));
      const changed = el(mark, `ai-change-${mark}`, text);
      if (text.length === 0) changed.dataset.empty = 'true';
      row.append(changed, document.createTextNode(proposal.suffix));
      return row;
    };
    if (proposal.tool === 'create_document') {
      card.append(el('div', 'ai-change-line', t('ai.change.createNote')));
    } else if (proposal.tool === 'insert_paragraphs' || proposal.tool === 'insert_table') {
      card.append(line('ai-change-after', '+', 'ins', proposal.inserted));
    } else if (proposal.tool === 'insert_footnote') {
      card.append(line('ai-change-after', t('ai.change.footnoteLabel'), 'ins', proposal.inserted));
    } else if (proposal.tool === 'delete_paragraph') {
      card.append(line('ai-change-before', '−', 'del', proposal.removed));
    } else if (proposal.tool === 'page_setup' || proposal.tool === 'set_columns') {
      card.append(el('div', 'ai-change-line', t('ai.change.pageSetupNote')));
    } else if (proposal.tool === 'insert_equation') {
      card.append(line('ai-change-after', t('ai.change.equationLabel'), 'ins', proposal.inserted));
    } else if (['format_text', 'format_paragraph', 'set_list', 'apply_style', 'edit_table', 'run_command', 'format_cell',
      'set_table_border', 'set_cell_size', 'insert_hyperlink', 'add_bookmark'].includes(proposal.tool)) {
      const target = el('div', 'ai-change-line ai-change-target');
      target.append(document.createTextNode(proposal.prefix), el('mark', 'ai-change-mark', proposal.inserted),
        document.createTextNode(proposal.suffix));
      card.append(target);
    } else {
      card.append(
        line('ai-change-before', t('ai.change.before'), 'del', proposal.removed),
        line('ai-change-after', t('ai.change.afterLabel'), 'ins', proposal.inserted),
      );
    }
    if (proposal.format) card.append(el('div', 'ai-change-summary', formatSummary(proposal.format)));
    const actions = el('div', 'ai-change-actions');
    const status = el('div', 'ai-change-status');
    card.append(actions, status);
    this.root.append(card);

    let resolve!: (approved: boolean) => void;
    const decision = new Promise<boolean>((r) => { resolve = r; });
    const decide = (approved: boolean) => {
      actions.replaceChildren();
      signal.removeEventListener('abort', onAbort);
      resolve(approved);
    };
    const onAbort = () => decide(false);
    if (autoApply) {
      decide(true);
    } else {
      const apply = button('dialog-btn dialog-btn-primary ai-change-apply', t('ai.change.apply'), () => decide(true));
      const reject = button('dialog-btn ai-change-reject', t('ai.change.reject'), () => decide(false));
      actions.append(apply, reject);
      signal.addEventListener('abort', onAbort, { once: true });
      apply.focus();
    }
    return {
      decision,
      settle: (outcome: ProposalOutcome, message?: string) => {
        card.dataset.state = outcome;
        // 문서 교체는 undo 대상이 아니다 — 「Ctrl+Z 로 되돌릴 수 있음」을 보이지 않는다.
        const key = outcome === 'applied' && proposal.tool === 'create_document' ? 'ai.change.createdDocument' : `ai.change.${outcome}`;
        status.textContent = message ? `${t(key)} — ${message}` : t(key);
      },
    };
  }

  finish(note: string | null, kind: 'info' | 'error' = 'info'): void {
    this.flush();
    this.root.querySelector('.ai-msg-pending')?.remove();
    this.root.removeAttribute('aria-busy');
    if (note) {
      const node = el('div', 'ai-msg-note', note);
      node.dataset.kind = kind;
      this.root.append(node);
    }
  }

  private flush(): void {
    if (this.frame) cancelAnimationFrame(this.frame);
    this.frame = 0;
    for (const part of this.dirty) renderMarkdown(part.node, part.source);
    this.dirty.clear();
  }
}

export class AiPanel {
  private readonly ctx: AiContext;
  private readonly chat: ChatController;
  private readonly host: AiPanelHost;
  private root: HTMLElement | null = null;
  private status!: HTMLElement;
  private notice!: HTMLParagraphElement;
  private action!: HTMLButtonElement;
  private modelLine!: HTMLElement;
  private chatView!: HTMLElement;
  private log!: HTMLElement;
  private empty!: HTMLElement;
  private quick!: HTMLElement;
  private input!: HTMLTextAreaElement;
  private sendButton!: HTMLButtonElement;
  private usageLine!: HTMLElement;
  private offs: Array<() => void> = [];
  private visible = false;
  /** 이번 실행에서 동의 대화상자를 이미 띄웠는가 — 거절한 사람에게 열 때마다 묻지 않는다. */
  private consentAsked = false;
  private state: AiPanelState = 'none';
  private renderSeq = 0;
  private usage: AiUsage = EMPTY_USAGE;
  private selectionFrame = 0;

  constructor(ctx: AiContext, chat: ChatController, host: AiPanelHost) {
    this.ctx = ctx;
    this.chat = chat;
    this.host = host;
  }

  get currentState(): AiPanelState {
    return this.state;
  }

  mount(container: HTMLElement): void {
    const root = el('div', 'ai-panel');

    const head = el('div', 'ai-panel-head');
    this.modelLine = el('div', 'ai-panel-model');
    const newChat = button('ai-icon-btn ai-new-chat', t('ai.chat.newChat'), () => this.newConversation());
    const settings = button('ai-icon-btn ai-open-settings', t('ai.command.settings'), () => this.openSettings());
    head.append(this.modelLine, newChat, settings);

    this.status = el('div', 'ai-panel-status');
    this.notice = el('p', 'ai-panel-notice');
    this.action = button('dialog-btn ai-panel-action', '', () => { void this.runAction(); });
    this.status.append(this.notice, this.action);

    this.chatView = el('div', 'ai-chat');
    this.log = el('div', 'ai-chat-log');
    this.log.setAttribute('role', 'log');
    this.log.setAttribute('aria-live', 'polite');
    this.empty = el('p', 'ai-chat-empty', t('ai.chat.empty'));
    this.log.append(this.empty);

    this.quick = el('div', 'ai-chat-quick');
    this.quick.hidden = true;
    for (const id of QUICK_ACTIONS) {
      const quickButton = button('ai-quick-btn', t(`ai.quick.${id}`), () => { void this.send(t(`ai.quick.${id}.prompt`)); });
      quickButton.dataset.action = id;
      this.quick.append(quickButton);
    }

    const compose = el('div', 'ai-chat-compose');
    this.input = el('textarea', 'ai-chat-input');
    this.input.rows = 3;
    this.input.placeholder = t('ai.chat.placeholder');
    this.input.setAttribute('aria-label', t('ai.chat.placeholder'));
    this.input.addEventListener('keydown', (event) => {
      // 한글 조합 중의 Enter 는 글자 확정이다 — 보내지 않는다.
      if (event.key === 'Enter' && !event.shiftKey && !event.isComposing && event.keyCode !== 229) {
        event.preventDefault();
        void this.send(this.input.value);
      }
    });
    this.sendButton = button('dialog-btn dialog-btn-primary ai-send-btn', t('ai.chat.send'), () => {
      if (this.chat.busy) this.chat.stop();
      else void this.send(this.input.value);
    });
    compose.append(this.input, this.sendButton);
    this.usageLine = el('div', 'ai-chat-usage');

    this.chatView.append(this.log, this.quick, compose, this.usageLine);
    root.append(head, this.status, this.chatView);
    container.append(root);
    this.root = root;

    this.offs.push(this.ctx.onChange(() => { void this.render(); }));
    this.offs.push(this.chat.onReset(() => this.clearLog(t('ai.chat.documentChanged'))));
    this.offs.push(this.host.onCursorChange(() => this.scheduleSelectionCheck()));
    this.renderUsage();
    void this.render();
  }

  setVisible(open: boolean): void {
    this.visible = open;
    if (!this.root) return;
    this.root.dataset.open = open ? 'true' : 'false';
    if (open) {
      this.scheduleSelectionCheck();
      void this.render().then(() => {
        void this.maybeAskConsent();
        if (this.state === 'ready') this.input.focus();
      });
    }
  }

  openSettings(): void {
    new AiSettingsDialog(this.ctx).show();
  }

  dispose(): void {
    this.chat.stop();
    for (const off of this.offs) off();
    this.offs = [];
    if (this.selectionFrame) cancelAnimationFrame(this.selectionFrame);
    this.root?.remove();
    this.root = null;
  }

  private async render(): Promise<void> {
    if (!this.root) return;
    const seq = ++this.renderSeq;
    const status = await this.ctx.keyStatus();
    // 늦게 끝난 이전 렌더가 새 상태를 덮지 않게 한다.
    if (seq !== this.renderSeq || !this.root) return;

    const previous = this.state;
    this.state = status === 'none' ? 'none'
      : status === 'locked' ? 'locked'
        : this.ctx.hasConsent() ? 'ready' : 'consent';
    this.root.dataset.state = this.state;

    const settings = this.ctx.settings;
    const model = modelsOf(settings.provider).find((m) => m.id === settings.model)?.label ?? settings.model;
    this.modelLine.textContent = t('ai.panel.modelLine', { provider: t(`ai.provider.${settings.provider}`), model });

    const ready = this.state === 'ready';
    this.status.hidden = ready;
    this.chatView.hidden = !ready;
    // 키 등록·동의·잠금 해제로 막 준비됐으면 바로 물을 수 있게 입력란으로 옮긴다.
    // 대화상자가 닫히며 편집 영역에 돌려준 포커스보다 뒤에 오므로 이쪽이 남는다.
    if (ready && previous !== 'ready' && this.visible) this.input.focus();
    if (!ready) {
      const [notice, action] = ({
        none: ['ai.panel.setupNotice', 'ai.panel.openSettings'],
        locked: ['ai.panel.lockedNotice', 'ai.panel.unlock'],
        consent: ['ai.panel.consentNotice', 'ai.panel.giveConsent'],
      } as const)[this.state as Exclude<AiPanelState, 'ready'>];
      this.notice.textContent = t(notice);
      this.action.textContent = t(action);
    }
  }

  private async runAction(): Promise<void> {
    switch (this.state) {
      case 'none':
      case 'ready':
        this.openSettings();
        return;
      case 'locked':
        new AiUnlockDialog(this.ctx).show();
        return;
      case 'consent':
        await this.askConsent();
    }
  }

  /** 키가 준비됐는데 동의가 없으면, 패널을 처음 열 때 한 번만 묻는다. */
  private async maybeAskConsent(): Promise<void> {
    if (!this.visible || this.consentAsked || this.state !== 'consent') return;
    await this.askConsent();
  }

  private async askConsent(): Promise<void> {
    this.consentAsked = true;
    const provider = this.ctx.settings.provider;
    const agreed = await showConfirm(
      t('ai.consent.title'),
      t('ai.consent.message', { provider: t(`ai.provider.${provider}`) }),
    );
    if (agreed) this.ctx.updateSettings({ consent: { at: new Date().toISOString(), provider } });
  }

  private setBusy(busy: boolean): void {
    this.sendButton.textContent = busy ? t('ai.chat.stop') : t('ai.chat.send');
    this.sendButton.classList.toggle('ai-stop-btn', busy);
    this.root?.toggleAttribute('data-busy', busy);
    for (const quick of this.quick.querySelectorAll('button')) quick.disabled = busy;
  }

  private nearBottom(): boolean {
    return this.log.scrollHeight - this.log.scrollTop - this.log.clientHeight < 40;
  }

  private scrollToEnd(force = false): void {
    if (force || this.nearBottom()) this.log.scrollTop = this.log.scrollHeight;
  }

  async send(rawText: string): Promise<void> {
    const text = rawText.trim();
    if (!text || this.chat.busy || this.state !== 'ready') return;
    this.input.value = '';
    this.empty.hidden = true;
    const userView = el('div', 'ai-msg ai-msg-user', text);
    const answer = new AssistantView();
    this.log.append(userView, answer.root);
    this.scrollToEnd(true);
    this.setBusy(true);

    const presenter: ProposalPresenter = {
      present: (proposal, options) => {
        const handle = answer.proposal(proposal, options.autoApply, options.signal);
        this.scrollToEnd(true);
        return handle;
      },
    };
    this.chat.setPresenter(presenter);
    let result: AgentRunResult | null = null;
    let failure: unknown = null;
    try {
      result = await this.chat.ask(text, {
        onTurnStart: () => answer.turnStart(),
        onText: (delta) => { answer.appendText(delta); this.scrollToEnd(); },
        onToolStart: (call) => { answer.toolStart(call); this.scrollToEnd(); },
        onToolEnd: (call, toolResult) => answer.toolEnd(call, toolResult.isError),
      });
    } catch (error) {
      failure = error;
    }
    this.chat.setPresenter(null);
    this.setBusy(false);

    if (result) {
      this.usage = addUsage(this.usage, result.usage);
      this.renderUsage();
      const note = ({
        done: null,
        aborted: t('ai.chat.stopped'),
        refusal: t('ai.chat.refused'),
        max_tokens: t('ai.chat.truncated'),
        round_limit: t('ai.chat.roundLimit'),
        error: result.error
          ? failureMessage(result.error.reason, result.error.retryAfterSeconds, result.error.detail)
          : t('ai.error.unknown'),
      } as const)[result.status];
      answer.finish(note, result.status === 'error' || result.status === 'refusal' ? 'error' : 'info');
      // 원인 진단용. 실패 정보에는 키가 없다.
      if (result.error) console.error('[AI] 요청 실패', result.error.reason, result.error.status ?? '', result.error.detail ?? '');
    } else {
      const reason = failure instanceof AiProviderError ? failure.reason : 'unknown';
      answer.finish(failureMessage(reason), 'error');
      if (!(failure instanceof AiProviderError)) console.error('[AI] 질문 처리 실패', failure);
    }
    this.scrollToEnd();
    if (this.visible) this.input.focus();
  }

  private newConversation(): void {
    this.chat.reset();
    this.clearLog(null);
    this.input.focus();
  }

  private clearLog(notice: string | null): void {
    if (!this.root) return;
    this.log.replaceChildren(this.empty);
    this.empty.hidden = false;
    if (notice) this.log.append(el('p', 'ai-chat-notice', notice));
    this.usage = EMPTY_USAGE;
    this.renderUsage();
    this.setBusy(false);
  }

  private renderUsage(): void {
    const fmt = (n: number) => n.toLocaleString();
    this.usageLine.textContent = t('ai.chat.usage', {
      input: fmt(this.usage.inputTokens + this.usage.cacheReadTokens + this.usage.cacheWriteTokens),
      cached: fmt(this.usage.cacheReadTokens),
      output: fmt(this.usage.outputTokens),
    });
  }

  /** 커서가 움직일 때마다가 아니라 화면 갱신 주기에 한 번, 패널이 보일 때만 선택을 확인한다. */
  private scheduleSelectionCheck(): void {
    if (!this.visible || this.selectionFrame) return;
    this.selectionFrame = requestAnimationFrame(() => {
      this.selectionFrame = 0;
      this.quick.hidden = !this.host.hasSelection();
    });
  }
}
