/**
 * 환경 설정 대화상자 (도구 > 환경 설정)
 *
 * 탭 구조: [글꼴] (향후 [편집], [보기] 등 탭 추가 가능)
 */
import { ModalDialog } from './dialog';
import { userSettings } from '@/core/user-settings';
import { FontSetDialog } from './font-set-dialog';
import {
  clearStoredLocalFonts,
  detectLocalFonts,
  getLocalFontState,
  isLocalFontAccessSupported,
  loadStoredLocalFonts,
  type LocalFontState,
} from '@/core/local-fonts';
import type { EventBus } from '@/core/event-bus';

import { t as i18nText } from '../i18n/index.ts';
export class OptionsDialog extends ModalDialog {
  private showRecentCheck!: HTMLInputElement;
  private recentCountInput!: HTMLInputElement;
  private recoveryEnabledCheck!: HTMLInputElement;
  private recoveryIntervalInput!: HTMLInputElement;
  private idleSaveEnabledCheck!: HTMLInputElement;
  private idleDelayInput!: HTMLInputElement;
  private pdfPrintGuidanceCheck!: HTMLInputElement;

  constructor(private readonly eventBus?: EventBus) {
    super(i18nText('dialog.options.title'), 480);
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'opt-body';

    // 탭 헤더
    const tabs = document.createElement('div');
    tabs.className = 'dialog-tabs';

    const fontTab = document.createElement('button');
    fontTab.className = 'dialog-tab active';
    fontTab.textContent = i18nText('dialog.options.fontTab.text');
    fontTab.dataset.tab = 'font';
    tabs.appendChild(fontTab);

    const fileTab = document.createElement('button');
    fileTab.className = 'dialog-tab';
    fileTab.textContent = i18nText('dialog.options.fileTab.text');
    fileTab.dataset.tab = 'file';
    tabs.appendChild(fileTab);

    body.appendChild(tabs);

    // 글꼴 탭 패널
    const fontPanel = this.createFontPanel();
    fontPanel.className = 'dialog-tab-panel opt-tab-panel active';
    fontPanel.dataset.tab = 'font';
    body.appendChild(fontPanel);

    const filePanel = this.createFilePanel();
    filePanel.className = 'dialog-tab-panel opt-tab-panel';
    filePanel.dataset.tab = 'file';
    body.appendChild(filePanel);

    // 탭 클릭 이벤트 (향후 탭 추가 대비)
    tabs.addEventListener('click', (e) => {
      const btn = (e.target as HTMLElement).closest('.dialog-tab') as HTMLElement | null;
      if (!btn) return;
      const tabId = btn.dataset.tab;
      tabs.querySelectorAll('.dialog-tab').forEach(t => t.classList.remove('active'));
      body.querySelectorAll('.dialog-tab-panel').forEach(p => p.classList.remove('active'));
      btn.classList.add('active');
      const panel = body.querySelector(`.dialog-tab-panel[data-tab="${tabId}"]`);
      panel?.classList.add('active');
    });

    return body;
  }

