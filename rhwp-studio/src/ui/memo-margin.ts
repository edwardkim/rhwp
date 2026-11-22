import type { EventBus } from '@/core/event-bus';
import type { MemoComposeRequest, MemoInfo } from '@/core/memo-editor';
import { t } from '../i18n/index.ts';
import { LocalContextMenu } from './local-context-menu';

export interface LocatedMemo {
  memo: MemoInfo;
  replies: MemoInfo[];
  x: number;
  y: number;
  edge: number;
}

export type MemoAction = (id: number, action?: 'update' | 'reply' | 'remove') => void;

const SVG_NS = 'http://www.w3.org/2000/svg';

function action(label: string, invoke: () => void, className = ''): HTMLButtonElement {
  const button = document.createElement('button');
  button.type = 'button';
  button.textContent = label;
  button.className = className;
  button.addEventListener('click', invoke);
  return button;
}

export class MemoMargin {
  private panel = document.createElement('aside');
  private cards = document.createElement('div');
  private empty = document.createElement('p');
  private guides = document.createElementNS(SVG_NS, 'svg');
  private composer: HTMLFormElement | null = null;
  private menu = new LocalContextMenu();
  private request: MemoComposeRequest | null = null;
  private connected = false;
  private selected: number | null = null;
  private hovered: number | null = null;
  private focused: number | null = null;
  private entries = new Map<number, { card: HTMLElement; signature: string }>();

  visible = false;
  annotations = true;

  get active(): number | null { return this.request?.cardId ?? this.hovered ?? this.focused ?? this.selected; }

  constructor(
    private container: HTMLElement, bus: EventBus, private open: MemoAction, private schedule: () => void,
  ) {
    const title = document.createElement('h2');
    title.textContent = t('dialog.memo.margin');
    this.panel.className = 'memo-margin';
    this.panel.setAttribute('aria-label', title.textContent);
    this.panel.hidden = true;
    this.cards.className = 'memo-margin-cards';
    this.empty.textContent = t('dialog.memo.marginEmpty');
    this.cards.append(this.empty);
    this.panel.append(title, this.cards);
    this.guides.classList.add('memo-margin-guides');
    this.guides.setAttribute('aria-hidden', 'true');
    container.after(this.panel, this.guides);
    this.cards.addEventListener('scroll', schedule, { passive: true });
    bus.on('memo-compose', value => this.compose(value as MemoComposeRequest));
    bus.on('memo-view-toggle', option => {
      if (option === 'margin') {
        this.visible = !this.visible;
        if (this.visible) this.annotations = true;
      }
      if (option === 'visibility') {
        this.annotations = !this.annotations;
        this.visible = this.annotations;
      }
      if (option === 'guides') this.connected = !this.connected;
      this.syncVisibility();
    });
    this.syncVisibility();
  }

  reset(): void {
    this.menu.hide();
    this.composer?.remove();
    this.composer = null;
    this.request = null;
    this.selected = this.hovered = this.focused = null;
    for (const { card } of this.entries.values()) card.remove();
    this.entries.clear();
  }

  private syncVisibility(): void {
    this.panel.hidden = !this.visible || !this.annotations;
    if (this.panel.hidden) this.menu.hide();
    for (const [name, value] of [['margin', this.visible], ['guides', this.connected],
      ['visibility', this.annotations]] as const) {
      for (const item of document.querySelectorAll('[data-cmd="view:memo-' + name + '"]')) {
        item.classList.toggle('active', value);
        item.setAttribute('aria-pressed', String(value));
      }
    }
    this.schedule();
  }

