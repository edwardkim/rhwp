import { ModalDialog } from './dialog';
import { countMemoReplies, type MemoEdit, type MemoInfo } from '@/core/memo-editor';
import { showConfirm } from './confirm-dialog';
import { t } from '../i18n/index.ts';

/** 모든 글은 textContent/value로 표시해 문서 메모를 HTML로 해석하지 않는다. */
export class MemoDialog extends ModalDialog {
  private list!: HTMLDivElement;
  private options = new Map<number | null, HTMLButtonElement>();
  private targetHelp!: HTMLParagraphElement;
  private location!: HTMLDetailsElement;
  private address!: HTMLSpanElement;
  private author!: HTMLInputElement;
  private text!: HTMLTextAreaElement;
  private error!: HTMLParagraphElement;
  private remove!: HTMLButtonElement;
  private goTo!: HTMLButtonElement;
  private reply!: HTMLButtonElement;
  private cancelReply!: HTMLButtonElement;
  private relation!: HTMLParagraphElement;
  private replying = false;
  private confirm?: HTMLButtonElement;
  private selected: number | null = null;
  private drafts = new Map<number | null, { author: string; text: string }>();

  constructor(
    private memos: MemoInfo[],
    private insertAvailable: boolean,
    private pageOf: (memo: MemoInfo) => number | null,
    private apply: (edit: MemoEdit) => MemoInfo[],
    private navigation: { initialId: number | null; goTo: (memo: MemoInfo) => boolean },
  ) {
    super(t('dialog.memo.title'), 560);
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'memo-dialog-body';
    this.list = document.createElement('div');
    this.list.className = 'memo-dialog-list';
    this.list.setAttribute('role', 'listbox');
    this.list.setAttribute('aria-label', t('dialog.memo.list'));

    this.targetHelp = document.createElement('p');
    this.targetHelp.textContent = t(this.insertAvailable ? 'dialog.memo.targetHelp' : 'dialog.memo.invalidTarget');
    this.location = document.createElement('details');
    this.location.className = 'memo-dialog-location';
    const summary = document.createElement('summary');
    summary.textContent = t('dialog.memo.locationDetails');
    this.address = document.createElement('span');
    this.location.append(summary, this.address);

    const authorLabel = document.createElement('label');
    authorLabel.textContent = t('dialog.memo.author');
    this.author = document.createElement('input');
    this.author.id = 'memo-author';
    authorLabel.htmlFor = this.author.id;
    this.author.className = 'dialog-input';
    this.author.maxLength = 64;
    this.author.style.width = '100%';
    this.author.style.textAlign = 'left';

    const textLabel = document.createElement('label');
    textLabel.textContent = t('dialog.memo.text');
    this.text = document.createElement('textarea');
    this.text.id = 'memo-text';
    textLabel.htmlFor = this.text.id;
    this.text.maxLength = 100000;
    this.text.rows = 6;
    this.text.style.width = '100%';

    this.remove = document.createElement('button');
    this.remove.style.alignSelf = 'flex-start';
    this.remove.type = 'button';
    this.remove.className = 'dialog-btn memo-delete';
    this.remove.textContent = t('dialog.memo.delete');
    this.remove.addEventListener('click', () => { void this.removeSelected(); });

    this.goTo = document.createElement('button');
    this.goTo.type = 'button';
    this.goTo.className = 'dialog-btn memo-go-to';
    this.goTo.textContent = t('dialog.memo.goTo');
    this.goTo.addEventListener('click', () => {
      const current = this.memos.find(memo => memo.id === this.selected);
      if (current && this.navigation.goTo(current)) this.hide();
      else this.error.textContent = t('dialog.memo.noLocation');
    });

    this.reply = document.createElement('button');
    this.reply.type = 'button';
    this.reply.className = 'dialog-btn memo-add-reply';
    this.reply.textContent = t('dialog.memo.addReply');
    this.reply.addEventListener('click', () => this.beginReply());

    this.cancelReply = document.createElement('button');
    this.cancelReply.type = 'button';
    this.cancelReply.className = 'dialog-btn memo-cancel-reply';
    this.cancelReply.textContent = t('dialog.memo.cancelReply');
    this.cancelReply.addEventListener('click', () => {
      this.select(this.selected, false);
      this.reply.focus();
    });
    this.relation = document.createElement('p');
    this.relation.hidden = true;

    this.error = document.createElement('p');
    this.error.setAttribute('role', 'alert');
    const actions = document.createElement('div');
    actions.style.display = 'flex';
    actions.style.gap = '6px';
    actions.style.flexWrap = 'wrap';
    actions.append(this.remove, this.goTo, this.reply, this.cancelReply);
    body.append(this.list, this.targetHelp, this.relation, this.location, authorLabel, this.author, textLabel, this.text, actions, this.error);
    this.refresh(this.navigation.initialId);
    return body;
  }

