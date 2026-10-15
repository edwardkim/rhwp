import { t as i18nText } from '../i18n/index.ts';

/** 편집기가 만들 수 없는 채우기도 원래 종류로 표시한다. none은 명시적인 삭제다. */
export function populateFillSelection(select: HTMLSelectElement, fillType = 'none'): void {
  for (const option of Array.from(select.options)) {
    if (option.dataset.preservedFill) option.remove();
  }
  if (fillType !== 'none' && fillType !== 'solid') {
    const option = document.createElement('option');
    option.value = fillType;
    option.dataset.preservedFill = 'true';
    option.textContent = fillType === 'gradient'
      ? i18nText('dialog.fill.keepGradient')
      : fillType === 'image'
        ? i18nText('dialog.fill.keepImage')
        : i18nText('dialog.fill.keepExisting');
    select.appendChild(option);
  }
  select.value = fillType;
}

export function updateFillControls(
  select: HTMLSelectElement,
  controls: Array<HTMLInputElement | HTMLSelectElement>,
): void {
  for (const control of controls) control.disabled = select.value !== 'solid';
}