  private createFontPanel(): HTMLElement {
    const panel = document.createElement('div');
    const fs = userSettings.getFontSettings();

    // ── 글꼴 보기 섹션 ──
    const viewSection = document.createElement('div');
    viewSection.className = 'dialog-section';

    const viewTitle = document.createElement('div');
    viewTitle.className = 'dialog-section-title';
    viewTitle.textContent = i18nText('dialog.options.viewTitle.text');
    viewSection.appendChild(viewTitle);

    // 최근 사용 글꼴 보이기
    const recentRow = document.createElement('div');
    recentRow.className = 'dialog-row opt-row';

    this.showRecentCheck = document.createElement('input');
    this.showRecentCheck.type = 'checkbox';
    this.showRecentCheck.id = 'opt-show-recent';
    this.showRecentCheck.checked = fs.showRecentFonts;

    const recentLabel = document.createElement('label');
    recentLabel.htmlFor = 'opt-show-recent';
    recentLabel.textContent = i18nText('dialog.options.recentLabel.text');

    this.recentCountInput = document.createElement('input');
    this.recentCountInput.type = 'number';
    this.recentCountInput.className = 'dialog-input opt-count-input';
    this.recentCountInput.min = '1';
    this.recentCountInput.max = '5';
    this.recentCountInput.value = String(fs.recentFontCount);

    const countLabel = document.createElement('span');
    countLabel.className = 'opt-count-label';
    countLabel.textContent = i18nText('dialog.options.countLabel.text');

    recentRow.appendChild(this.showRecentCheck);
    recentRow.appendChild(recentLabel);
    recentRow.appendChild(this.recentCountInput);
    recentRow.appendChild(countLabel);
    viewSection.appendChild(recentRow);

    panel.appendChild(viewSection);

    // ── 대표 글꼴 등록 섹션 ──
    const fontSetSection = document.createElement('div');
    fontSetSection.className = 'dialog-section';

    const fontSetTitle = document.createElement('div');
    fontSetTitle.className = 'dialog-section-title';
    fontSetTitle.textContent = i18nText('dialog.options.fontSetTitle.text');
    fontSetSection.appendChild(fontSetTitle);

    const fontSetDesc = document.createElement('p');
    fontSetDesc.className = 'opt-desc';
    fontSetDesc.textContent = i18nText('dialog.options.fontSetDesc.text');
    fontSetSection.appendChild(fontSetDesc);

    const fontSetBtn = document.createElement('button');
    fontSetBtn.className = 'dialog-btn opt-fontset-btn';
    fontSetBtn.textContent = i18nText('dialog.options.fontSetBtn.text');
    fontSetBtn.addEventListener('click', () => {
      const dlg = new FontSetDialog();
      dlg.show();
    });
    fontSetSection.appendChild(fontSetBtn);

    panel.appendChild(fontSetSection);

    // ── 로컬 글꼴 섹션 ──
    const localSection = document.createElement('div');
    localSection.className = 'dialog-section';

    const localTitle = document.createElement('div');
    localTitle.className = 'dialog-section-title';
    localTitle.textContent = i18nText('dialog.options.localTitle.text');
    localSection.appendChild(localTitle);

    const localDesc = document.createElement('p');
    localDesc.className = 'opt-desc';
    localDesc.textContent = i18nText('dialog.options.localDesc.text');
    localSection.appendChild(localDesc);

    const localRow = document.createElement('div');
    localRow.className = 'dialog-row opt-row opt-local-actions';

    const localBtn = document.createElement('button');
    localBtn.className = 'dialog-btn opt-fontset-btn';
    localBtn.textContent = i18nText('dialog.options.localBtn.text');

    const resetBtn = document.createElement('button');
    resetBtn.className = 'dialog-btn opt-fontset-btn';
    resetBtn.textContent = i18nText('dialog.options.resetBtn.text');

    const localStatus = document.createElement('p');
    localStatus.className = 'opt-local-status';

    const updateLocalStatus = (message?: string): void => {
      const state = getLocalFontState();
      localStatus.textContent = message ?? formatLocalFontStatus(state);
      resetBtn.disabled = !state.stored;
      localBtn.textContent = state.stored ? i18nText('dialog.options.localBtn.text.x827ff8') : i18nText('dialog.options.localBtn.text');
    };

    updateLocalStatus(i18nText('dialog.options.updateLocalStatus.label'));
    void loadStoredLocalFonts().then(
      () => updateLocalStatus(),
      () => updateLocalStatus(i18nText('dialog.options.updateLocalStatus.label.x562451')),
    );

    localBtn.addEventListener('click', async () => {
      if (!isLocalFontAccessSupported()) {
        localStatus.textContent = getLocalFontState().method === 'font-presence-probe'
          ? i18nText('dialog.options.localStatus.text')
          : i18nText('dialog.options.localStatus.text.xc0a701');
        return;
      }
      localBtn.disabled = true;
      resetBtn.disabled = true;
      localStatus.textContent = i18nText('dialog.options.localStatus.text.xfd877c');
      try {
        const fonts = await detectLocalFonts({ force: true });
        updateLocalStatus(i18nText('dialog.options.updateLocalStatus.label.x97c46b', { p1: fonts.length }));
        this.eventBus?.emit('local-fonts-changed', { fonts, source: 'options-dialog' });
      } catch (error) {
        updateLocalStatus(describeLocalFontDetectionError(error));
      }
      localBtn.disabled = false;
    });

    resetBtn.addEventListener('click', async () => {
      localBtn.disabled = true;
      resetBtn.disabled = true;
      localStatus.textContent = i18nText('dialog.options.localStatus.text.x4e6625');
      try {
        await clearStoredLocalFonts();
        updateLocalStatus(i18nText('dialog.options.updateLocalStatus.label.xcbeff3'));
        this.eventBus?.emit('local-fonts-changed', { fonts: [], source: 'options-dialog-clear' });
      } catch {
        updateLocalStatus(i18nText('dialog.options.updateLocalStatus.label.x1ca5cc'));
      }
      localBtn.disabled = false;
    });

    localRow.appendChild(localBtn);
    localRow.appendChild(resetBtn);
    localSection.appendChild(localRow);
    localSection.appendChild(localStatus);

    panel.appendChild(localSection);

    return panel;
  }

