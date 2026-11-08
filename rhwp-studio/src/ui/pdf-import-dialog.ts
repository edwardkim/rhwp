import { ModalDialog } from './dialog';
import type { PdfImportLayout } from '../core/pdf-import-layout';
import { preparePdfForEditor, type PdfInputSource, type PreparedPdfDocument, type PdfImportPolicy } from '../core/pdf-open';
import type { PdfImportOptions, PdfPageProgress } from '../../../rhwp-shared/pdf/import-session.js';

/** Read options before File.arrayBuffer; cancellation never publishes a candidate. */
class PdfImportDialog extends ModalDialog {
  private body!: HTMLDivElement;
  private limitActions: HTMLButtonElement[] = [];
  private profile!: HTMLSelectElement;
  private limits!: HTMLTextAreaElement;
  private timeout!: HTMLInputElement;
  private password!: HTMLInputElement;
  private passwordArea!: HTMLDivElement;
  private details!: HTMLPreElement;
  private summary!: HTMLParagraphElement;
  private status!: HTMLParagraphElement;
  private error!: HTMLParagraphElement;
  private progress!: HTMLProgressElement;
  private policySummary!: HTMLDListElement;
  private controller: AbortController | null = null;
  private settled = false;
  private resolve!: (value: PreparedPdfDocument) => void;
  private reject!: (error: unknown) => void;

  constructor(private readonly source: PdfInputSource, private readonly fileName: string,
    private readonly policy: (options: PdfImportOptions) => PdfImportPolicy,
    private readonly onPrepared?: (result: PreparedPdfDocument, signal: AbortSignal) => Promise<void>,
    private readonly layout: PdfImportLayout = 'fidelity') {
    super('PDF 가져오기', 520);
  }

