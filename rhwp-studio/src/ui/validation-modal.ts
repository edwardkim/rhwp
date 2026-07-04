/**
 * HWPX 비표준 lineseg 감지 알림 모달 (#177).
 *
 * 문서 로드 완료 직후 `wasm.getValidationWarnings()` 결과를 받아,
 * 경고가 있으면 사용자에게 모달로 알리고 선택지를 제공한다.
 *
 * 버튼:
 *  - [자동 보정] (기본 선택 · Primary) — 보정 후 렌더 재계산
 *  - [그대로 보기] — 모달만 닫음
 *  - [상세 보기] — 경고 목록 펼침 토글
 *
 * 기본 선택은 Enter 키로 자동 보정이 실행되도록 한다 (작업지시자 지시).
 *
 * Discussion #188 원칙:
 *  - 한컴이 조용히 보정하는 비표준 lineseg 를 rhwp 는 사용자에게 고지한다
 *  - 자동 수정은 사용자 명시 선택 후에만 실행
 *  - 기본값은 자동 보정(권장) — 대부분 사용자가 원하는 동작
 */

import type { ValidationReport } from '../core/wasm-bridge';
import { enableDialogDrag } from './dialog-drag';

/** 모달이 반환하는 사용자 선택. */
export type ValidationChoice = 'auto-fix' | 'as-is' | 'cancel';

export class ValidationModal {
  private overlay: HTMLDivElement | null = null;
  private captureHandler: ((e: KeyboardEvent) => void) | null = null;
  private resolver: ((choice: ValidationChoice) => void) | null = null;
  private report: ValidationReport;

  constructor(report: ValidationReport) {
    this.report = report;
  }

  /** 모달 표시 + 사용자 선택 반환. */
  async showAsync(): Promise<ValidationChoice> {
    return new Promise((resolve) => {
      this.resolver = resolve;
      this.build();
      document.body.appendChild(this.overlay!);
      this.bindKeyboard();

      // 기본 포커스: 자동 보정 버튼
      const primaryBtn = this.overlay!.querySelector(
        '.dialog-btn-primary',
      ) as HTMLButtonElement | null;
      primaryBtn?.focus();
    });
  }

  private build(): void {
    this.overlay = document.createElement('div');
    this.overlay.className = 'modal-overlay';

    const dialog = document.createElement('div');
    dialog.className = 'dialog-wrap';
    dialog.style.width = '480px';

    // 타이틀
    const title = document.createElement('div');
    title.className = 'dialog-title';
    title.textContent = 'HWPX 비표준 감지';
    const closeBtn = document.createElement('button');
    closeBtn.className = 'dialog-close';
    closeBtn.textContent = '\u00D7';
    closeBtn.addEventListener('click', () => this.resolve('cancel'));
    title.appendChild(closeBtn);
    dialog.appendChild(title);
    enableDialogDrag(dialog, title);

    // 본문
    const body = document.createElement('div');
    body.className = 'dialog-body';
    body.style.padding = '16px 20px';
    body.style.lineHeight = '1.6';

    const desc = document.createElement('p');
    desc.style.margin = '0 0 12px 0';
    desc.textContent = `이 문서는 HWPX 명세를 일부 준수하지 않는 값을 포함합니다 (경고 ${this.report.count}건). 렌더링 품질을 위해 자동 보정을 권장합니다.`;
    body.appendChild(desc);

    // 경고 요약
    const summary = document.createElement('ul');
    summary.style.margin = '0 0 12px 16px';
    summary.style.padding = '0';
    summary.style.fontSize = '13px';
    summary.style.color = 'var(--color-text-secondary)';
    for (const [kind, cnt] of Object.entries(this.report.summary)) {
      const li = document.createElement('li');
      li.textContent = `${kind}: ${cnt}건`;
      summary.appendChild(li);
    }
    body.appendChild(summary);

    // 상세 보기 (토글 영역, 기본 닫힘)
    const details = document.createElement('details');
    details.style.marginTop = '8px';
    const summaryEl = document.createElement('summary');
    summaryEl.textContent = '상세 보기';
    summaryEl.style.cursor = 'pointer';
    summaryEl.style.fontSize = '13px';
    summaryEl.style.color = 'var(--ui-link)';
    details.appendChild(summaryEl);

    const detailList = document.createElement('div');
    detailList.style.maxHeight = '180px';
    detailList.style.overflow = 'auto';
    detailList.style.marginTop = '8px';
    detailList.style.padding = '8px';
    detailList.style.background = 'var(--color-surface-raised)';
    detailList.style.borderRadius = '4px';
    detailList.style.fontFamily = 'monospace';
    detailList.style.fontSize = '12px';
    detailList.style.color = 'var(--color-text)';

    const maxShow = 50;
    const shown = this.report.warnings.slice(0, maxShow);
    for (const w of shown) {
      const line = document.createElement('div');
      const cellStr = w.cell
        ? ` [셀 ctrl=${w.cell.ctrl} row=${w.cell.row} col=${w.cell.col} para=${w.cell.innerPara}]`
        : '';
      line.textContent = `section=${w.section} para=${w.paragraph} ${w.kind}${cellStr}`;
      detailList.appendChild(line);
    }
    if (this.report.warnings.length > maxShow) {
      const more = document.createElement('div');
      more.style.color = 'var(--color-text-hint)';
      more.style.marginTop = '4px';
      more.textContent = `... 외 ${this.report.warnings.length - maxShow}건`;
      detailList.appendChild(more);
    }
    details.appendChild(detailList);
    body.appendChild(details);

    dialog.appendChild(body);

    // 하단 버튼 영역 (3버튼)
    const footer = document.createElement('div');
    footer.className = 'dialog-footer';

    const autoFixBtn = document.createElement('button');
    autoFixBtn.className = 'dialog-btn dialog-btn-primary';
    autoFixBtn.textContent = '자동 보정 (권장)';
    autoFixBtn.addEventListener('click', () => this.resolve('auto-fix'));

    const asIsBtn = document.createElement('button');
    asIsBtn.className = 'dialog-btn';
    asIsBtn.textContent = '그대로 보기';
    asIsBtn.addEventListener('click', () => this.resolve('as-is'));

    footer.appendChild(autoFixBtn);
    footer.appendChild(asIsBtn);
    dialog.appendChild(footer);

    this.overlay.appendChild(dialog);
  }

  private bindKeyboard(): void {
    this.captureHandler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.stopPropagation();
        e.preventDefault();
        this.resolve('cancel');
        return;
      }
      if (e.key === 'Enter') {
        // 기본 선택 = 자동 보정
        e.stopPropagation();
        e.preventDefault();
        this.resolve('auto-fix');
        return;
      }
      // 다른 키는 외부 전파 차단 (편집 영역으로 빠지지 않도록)
      e.stopPropagation();
    };
    document.addEventListener('keydown', this.captureHandler, true);
  }

  private resolve(choice: ValidationChoice): void {
    if (this.captureHandler) {
      document.removeEventListener('keydown', this.captureHandler, true);
      this.captureHandler = null;
    }
    this.overlay?.remove();
    this.overlay = null;
    if (this.resolver) {
      this.resolver(choice);
      this.resolver = null;
    }
  }
}

/**
 * 경고가 있으면 모달을 표시하고 사용자 선택을 반환한다.
 * 경고 0건이면 모달 미생성 (비침습) — 즉시 'as-is' 반환.
 */
export async function showValidationModalIfNeeded(
  report: ValidationReport,
): Promise<ValidationChoice> {
  if (!report || report.count === 0) return 'as-is';
  return new ValidationModal(report).showAsync();
}