  private createFilePanel(): HTMLElement {
    const panel = document.createElement('div');
    const autosave = userSettings.getAutosaveSettings();
    const dialogSettings = userSettings.getDialogSettings();

    const saveSection = document.createElement('div');
    saveSection.className = 'dialog-section';

    const saveTitle = document.createElement('div');
    saveTitle.className = 'dialog-section-title';
    saveTitle.textContent = i18nText('dialog.options.saveTitle.text');
    saveSection.appendChild(saveTitle);

    const desc = document.createElement('p');
    desc.className = 'opt-desc';
    desc.textContent = i18nText('dialog.options.desc.text');
    saveSection.appendChild(desc);

    this.recoveryEnabledCheck = document.createElement('input');
    this.recoveryEnabledCheck.type = 'checkbox';
    this.recoveryEnabledCheck.id = 'opt-recovery-enabled';
    this.recoveryEnabledCheck.checked = autosave.recoveryEnabled;

    this.recoveryIntervalInput = document.createElement('input');
    this.recoveryIntervalInput.type = 'number';
    this.recoveryIntervalInput.className = 'dialog-input opt-interval-input';
    this.recoveryIntervalInput.min = '1';
    this.recoveryIntervalInput.max = '120';
    this.recoveryIntervalInput.value = String(autosave.recoveryIntervalMinutes);

    saveSection.appendChild(createAutosaveNumberRow({
      checkbox: this.recoveryEnabledCheck,
      labelText: '복구용 자동 저장',
      numberInput: this.recoveryIntervalInput,
      unitText: '분',
    }));

    this.idleSaveEnabledCheck = document.createElement('input');
    this.idleSaveEnabledCheck.type = 'checkbox';
    this.idleSaveEnabledCheck.id = 'opt-idle-save-enabled';
    this.idleSaveEnabledCheck.checked = autosave.idleSaveEnabled;

    this.idleDelayInput = document.createElement('input');
    this.idleDelayInput.type = 'number';
    this.idleDelayInput.className = 'dialog-input opt-interval-input';
    this.idleDelayInput.min = '5';
    this.idleDelayInput.max = '600';
    this.idleDelayInput.value = String(autosave.idleDelaySeconds);

    saveSection.appendChild(createAutosaveNumberRow({
      checkbox: this.idleSaveEnabledCheck,
      labelText: '쉴 때 자동 저장',
      numberInput: this.idleDelayInput,
      unitText: '초',
    }));

    const syncDisabled = (): void => {
      this.recoveryIntervalInput.disabled = !this.recoveryEnabledCheck.checked;
      this.idleDelayInput.disabled = !this.idleSaveEnabledCheck.checked;
    };
    this.recoveryEnabledCheck.addEventListener('change', syncDisabled);
    this.idleSaveEnabledCheck.addEventListener('change', syncDisabled);
    syncDisabled();

    panel.appendChild(saveSection);

    const pdfSection = document.createElement('div');
    pdfSection.className = 'dialog-section';

    const pdfTitle = document.createElement('div');
    pdfTitle.className = 'dialog-section-title';
    pdfTitle.textContent = i18nText('dialog.options.pdfTitle.text');
    pdfSection.appendChild(pdfTitle);

    const pdfDesc = document.createElement('p');
    pdfDesc.className = 'opt-desc';
    pdfDesc.textContent =
      i18nText('dialog.options.pdfDesc.text');
    pdfSection.appendChild(pdfDesc);

    const pdfRow = document.createElement('div');
    pdfRow.className = 'dialog-row opt-row';

    this.pdfPrintGuidanceCheck = document.createElement('input');
    this.pdfPrintGuidanceCheck.type = 'checkbox';
    this.pdfPrintGuidanceCheck.id = 'opt-pdf-print-guidance';
    this.pdfPrintGuidanceCheck.checked = dialogSettings.showPdfPrintGuidance;

    const pdfLabel = document.createElement('label');
    pdfLabel.htmlFor = 'opt-pdf-print-guidance';
    pdfLabel.textContent = i18nText('dialog.options.pdfLabel.text');

    pdfRow.append(this.pdfPrintGuidanceCheck, pdfLabel);
    pdfSection.appendChild(pdfRow);
    panel.appendChild(pdfSection);

    return panel;
  }