  protected createBody(): HTMLElement {
    const body = this.body = document.createElement('div');
    body.className = 'dialog-pdf-import-body';
    const file = document.createElement('div'); file.className = 'dialog-pdf-import-file';
    const name = document.createElement('strong'); name.textContent = this.fileName;
    const description = document.createElement('p');
    description.textContent = 'PDF 내용을 편집할 수 있는 문서로 가져옵니다.';
    file.append(name, description); body.append(file);

    const field = (text: string, control: HTMLElement, id: string, container: HTMLElement) => {
      const row = document.createElement('div'); row.className = 'dialog-pdf-import-field';
      const label = document.createElement('label'); label.textContent = text; label.htmlFor = id;
      control.id = id; row.append(label, control); container.append(row);
    };
    this.profile = document.createElement('select'); this.profile.dataset.pdfProfile = '';
    this.profile.className = 'dialog-select';
    for (const [value, text] of [['standard', '표준 (권장)'], ['constrained', '자원 절약']]) {
      const option = document.createElement('option'); option.value = value; option.textContent = text; this.profile.append(option);
    }
    field('처리 설정', this.profile, 'pdf-import-profile', body);
    this.summary = document.createElement('p'); this.summary.id = 'pdf-import-profile-help';
    this.summary.className = 'dialog-pdf-import-help'; body.append(this.summary);
    this.profile.setAttribute('aria-describedby', this.summary.id);

    const advanced = document.createElement('details'); advanced.className = 'dialog-pdf-import-advanced';
    const heading = document.createElement('summary'); heading.textContent = '고급 설정';
    const settings = document.createElement('div'); settings.className = 'dialog-pdf-import-settings';
    advanced.append(heading, settings); body.append(advanced);
    this.passwordArea = document.createElement('div'); this.passwordArea.hidden = true;
    this.password = document.createElement('input'); this.password.type = 'password';
    this.password.autocomplete = 'off'; this.password.maxLength = 1024;
    this.password.className = 'dialog-input'; this.password.oninput = () => this.clearError();
    field('PDF 열기 암호', this.password, 'pdf-import-password', this.passwordArea);
    body.append(this.passwordArea);
    this.timeout = document.createElement('input'); this.timeout.type = 'number'; this.timeout.min = '1';
    this.timeout.max = '300'; this.timeout.value = '30'; this.timeout.className = 'dialog-input';
    field('제한 시간 (초)', this.timeout, 'pdf-import-timeout', settings);
    const timeHelp = document.createElement('p'); timeHelp.id = 'pdf-import-timeout-help';
    timeHelp.className = 'dialog-pdf-import-help';
    timeHelp.textContent = '1~300초. 시간이 지나면 가져오기를 중단하며, 설정을 바꿔 다시 시도할 수 있습니다.';
    settings.append(timeHelp); this.timeout.setAttribute('aria-describedby', timeHelp.id);
    const policyTitle = document.createElement('h3'); policyTitle.textContent = '적용되는 한도';
    this.policySummary = document.createElement('dl'); this.policySummary.className = 'dialog-pdf-import-policy';
    settings.append(policyTitle, this.policySummary);

    const expert = document.createElement('details'); expert.className = 'dialog-pdf-import-expert';
    const expertHeading = document.createElement('summary'); expertHeading.textContent = '전문가 설정 (JSON)';
    const expertBody = document.createElement('div'); expertBody.className = 'dialog-pdf-import-settings';
    expert.append(expertHeading, expertBody); settings.append(expert);
    const limitsHelp = document.createElement('p'); limitsHelp.id = 'pdf-import-limits-help';
    limitsHelp.className = 'dialog-pdf-import-help';
    limitsHelp.textContent = '대부분은 입력하지 않아도 됩니다. 비워 두면 선택한 처리 설정의 기본 한도를 사용합니다. 값을 지정하면 해당 한도만 낮춥니다.';
    this.limits = document.createElement('textarea'); this.limits.rows = 4; this.limits.maxLength = 8192;
    this.limits.placeholder = '{}'; this.limits.className = 'dialog-pdf-import-json'; this.limits.spellcheck = false;
    this.limits.setAttribute('aria-describedby', limitsHelp.id);
    field('맞춤 한도 (JSON, 선택)', this.limits, 'pdf-import-limits', expertBody); expertBody.append(limitsHelp);
    const help = document.createElement('p'); help.className = 'dialog-pdf-import-help';
    help.textContent = 'maxSelectedPages는 가져올 최대 쪽수, backend.inputBytes는 최대 파일 크기(바이트)입니다. 1 MiB는 1,048,576바이트입니다. 한도를 넘으면 중단되며, 앞의 몇 쪽만 가져오는 기능은 아닙니다.';
    const example = { maxSelectedPages: 8, backend: { inputBytes: 8388608 } };
    const exampleText = document.createElement('p'); exampleText.className = 'dialog-pdf-import-help';
    exampleText.textContent = '예: 최대 8쪽, 파일 크기 8 MiB. 나머지 한도는 선택한 처리 설정을 따릅니다.';
    const exampleCode = document.createElement('code'); exampleCode.className = 'dialog-pdf-import-example';
    exampleCode.textContent = JSON.stringify(example);
    const actions = document.createElement('div'); actions.className = 'dialog-pdf-import-actions';
    for (const [label, value] of [['예시 넣기', JSON.stringify(example, null, 2)], ['기본 한도 사용', '']]) {
      const action = document.createElement('button'); action.type = 'button'; action.className = 'dialog-btn';
      action.textContent = label;
      action.onclick = () => { this.limits.value = value; this.refreshPolicy(); this.limits.focus(); };
      this.limitActions.push(action); actions.append(action);
    }
    expertBody.append(help, exampleText, exampleCode, actions);
    const detailsLabel = document.createElement('h3'); detailsLabel.textContent = '전체 적용 한도 (읽기 전용)';
    this.details = document.createElement('pre'); this.details.className = 'dialog-pdf-import-json';
    this.details.tabIndex = 0; this.details.setAttribute('aria-label', '전체 적용 한도');
    expertBody.append(detailsLabel, this.details);

    this.status = document.createElement('p'); this.status.setAttribute('role', 'status'); this.status.hidden = true;
    this.progress = document.createElement('progress'); this.progress.hidden = true;
    this.progress.max = 100; this.progress.value = 0;
    this.progress.setAttribute('aria-label', 'PDF 가져오기 진행 중');
    this.error = document.createElement('p'); this.error.id = 'pdf-import-error';
    this.error.setAttribute('role', 'alert'); this.error.hidden = true;
    this.error.className = 'dialog-pdf-import-error';
    body.append(this.progress, this.status, this.error);
    this.profile.onchange = this.limits.oninput = () => this.refreshPolicy();
    this.timeout.oninput = () => this.clearError();
    this.refreshPolicy();
    return body;
  }

  private clearError(): void {
    this.error.hidden = true; this.error.textContent = '';
    for (const control of [this.limits, this.timeout, this.password]) {
      control.removeAttribute('aria-invalid'); control.removeAttribute('aria-errormessage');
    }
  }

