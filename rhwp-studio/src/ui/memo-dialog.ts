import { ModalDialog } from './dialog';
import type { MemoEdit, MemoInfo } from '@/core/memo-editor';

/** 모든 글은 textContent/value로 표시해 문서 메모를 HTML로 해석하지 않는다. */
export class MemoDialog extends ModalDialog {
  private list!: HTMLSelectElement;
  private author!: HTMLInputElement;
  private text!: HTMLTextAreaElement;
  private error!: HTMLParagraphElement;
  private remove!: HTMLButtonElement;
  private selected: number | null = null;
  private drafts = new Map<number | null, { author: string; text: string }>();
  constructor(private memos: MemoInfo[], private apply: (edit: MemoEdit) => MemoInfo[]) {
    super('메모', 560);
  }
  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.style.display = 'flex'; body.style.flexDirection = 'column'; body.style.gap = '8px';
    this.list = document.createElement('select');
    this.list.setAttribute('aria-label', '문서의 메모');
    this.list.style.width = '100%';
    this.list.size = 5;
    this.list.addEventListener('change', () => this.select(this.list.value === '' ? null : Number(this.list.value)));
    const label = document.createElement('p');
    label.textContent = '새 메모는 대화상자를 열기 전의 커서 또는 한 문단 선택 범위에 연결됩니다.';
    const authorLabel = document.createElement('label');
    authorLabel.textContent = '작성자';
    this.author = document.createElement('input');
    this.author.id = 'memo-author'; authorLabel.htmlFor = this.author.id;
    this.author.className = 'dialog-input'; this.author.maxLength = 64; this.author.style.width = '100%'; this.author.style.textAlign = 'left';
    const textLabel = document.createElement('label');
    textLabel.textContent = '메모 내용';
    this.text = document.createElement('textarea');
    this.text.id = 'memo-text'; textLabel.htmlFor = this.text.id;
    this.text.maxLength = 100000;
    this.text.rows = 6; this.text.style.width = '100%';
    this.remove = document.createElement('button');
    this.remove.style.alignSelf = 'flex-start';
    this.remove.type = 'button'; this.remove.className = 'dialog-btn';
    this.remove.textContent = '선택한 메모 삭제';
    this.remove.addEventListener('click', () => {
      if (this.selected !== null) this.submit({ kind: 'remove', id: this.selected });
    });
    this.error = document.createElement('p'); this.error.setAttribute('role', 'alert');
    body.append(this.list, label, authorLabel, this.author, textLabel, this.text, this.remove, this.error);
    this.refresh();
    return body;
  }
  private refresh(): void {
    this.list.replaceChildren();
    const fresh = document.createElement('option'); fresh.value = ''; fresh.textContent = '새 메모';
    this.list.append(fresh);
    for (const memo of this.memos) {
      const option = document.createElement('option'); option.value = String(memo.id);
      option.textContent = [
        memo.parentId == null ? '' : '↳ 댓글',
        memo.page ? `${memo.page}쪽` : '쪽 위치 없음',
        memoLocation(memo),
        memo.author || '작성자 없음',
        memo.text.slice(0, 60),
      ].filter(Boolean).join(' · ');
      this.list.append(option);
    }
    this.select(null, false);
  }
  private select(id: number | null, remember = true): void {
    if (remember) this.drafts.set(this.selected, { author: this.author.value, text: this.text.value });
    const draft = this.drafts.get(id);
    this.selected = id; this.list.value = id === null ? '' : String(id);
    const memo = this.memos.find(m => m.id === id);
    this.author.value = draft?.author ?? memo?.author ?? ''; this.author.disabled = !!memo;
    this.text.value = draft?.text ?? memo?.text ?? ''; this.text.disabled = !!memo && !memo.editable;
    this.remove.disabled = !memo?.editable;
    this.error.textContent = memo && !memo.editable ? '이 메모는 현재 편집할 수 없습니다.' : '';
  }
  private submit(edit: MemoEdit): void {
    try { this.memos = this.apply(edit); this.drafts.delete(this.selected); this.refresh(); this.error.textContent = '메모를 반영했습니다.'; }
    catch (error) { this.error.textContent = error instanceof Error ? error.message : String(error); }
  }
  protected onConfirm(): boolean {
    this.submit(this.selected === null
      ? { kind: 'insert', author: this.author.value, text: this.text.value }
      : { kind: 'update', id: this.selected, text: this.text.value });
    return false;
  }
  override show(): void {
    super.show();
    this.dialog.dataset.memoDialog = '';
    this.dialog.setAttribute('role', 'dialog'); this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.setAttribute('aria-label', '메모');
    this.dialog.querySelector('.dialog-btn-primary')!.textContent = '반영';
    this.dialog.querySelector('.dialog-footer .dialog-btn:not(.dialog-btn-primary)')!.textContent = '닫기';
    this.text.focus();
  }
}

function memoLocation(memo: MemoInfo): string {
  if (memo.paraIndex === null || memo.startCharIdx === null || memo.endCharIdx === null) return '';
  const parts = [`본문 문단 ${memo.paraIndex + 1}`];
  for (const entry of memo.path ?? []) {
    const object = entry.type === 'cell'
      ? `표 셀 ${entry.cellIndex + 1} (개체 ${entry.controlIndex + 1})`
      : `글상자 (개체 ${entry.controlIndex + 1})`;
    parts.push(`${object} 문단 ${entry.paraIndex + 1}`);
  }
  const start = memo.startCharIdx + 1;
  parts.push(memo.startCharIdx === memo.endCharIdx
    ? `글자 ${start} 앞`
    : `글자 ${start}–${memo.endCharIdx}`);
  return parts.join(' / ');
}
