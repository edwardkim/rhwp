/**
 * 대표 글꼴 추가/편집 대화상자
 *
 * 이름 입력 + 7개 언어별 글꼴 드롭다운
 */
import { ModalDialog } from './dialog';
import { LANG_LABELS } from '@/core/user-settings';
import type { FontSet } from '@/core/user-settings';
import { REGISTERED_FONTS } from '@/core/font-loader';
import { getLocalFonts } from '@/core/local-fonts';

import { t } from '../i18n/index.ts';
/** 웹폰트 목록 (중복 제거 + 정렬) */
function getWebFonts(): string[] {
  const unique = new Set<string>();
  for (const name of REGISTERED_FONTS) unique.add(name);
  ['serif', 'sans-serif', 'monospace'].forEach(f => unique.add(f));
  return Array.from(unique).sort((a, b) => a.localeCompare(b, 'ko'));
}

export class FontSetEditDialog extends ModalDialog {
  private nameInput!: HTMLInputElement;
  private langSelects: HTMLSelectElement[] = [];
  private editTarget: FontSet | null;
  private onApply: (fs: FontSet) => void;

  constructor(editTarget: FontSet | null, onApply: (fs: FontSet) => void) {
    super(editTarget ? t('dialog.fontSetEdit.getWebFonts.title') : t('dialog.fontSetEdit.getWebFonts.title.xf9e4d2'), 400);
    this.editTarget = editTarget;
    this.onApply = onApply;
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'fse-body';
    const webFonts = getWebFonts();
    const localFonts = getLocalFonts();
    const langKeys: (keyof Omit<FontSet, 'name'>)[] = [
      'korean', 'english', 'chinese', 'japanese', 'other', 'symbol', 'user',
    ];

    // 이름 입력
    const nameRow = document.createElement('div');
    nameRow.className = 'dialog-row fse-row';

    const nameLabel = document.createElement('label');
    nameLabel.className = 'dialog-label fse-label';
    nameLabel.textContent = t('dialog.fontSetEdit.nameLabel.text');

    this.nameInput = document.createElement('input');
    this.nameInput.type = 'text';
    this.nameInput.className = 'dialog-input fse-name-input';
    this.nameInput.placeholder = t('dialog.fontSetEdit.nameInput.placeholder');
    if (this.editTarget) this.nameInput.value = this.editTarget.name;

    nameRow.appendChild(nameLabel);
    nameRow.appendChild(this.nameInput);
    body.appendChild(nameRow);

    // 구분선
    const sep = document.createElement('hr');
    sep.className = 'fse-sep';
    body.appendChild(sep);

    // 7개 언어별 글꼴 선택
    LANG_LABELS.forEach((label, i) => {
      const row = document.createElement('div');
      row.className = 'dialog-row fse-row';

      const lbl = document.createElement('label');
      lbl.className = 'dialog-label fse-label';
      lbl.textContent = label;

      const select = document.createElement('select');
      select.className = 'dialog-select fse-select';

      // 웹폰트 optgroup
      const webGroup = document.createElement('optgroup');
      webGroup.label = '웹 글꼴';
      for (const fontName of webFonts) {
        const opt = document.createElement('option');
        opt.value = fontName;
        opt.textContent = fontName;
        webGroup.appendChild(opt);
      }
      select.appendChild(webGroup);

      // 로컬 글꼴 optgroup (감지된 경우에만)
      if (localFonts.length > 0) {
        const localGroup = document.createElement('optgroup');
        localGroup.label = '로컬 글꼴';
        for (const fontName of localFonts) {
          const opt = document.createElement('option');
          opt.value = fontName;
          opt.textContent = fontName;
          localGroup.appendChild(opt);
        }
        select.appendChild(localGroup);
      }

      // 기존 값 설정
      if (this.editTarget) {
        select.value = this.editTarget[langKeys[i]];
      } else {
        select.value = '함초롬바탕'; // 기본값
      }

      this.langSelects.push(select);

      row.appendChild(lbl);
      row.appendChild(select);
      body.appendChild(row);
    });

    return body;
  }

  protected onConfirm(): void {
    const name = this.nameInput.value.trim();
    if (!name) {
      alert(t('dialog.fontSetEdit.onConfirm.message'));
      return;
    }

    const fs: FontSet = {
      name,
      korean: this.langSelects[0].value,
      english: this.langSelects[1].value,
      chinese: this.langSelects[2].value,
      japanese: this.langSelects[3].value,
      other: this.langSelects[4].value,
      symbol: this.langSelects[5].value,
      user: this.langSelects[6].value,
    };

    this.onApply(fs);
  }
}