  protected onConfirm(): void {
    const count = Math.min(5, Math.max(1, parseInt(this.recentCountInput.value) || 3));
    userSettings.updateFontSettings({
      showRecentFonts: this.showRecentCheck.checked,
      recentFontCount: count,
    });
    userSettings.updateAutosaveSettings({
      recoveryEnabled: this.recoveryEnabledCheck.checked,
      recoveryIntervalMinutes: clampInteger(this.recoveryIntervalInput.value, 10, 1, 120),
      idleSaveEnabled: this.idleSaveEnabledCheck.checked,
      idleDelaySeconds: clampInteger(this.idleDelayInput.value, 10, 5, 600),
    });
    userSettings.setShowPdfPrintGuidance(this.pdfPrintGuidanceCheck.checked);
    this.eventBus?.emit('autosave-settings-changed', { source: 'options-dialog' });
  }
}

function createAutosaveNumberRow(options: {
  checkbox: HTMLInputElement;
  labelText: string;
  numberInput: HTMLInputElement;
  unitText: string;
}): HTMLElement {
  const row = document.createElement('div');
  row.className = 'dialog-row opt-row opt-autosave-row';

  const label = document.createElement('label');
  label.htmlFor = options.checkbox.id;
  label.textContent = options.labelText;

  const unit = document.createElement('span');
  unit.className = 'opt-count-label';
  unit.textContent = options.unitText;

  row.appendChild(options.checkbox);
  row.appendChild(label);
  row.appendChild(options.numberInput);
  row.appendChild(unit);
  return row;
}

function clampInteger(value: string, fallback: number, min: number, max: number): number {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed)) return fallback;
  return Math.min(max, Math.max(min, parsed));
}

function formatLocalFontStatus(state: LocalFontState): string {
  if (state.lastError) {
    return `저장소 접근 실패: ${state.lastError}`;
  }
  if (!state.stored) {
    if (state.method === 'font-presence-probe') {
      return '저장된 감지 결과가 없습니다. Firefox에서는 문서를 열 때 필요한 글꼴만 확인합니다.';
    }
    if (!state.supported) {
      return '이 브라우저는 로컬 글꼴 감지를 지원하지 않습니다.';
    }
    return '저장된 감지 결과가 없습니다.';
  }

  const detectedAt = formatDetectedAt(state.detectedAt);
  const dateSuffix = detectedAt ? ` · ${detectedAt}` : '';
  if (state.source === 'font-presence-probe') {
    return `문서별 확인 결과 저장됨: 사용 가능 ${state.count}개 / 확인한 글꼴 ${state.checkedFamilies.length}개${dateSuffix}`;
  }
  if (state.checkedFamilies.length > 0) {
    return `로컬 글꼴 결과 저장됨: 사용 가능 ${state.count}개 / 문서 후보 ${state.checkedFamilies.length}개 / 열거 누락 확인 ${state.probedFamilies.length}개${dateSuffix}`;
  }
  return `로컬 글꼴 열거 결과 저장됨: ${state.count}개 · 문서별 누락 후보는 문서를 열 때 추가 확인${dateSuffix}`;
}

function formatDetectedAt(value: string | null): string {
  if (!value) return '';
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return '';
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${date.getFullYear()}.${pad(date.getMonth() + 1)}.${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

function describeLocalFontDetectionError(error: unknown): string {
  const name = typeof error === 'object' && error !== null && 'name' in error
    ? String((error as { name?: unknown }).name ?? '')
    : '';
  const message = error instanceof Error ? error.message : String(error ?? '');
  const normalized = `${name} ${message}`.toLowerCase();
  if (name === 'NotAllowedError' || normalized.includes('permission') || normalized.includes('denied')) {
    return '로컬 글꼴 접근 권한이 허용되지 않았습니다. 브라우저 권한 설정에서 허용한 뒤 다시 시도해 주세요.';
  }
  return '글꼴 감지에 실패했습니다. 웹 대체 글꼴로 계속 사용할 수 있습니다.';
}