  private refresh(preferred: number | null = null): void {
    this.list.replaceChildren();
    this.options.clear();
    const fresh = this.appendOption(null, t('dialog.memo.new'));
    fresh.disabled = !this.insertAvailable;
    this.list.tabIndex = this.insertAvailable || this.memos.length ? -1 : 0;

    for (const memo of this.memos) {
      const option = this.appendOption(memo.id, memo.text);
      option.classList.toggle('is-reply', memo.parentId !== null);
      const meta = document.createElement('span');
      meta.className = 'memo-list-meta';
      const page = this.pageOf(memo);
      const target = memo.path?.at(-1);
      const fields = [
        memo.parentId === null ? '' : t('dialog.memo.reply'),
        memo.author || t('dialog.memo.noAuthor'),
        memo.date?.replace('T', ' ').slice(0, 16),
        page === null ? '' : t('dialog.memo.page', { p1: page }),
        memo.paraIndex === null ? '' : t(target?.type === 'cell' ? 'dialog.memo.areaCell'
          : target?.type === 'textbox' ? 'dialog.memo.areaTextBox' : 'dialog.memo.areaBody'),
      ];
      meta.textContent = fields.filter(Boolean).join(' · ');
      option.append(meta);
    }

    const fallback = this.insertAvailable ? null : this.memos[0]?.id ?? null;
    this.select(this.memos.some(memo => memo.id === preferred) ? preferred : fallback, false);
  }

  private appendOption(id: number | null, text: string): HTMLButtonElement {
    const option = document.createElement('button');
    option.type = 'button';
    option.setAttribute('role', 'option');
    option.dataset.memoChoice = id === null ? '' : String(id);
    const preview = document.createElement('span');
    preview.className = 'memo-list-preview';
    preview.textContent = text.trim() || t('dialog.memo.emptyPreview');
    option.append(preview);
    option.addEventListener('click', () => {
      if (this.selected !== id) this.select(id);
    });
    this.options.set(id, option);
    this.list.append(option);
    return option;
  }

  protected override onDialogKey(event: KeyboardEvent): boolean {
    const keys = ['ArrowUp', 'ArrowDown', 'Home', 'End'];
    if (!this.list.contains(document.activeElement) || !keys.includes(event.key)) return false;
    const choices = [...this.options].filter(([, option]) => !option.disabled);
    if (!choices.length) return false;
    const index = choices.findIndex(([id]) => id === this.selected);
    const next = event.key === 'Home' ? 0 : event.key === 'End' ? choices.length - 1
      : Math.max(0, Math.min(choices.length - 1, index + (event.key === 'ArrowUp' ? -1 : 1)));
    const [id, option] = choices[next];
    if (id !== this.selected) this.select(id);
    option.focus();
    return true;
  }

  private focusList(): void {
    const selected = this.options.get(this.selected);
    (selected && !selected.disabled ? selected : this.list).focus();
  }

  private select(id: number | null, remember = true): void {
    if (remember && !this.replying) this.rememberDraft();
    this.replying = false;
    this.reply.hidden = false;
    this.cancelReply.hidden = true;
    this.cancelReply.disabled = true;
    const draft = this.drafts.get(id);
    const memo = this.memos.find(m => m.id === id);
    this.selected = id;
    for (const [choice, option] of this.options) {
      const selected = choice === id && !option.disabled;
      option.setAttribute('aria-selected', String(selected));
      option.tabIndex = selected ? 0 : -1;
      if (selected) option.scrollIntoView({ block: 'nearest' });
    }
    this.targetHelp.hidden = !!memo;
    this.address.textContent = memo ? memoLocation(memo) : '';
    this.location.hidden = !this.address.textContent;
    if (remember) this.location.open = false;
    this.author.value = draft?.author ?? memo?.author ?? '';
    this.author.disabled = !!memo || !this.insertAvailable;
    this.text.value = draft?.text ?? memo?.text ?? '';
    this.text.disabled = memo ? !memo.editable : !this.insertAvailable;
    if (this.confirm) this.confirm.disabled = this.text.disabled;
    this.remove.disabled = !memo?.editable;
    this.reply.disabled = !memo?.editable || memo.parentId !== null;
    this.setRelation(memo ? this.parentLabel(memo) : '');
    this.goTo.disabled = !memo || this.pageOf(memo) === null;
    this.goTo.title = memo && this.goTo.disabled ? t('dialog.memo.noLocation') : '';
    this.error.textContent = memo && !memo.editable ? t('dialog.memo.readOnly') : '';
  }

  private parentLabel(memo: MemoInfo): string {
    if (memo.parentId === null) return '';
    const parent = this.memos.find(item => item.id === memo.parentId);
    return t('dialog.memo.parentMemo', { p1: parent
      ? this.memoLabel(parent) : '#' + memo.parentId });
  }

  private memoLabel(memo: MemoInfo): string {
    return '#' + (memo.index + 1) + ' ' + memo.text.slice(0, 44);
  }

