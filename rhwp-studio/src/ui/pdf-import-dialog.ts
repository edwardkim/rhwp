import { ModalDialog } from './dialog';
import type { PdfImportLayout } from '../core/pdf-import-layout';
import { preparePdfForEditor, type PdfInputSource, type PreparedPdfDocument, type PdfImportPolicy } from '../core/pdf-open';
import type { PdfImportOptions } from '../../../rhwp-shared/pdf/import-session.js';

/** Read options before File.arrayBuffer; cancellation never publishes a candidate. */
class PdfImportDialog extends ModalDialog {
  private profile!: HTMLSelectElement;
  private limits!: HTMLTextAreaElement;
  private timeout!: HTMLInputElement;
  private details!: HTMLPreElement;
  private summary!: HTMLParagraphElement;
  private status!: HTMLParagraphElement;
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
    const body = document.createElement('div');
    const title = document.createElement('p');
    title.textContent = `${this.fileName} · 내용을 편집할 수 있는 문서로 가져옵니다.`;
    const label = (text: string, control: HTMLElement, container: HTMLElement = body) => {
      const row = document.createElement('label'); row.style.display = 'block'; row.textContent = text;
      row.append(control); container.append(row);
    };
    body.append(title);
    this.profile = document.createElement('select'); this.profile.dataset.pdfProfile = '';
    for (const [value, text] of [['standard', '표준'], ['constrained', '자원 절약']]) {
      const option = document.createElement('option'); option.value = value; option.textContent = text; this.profile.append(option);
    }
    label('자원 프로필 ', this.profile);
    this.summary = document.createElement('p'); body.append(this.summary);
    const advanced = document.createElement('details');
    const heading = document.createElement('summary'); heading.textContent = '고급 설정과 유효 한도'; advanced.append(heading); body.append(advanced);
    this.timeout = document.createElement('input'); this.timeout.type = 'number'; this.timeout.min = '1';
    this.timeout.max = '300'; this.timeout.value = '30'; label('제한 시간(초) ', this.timeout, advanced);
    this.limits = document.createElement('textarea'); this.limits.rows = 3; this.limits.maxLength = 8192; this.limits.placeholder = '{}';
    label('개별 한도(JSON, 선택) ', this.limits, advanced);
    this.details = document.createElement('pre'); this.details.style.maxHeight = '160px'; this.details.style.overflow = 'auto';
    this.details.style.whiteSpace = 'pre-wrap'; advanced.append(this.details);
    this.status = document.createElement('p'); this.status.setAttribute('role', 'status'); body.append(this.status);
    this.profile.onchange = this.limits.oninput = () => this.refreshPolicy(); this.refreshPolicy();
    return body;
  }

  private options(): PdfImportOptions {
    if (this.limits.value.length > 8192) throw new Error('PDF_RESOURCE_POLICY: 한도 JSON이 너무 큽니다.');
    return { ...(this.layout === 'editable' ? { layout: this.layout } : {}), resourceProfile: this.profile.value as 'standard' | 'constrained',
      ...(this.limits.value.trim() ? { resourceLimits: JSON.parse(this.limits.value) } : {}) };
  }
  private refreshPolicy(): void {
    try {
      const policy = this.policy(this.options());
      this.summary.textContent = `입력 최대 ${(policy.backend.inputBytes / 1048576).toFixed(0)} MiB · 최대 ${policy.maxSelectedPages}쪽`;
      this.details.textContent = JSON.stringify(policy, null, 2); this.status.textContent = '';
    }
    catch { this.details.textContent = ''; this.status.textContent = '프로필의 상한 이내인 올바른 한도 JSON을 입력하세요.'; }
  }
  protected onConfirm(): boolean {
    if (this.controller) return false;
    let options: PdfImportOptions, policy: PdfImportPolicy;
    const seconds = Number(this.timeout.value);
    try {
      if (!Number.isFinite(seconds) || seconds < 1 || seconds > 300) throw new Error('제한 시간은 1~300초로 입력하세요.');
      options = this.options(); policy = this.policy(options);
    } catch (error) { this.status.textContent = String(error); return false; }
    this.controller = new AbortController();
    this.profile.disabled = this.limits.disabled = this.timeout.disabled = true;
    const button = this.dialog.querySelector<HTMLButtonElement>('.dialog-btn-primary')!;
    button.disabled = true;
    const phases: Record<string, string> = { initializing: '변환기 준비 중…', importing: 'PDF 변환 중…',
      ready: '변환 확인 중…', serializing: '편집 문서 준비 중…' };
    this.status.textContent = '파일 읽는 중…';
    void preparePdfForEditor(this.source, options, policy, { signal: this.controller.signal,
      timeoutMs: seconds * 1000, onProgress: phase => { this.status.textContent = phases[phase] ?? phase; },
    }).then(async result => {
      if (this.settled) return;
      this.status.textContent = '편집 문서 확인 중…';
      await this.onPrepared?.(result, this.controller!.signal);
      return result;
    }).then(result => {
      if (this.settled || !result) return;
      this.settled = true; this.resolve(result); super.hide();
    }, error => {
      if (this.settled) return;
      this.status.textContent = `가져오지 못했습니다. ${String(error)} 설정을 확인한 뒤 다시 시도하세요.`;
      this.controller = null; this.profile.disabled = this.limits.disabled = this.timeout.disabled = false;
      button.disabled = false; button.textContent = '다시 시도';
    });
    return false;
  }
  override hide(): void {
    this.controller?.abort();
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