  private compose(request: MemoComposeRequest): void {
    if (this.composer) {
      this.composer.querySelector('textarea')?.focus();
      return;
    }
    this.request = request;
    this.visible = this.annotations = true;
    const form = document.createElement('form');
    form.className = 'memo-inline-editor';
    const title = document.createElement('strong');
    title.textContent = t(request.kind === 'insert' ? 'dialog.memo.insertHere'
      : request.kind === 'reply' ? 'dialog.memo.addReply' : 'dialog.memo.editHere');
    const author = document.createElement('input');
    author.placeholder = t('dialog.memo.author');
    author.setAttribute('aria-label', author.placeholder);
    author.value = request.author;
    author.maxLength = 64;
    author.disabled = author.hidden = request.kind === 'update';
    const textarea = document.createElement('textarea');
    textarea.setAttribute('aria-label', t('dialog.memo.text'));
    textarea.value = request.text;
    textarea.maxLength = 100000;
    textarea.rows = 4;
    const error = document.createElement('p');
    error.setAttribute('role', 'alert');
    const close = () => {
      form.remove();
      this.composer = this.request = null;
      const trigger = request.cardId === null ? null : this.entries.get(request.cardId)?.card.querySelector('button');
      if (trigger) trigger.focus();
      else request.restoreFocus();
      this.schedule();
    };
    const save = document.createElement('button');
    save.type = 'submit';
    save.textContent = t('dialog.memo.applyInline');
    const buttons = document.createElement('div');
    buttons.append(save, action(t('dialog.memo.cancelInline'), close));
    form.append(title, author, textarea, buttons, error);
    form.addEventListener('submit', event => {
      event.preventDefault();
      try { request.submit(author.value, textarea.value); close(); }
      catch (reason) { error.textContent = reason instanceof Error ? reason.message : String(reason); }
    });
    form.addEventListener('keydown', event => {
      if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); close(); }
    });
    this.composer = form;
    const card = request.cardId === null ? null : this.entries.get(request.cardId)?.card;
    (card ?? this.panel).insertBefore(form, card ? null : this.cards);
    this.syncVisibility();
    textarea.focus();
  }

  private renderCard(memo: MemoInfo, replies: MemoInfo[]): HTMLElement {
    const card = document.createElement('article');
    card.className = 'memo-card';
    card.dataset.memoId = String(memo.id);
    card.addEventListener('pointerenter', () => { this.hovered = memo.id; this.schedule(); });
    card.addEventListener('pointerleave', () => { this.hovered = null; this.schedule(); });
    card.addEventListener('focusin', () => { this.focused = memo.id; this.schedule(); });
    card.addEventListener('focusout', event => {
      if (!card.contains(event.relatedTarget as Node | null)) {
        this.focused = null;
        this.schedule();
      }
    });
    card.addEventListener('contextmenu', event => {
      if (event.target instanceof HTMLElement && event.target.closest('.memo-inline-editor')) return;
      event.preventDefault();
      this.selected = memo.id;
      this.schedule();
      const disabledReason = memo.editable ? undefined : t('dialog.memo.readOnly');
      this.menu.show(event.clientX, event.clientY, [
        { type: 'command', label: t('dialog.memo.editHere'), disabledReason, run: () => this.open(memo.id, 'update') },
        { type: 'command', label: t('dialog.memo.delete'), disabledReason, run: () => this.open(memo.id, 'remove') },
      ]);
    });
    const header = document.createElement('header');
    const author = document.createElement('strong');
    author.textContent = memo.author || t('dialog.memo.noAuthor');
    header.append(author);
    if (memo.date) {
      const date = document.createElement('time');
      date.dateTime = memo.date;
      date.textContent = memo.date.replace('T', ' ').slice(0, 16);
      header.append(date);
    }
    const body = action(memo.text, () => {
      this.selected = memo.id;
      this.schedule();
      this.open(memo.id);
    }, 'memo-card-open');
    body.title = memo.text;
    card.append(header, body);
    if (memo.editable) {
      const tools = document.createElement('div');
      tools.className = 'memo-card-actions';
      tools.append(action(t('dialog.memo.editHere'), () => this.open(memo.id, 'update')),
        action(t('dialog.memo.reply').replace(/^↳\s*/, ''), () => this.open(memo.id, 'reply')));
      card.append(tools);
    }
    if (replies.length) {
      const thread = document.createElement('details');
      const count = document.createElement('summary');
      count.textContent = t('dialog.memo.replyCount', { p1: replies.length });
      thread.append(count);
      for (const reply of replies) {
        const row = document.createElement('div');
        row.className = 'memo-card-reply';
        const text = reply.author ? reply.author + ' · ' + reply.text : reply.text;
        row.append(action(text, () => this.open(reply.id, reply.editable ? 'update' : undefined), 'memo-reply-open'));
        thread.append(row);
      }
      thread.addEventListener('toggle', this.schedule);
      card.append(thread);
    }
    return card;
  }

  update(located: readonly LocatedMemo[]): void {
    this.guides.replaceChildren();
    if (this.panel.hidden) return;
    const content = this.container.querySelector('#scroll-content')!;
    const editorRect = this.container.parentElement!.getBoundingClientRect();
    const contentRect = content.getBoundingClientRect();
    const viewRect = this.container.getBoundingClientRect();
    const railRect = this.cards.getBoundingClientRect();
    const visible = located.filter(mark => contentRect.top + mark.y >= viewRect.top
      && contentRect.top + mark.y <= viewRect.bottom).sort((a, b) => a.y - b.y);
    const ids = new Set(visible.map(({ memo }) => memo.id));
    for (const [id, { card }] of this.entries) {
      if (ids.has(id) || this.composer?.parentElement === card) continue;
      card.remove();
      this.entries.delete(id);
    }
    this.empty.hidden = visible.length !== 0;
    let next = 4;
    for (const { memo, replies, x, y, edge } of visible) {
      const signature = JSON.stringify([memo, replies]);
      let entry = this.entries.get(memo.id);
      if (!entry || (entry.signature !== signature && this.composer?.parentElement !== entry.card)) {
        const card = this.renderCard(memo, replies);
        if (entry) entry.card.replaceWith(card);
        else this.cards.append(card);
        entry = { card, signature };
        this.entries.set(memo.id, entry);
      }
      const { card } = entry;
      const active = this.active === memo.id;
      card.classList.toggle('is-active', active);
      const fromY = contentRect.top + y;
      const height = card.offsetHeight;
      const preferred = fromY - railRect.top - height / 2;
      const top = Math.max(next, Math.min(preferred, railRect.height - height - 4));
      card.style.top = String(top) + 'px';
      next = top + height + 7;

      const toY = railRect.top + top + Math.min(height / 2, 32) - this.cards.scrollTop;
      const fromX = contentRect.left + x;
      if (!this.connected || toY < railRect.top || toY > railRect.bottom
        || fromX < viewRect.left || fromX > viewRect.right) continue;
      const elbow = Math.max(fromX, Math.min(viewRect.right, contentRect.left + edge));
      const point = (px: number, py: number) => (px - editorRect.left) + ',' + (py - editorRect.top);
      const guide = document.createElementNS(SVG_NS, 'polyline');
      guide.classList.toggle('is-active', active);
      guide.setAttribute('points', [point(fromX, fromY), point(elbow, fromY), point(railRect.left, toY)].join(' '));
      this.guides.append(guide);
    }
  }
}