  private setRelation(label: string): void {
    this.relation.hidden = !label;
    this.relation.textContent = label;
  }

  private rememberDraft(): void {
    this.drafts.set(this.selected, { author: this.author.value, text: this.text.value });
  }

  private beginReply(): void {
    const parent = this.memos.find(item => item.id === this.selected);
    if (!parent?.editable || parent.parentId !== null) return;
    this.rememberDraft();
    this.replying = true;
    this.location.hidden = true;
    this.reply.hidden = true;
    this.reply.disabled = true;
    this.cancelReply.hidden = false;
    this.cancelReply.disabled = false;
    this.goTo.disabled = true;
    this.remove.disabled = true;
    this.author.disabled = false;
    this.author.value = '';
    this.text.disabled = false;
    this.text.value = '';
    if (this.confirm) this.confirm.disabled = false;
    this.setRelation(t('dialog.memo.parentMemo', { p1: this.memoLabel(parent) }));
    this.error.textContent = '';
    this.text.focus();
  }

  private async removeSelected(): Promise<void> {
    const id = this.selected;
    if (id === null || this.replying) return;
    const count = countMemoReplies(id, this.memos);
    if (count > 0) {
      const confirmed = await showConfirm(t('dialog.memo.deleteThread'),
        t('dialog.memo.deleteThreadConfirm', { p1: count }));
      if (!confirmed) {
        if (this.dialog.isConnected) this.remove.focus();
        return;
      }
    }
    if (this.dialog.isConnected) this.submit({ kind: 'remove', id, confirmedReplies: count });
  }

  private submit(edit: MemoEdit): void {
    try {
      const previous = this.memos;
      this.memos = this.apply(edit);
      if (edit.kind !== 'reply') this.drafts.delete(this.selected);
      const preferred = edit.kind === 'update' ? edit.id : edit.kind === 'reply'
        ? this.memos.find(memo => memo.parentId === edit.id && !previous.some(item => item.id === memo.id))?.id ?? edit.id
        : null;
      this.refresh(preferred);
      if (edit.kind === 'remove') this.focusList();
      this.error.textContent = t('dialog.memo.applied');
    } catch (error) {
      this.error.textContent = error instanceof Error ? error.message : String(error);
    }
  }

  protected onConfirm(): boolean {
    if (this.confirm?.disabled) return false;
    this.submit(this.selected === null
      ? { kind: 'insert', author: this.author.value, text: this.text.value }
      : this.replying
        ? { kind: 'reply', id: this.selected, author: this.author.value, text: this.text.value }
        : { kind: 'update', id: this.selected, text: this.text.value });
    return false;
  }

  protected override onTabKey(event: KeyboardEvent): boolean {
    const controls = [...this.dialog.querySelectorAll<HTMLElement>(
      'button:not(:disabled), input:not(:disabled), textarea:not(:disabled), summary, [role="listbox"]',
    )].filter(control => control.tabIndex >= 0 && control.getClientRects().length > 0);
    const current = controls.indexOf(document.activeElement as HTMLElement);
    const next = event.shiftKey
      ? (current <= 0 ? controls.length - 1 : current - 1)
      : (current + 1) % controls.length;
    controls[next].focus();
    return true;
  }

  override show(): void {
    super.show();
    this.dialog.dataset.memoDialog = '';
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.setAttribute('aria-label', t('dialog.memo.title'));
    this.confirm = this.dialog.querySelector<HTMLButtonElement>('.dialog-btn-primary')!;
    this.confirm.textContent = t('dialog.memo.apply');
    this.confirm.disabled = this.text.disabled;
    this.dialog.querySelector('.dialog-footer .dialog-btn:not(.dialog-btn-primary)')!.textContent = t('dialog.memo.close');
    this.dialog.querySelector('.dialog-close')?.setAttribute('aria-label', t('dialog.memo.close'));
    if (!this.text.disabled) this.text.focus();
    else this.focusList();
  }
}

function memoLocation(memo: MemoInfo): string {
  if (memo.paraIndex === null || memo.startCharIdx === null || memo.endCharIdx === null) return '';
  const parts = [t('dialog.memo.paragraph', { p1: memo.paraIndex + 1 })];
  for (const entry of memo.path ?? []) {
    const object = entry.type === 'cell'
      ? t('dialog.memo.cell', { p1: entry.cellIndex + 1, p2: entry.controlIndex + 1 })
      : t('dialog.memo.textBox', { p1: entry.controlIndex + 1 });
    parts.push(t('dialog.memo.innerParagraph', { p1: object, p2: entry.paraIndex + 1 }));
  }
  const start = memo.startCharIdx + 1;
  parts.push(memo.startCharIdx === memo.endCharIdx
    ? t('dialog.memo.point', { p1: start })
    : t('dialog.memo.range', { p1: start, p2: memo.endCharIdx }));
  return parts.join(' / ');
}