  private showError(message: string, control?: HTMLElement, focus = false): void {
    (control?.parentElement ?? this.body).append(this.error);
    this.error.textContent = message; this.error.hidden = false;
    if (control) {
      control.setAttribute('aria-invalid', 'true'); control.setAttribute('aria-errormessage', this.error.id);
      if (focus) {
        for (let parent = control.parentElement; parent; parent = parent.parentElement) {
          if (parent instanceof HTMLDetailsElement) parent.open = true;
        }
        control.focus();
      }
    }
    if (this.dialog.isConnected) this.error.scrollIntoView({ block: 'nearest' });
  }

  private showStatus(text: string): void {
    this.status.textContent = text; this.status.hidden = !text;
    if (text && this.dialog.isConnected) this.status.scrollIntoView({ block: 'nearest' });
  }

  protected override onTabKey(event: KeyboardEvent): boolean {
    const controls = [...this.dialog.querySelectorAll<HTMLElement>(
      'button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), summary, [tabindex="0"]',
    )].filter(control => control.checkVisibility({ checkVisibilityCSS: true }));
    if (!controls.length) return false;
    const index = controls.indexOf(document.activeElement as HTMLElement);
    const next = index < 0 ? (event.shiftKey ? controls.length - 1 : 0)
      : (index + (event.shiftKey ? -1 : 1) + controls.length) % controls.length;
    controls[next].focus();
    return true;
  }

  private options(): PdfImportOptions {
    if (this.limits.value.length > 8192) throw new Error('PDF_RESOURCE_POLICY: 한도 JSON이 너무 큽니다.');
    return { ...(this.password.value ? { password: this.password.value } : {}), ...(this.layout === 'editable' ? { layout: this.layout } : {}), resourceProfile: this.profile.value as 'standard' | 'constrained',
      ...(this.limits.value.trim() ? { resourceLimits: JSON.parse(this.limits.value) } : {}) };
  }
  private refreshPolicy(): void {
    this.clearError();
    try {
      const policy = this.policy(this.options());
      const bytes = policy.backend.inputBytes;
      const unit = bytes >= 1048576 ? 'MiB' : bytes >= 1024 ? 'KiB' : 'B';
      const size = `${Number((bytes / (unit === 'MiB' ? 1048576 : unit === 'KiB' ? 1024 : 1)).toFixed(2)).toLocaleString('ko-KR')} ${unit}`;
      const pages = Number(policy.maxSelectedPages).toLocaleString('ko-KR');
      this.summary.textContent = `${this.profile.value === 'constrained' ? '자원 사용을 제한합니다.' : '기본 한도로 가져옵니다.'} 최대 ${size} · ${pages}쪽`;
      this.policySummary.replaceChildren();
      for (const [name, value] of [['파일 크기', size], ['가져올 쪽수', `${pages}쪽`]]) {
        const term = document.createElement('dt'); term.textContent = name;
        const detail = document.createElement('dd'); detail.textContent = value;
        this.policySummary.append(term, detail);
      }
      this.details.textContent = JSON.stringify(policy, null, 2);
    } catch {
      this.details.textContent = ''; this.policySummary.replaceChildren();
      this.summary.textContent = '맞춤 한도를 확인하세요. 현재 설정을 적용할 수 없습니다.';
      this.showError('맞춤 한도에 올바른 JSON을 입력하세요. 값은 선택한 처리 설정의 상한을 넘을 수 없습니다.', this.limits);
    }
  }
  protected override shouldConfirmOnEnter(target: HTMLElement): boolean {
    return target === this.password;
  }

  protected onConfirm(): boolean {
    if (this.controller) return false;
    this.clearError();
    let options: PdfImportOptions, policy: PdfImportPolicy;
    if (new TextEncoder().encode(this.password.value).length > 1024) {
      this.passwordArea.hidden = false;
      this.showError('암호는 UTF-8 기준 1,024바이트 이하로 입력하세요.', this.password, true);
      return false;
    }
    const seconds = Number(this.timeout.value);
    if (!Number.isFinite(seconds) || seconds < 1 || seconds > 300) {
      this.showError('제한 시간은 1~300초로 입력하세요.', this.timeout, true); return false;
    }
    try { options = this.options(); policy = this.policy(options); }
    catch {
      this.showError('맞춤 한도에 올바른 JSON을 입력하세요. 값은 선택한 처리 설정의 상한을 넘을 수 없습니다.', this.limits, true);
      return false;
    }
    this.controller = new AbortController();
    this.password.disabled = this.profile.disabled = this.limits.disabled = this.timeout.disabled = true;
    for (const action of this.limitActions) action.disabled = true;
    const button = this.dialog.querySelector<HTMLButtonElement>('.dialog-btn-primary')!;
    button.disabled = true;
    const phases: Record<string, string> = { initializing: '변환기 준비 중…', importing: 'PDF 변환 중…',
      ready: '변환 확인 중…', serializing: '편집 문서 준비 중…' };
    this.progress.hidden = false; this.progress.value = 0;
    this.progress.removeAttribute('aria-valuetext');
    let pageProgress: PdfPageProgress | undefined;
    this.showStatus('파일 읽는 중… · 0%');
    void preparePdfForEditor(this.source, options, policy, { signal: this.controller.signal,
      timeoutMs: seconds * 1000, onProgress: (phase, update) => {
        if (this.settled || this.controller?.signal.aborted) return;
        if (update) pageProgress = update;
        if (pageProgress) {
          const p = pageProgress;
          const percent = Math.floor(p.completedPages * 100 / p.totalPages);
          this.progress.value = percent;
          const selection = p.totalPages !== p.sourcePageCount
            ? ` (선택 ${p.completedPages}/${p.totalPages}쪽 완료)` : ` (${p.completedPages}쪽 완료)`;
          const message = phase === 'importing'
            ? `${p.sourcePage} / ${p.sourcePageCount}쪽 ${p.completedPages === p.totalPages ? '변환 완료' : '처리 중'} · ${percent}%${selection}`
            : `페이지 변환 ${p.totalPages}/${p.totalPages}쪽 완료 · 100% — ${phases[phase] ?? phase}`;
          this.progress.setAttribute('aria-valuetext', message);
          this.showStatus(message);
        } else {
          this.showStatus(`${phases[phase] ?? phase} · 페이지 수 확인 중 · 0%`);
        }
      },
    }).then(async result => {
      if (this.settled) return;
      this.progress.value = 100;
      const message = `페이지 변환 완료 · ${result.pageCount}쪽 · 100% — 편집 문서 확인 중…`;
      this.progress.setAttribute('aria-valuetext', message); this.showStatus(message);
      await this.onPrepared?.(result, this.controller!.signal);
      return result;
    }).then(result => {
      if (this.settled || !result) return;
      this.password.value = ''; this.settled = true; this.resolve(result); super.hide();
    }, error => {
      if (this.settled) return;
      this.progress.hidden = true; this.showStatus('');
      this.controller = null; this.password.disabled = this.profile.disabled = this.limits.disabled = this.timeout.disabled = false;
      const message = String(error);
      if (/PDF_PASSWORD_(REQUIRED|INVALID)/.test(message)) {
        this.passwordArea.hidden = false; this.password.value = '';
        this.showError(message.includes('PDF_PASSWORD_INVALID')
          ? 'PDF 열기 암호가 올바르지 않습니다. 다시 입력하세요.'
          : '암호가 설정된 PDF입니다. 열기 암호를 입력하세요.', this.password, true);
      } else {
        this.password.value = '';
        this.showError(`가져오지 못했습니다. ${message}`);
      }
      for (const action of this.limitActions) action.disabled = false;
      button.disabled = false; button.textContent = '다시 시도';
    });
    return false;
  }
  override hide(): void {
    this.controller?.abort();
    if (this.password) this.password.value = '';
    if (!this.settled) {
      this.settled = true;
      this.reject(Object.assign(new Error('PDF_IMPORT_CANCELLED'), { code: 'PDF_IMPORT_CANCELLED' }));
    }
    super.hide();
  }
  showAsync(): Promise<PreparedPdfDocument> {
    return new Promise((resolve, reject) => {
      this.resolve = resolve; this.reject = reject; super.show();
      this.dialog.classList.add('dialog-pdf-import');
      this.dialog.querySelector('.dialog-btn-primary')!.textContent = '가져오기';
    });
  }
}

let active: PdfImportDialog | null = null;
export function cancelPdfImport(): void { active?.hide(); active = null; }
export async function showPdfImportDialog(source: PdfInputSource, fileName: string,
  policy: (options: PdfImportOptions) => PdfImportPolicy,
  onPrepared?: (result: PreparedPdfDocument, signal: AbortSignal) => Promise<void>,
  layout: PdfImportLayout = 'fidelity'): Promise<PreparedPdfDocument> {
  cancelPdfImport();
  const dialog = new PdfImportDialog(source, fileName, policy, onPrepared, layout); active = dialog;
  try { return await dialog.showAsync(); }
  finally { if (active === dialog) active = null; }
}
