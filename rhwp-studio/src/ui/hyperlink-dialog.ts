import { webHyperlinkUrl } from '@/core/hyperlink';
import { ModalDialog } from './dialog';
import './hyperlink-dialog.css';

export type HyperlinkEdit = { kind: 'save'; uri: string; text: string } | { kind: 'remove' };

export class HyperlinkDialog extends ModalDialog {
  private textInput!: HTMLInputElement;
  private uriInput!: HTMLInputElement;
  private error!: HTMLParagraphElement;
  private previewButton!: HTMLButtonElement;

  constructor(
    private initial: { text: string; uri: string; existing: boolean; canInsertText: boolean },
    private apply: (edit: HyperlinkEdit) => void,
  ) { super(initial.existing ? '하이퍼링크 고치기' : '하이퍼링크', 560); }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'dialog-hyperlink-body';
    const input = (label: string, id: string): HTMLInputElement => {
      const row = document.createElement('div');
      row.className = 'dialog-row';
      const name = document.createElement('label');
      name.className = 'dialog-label';
      name.htmlFor = id;
      name.textContent = label;
      const field = document.createElement('input');
      field.className = 'dialog-input';
      field.id = id;
      field.type = 'text';
      row.append(name, field);
      body.append(row);
      return field;
    };
    this.textInput = input('표시할 문자열', 'hyperlink-text');
    this.textInput.value = this.initial.text;
    this.textInput.readOnly = !this.initial.canInsertText;
    if (this.textInput.readOnly) this.textInput.title = '표시할 문자열은 문서에서 직접 편집할 수 있습니다.';
    const targets = document.createElement('fieldset');
    targets.className = 'dialog-hyperlink-target';
    const legend = document.createElement('legend');
    legend.textContent = '연결 대상';
    targets.append(legend);
    const tabs = document.createElement('div');
    tabs.className = 'dialog-tabs';
    tabs.setAttribute('role', 'tablist');
    tabs.setAttribute('aria-label', '연결 대상');
    const webTab = document.createElement('button');
    webTab.type = 'button';
    webTab.id = 'hyperlink-web-tab';
    webTab.className = 'dialog-tab active';
    webTab.textContent = '웹 주소';
    webTab.setAttribute('role', 'tab');
    webTab.setAttribute('aria-selected', 'true');
    webTab.setAttribute('aria-controls', 'hyperlink-web-panel');
    tabs.append(webTab);
    targets.append(tabs);
    const panel = document.createElement('div');
    panel.className = 'dialog-section dialog-hyperlink-panel';
    panel.id = 'hyperlink-web-panel';
    panel.setAttribute('role', 'tabpanel');
    panel.setAttribute('aria-labelledby', webTab.id);
    const webLabel = document.createElement('label');
    webLabel.htmlFor = 'hyperlink-uri';
    webLabel.textContent = '웹 주소';
    panel.append(webLabel);
    this.uriInput = document.createElement('input');
    this.uriInput.className = 'dialog-input';
    this.uriInput.id = 'hyperlink-uri';
    this.uriInput.type = 'url';
    this.uriInput.setAttribute('aria-label', '웹 주소');
    this.uriInput.placeholder = 'https://example.com';
    this.uriInput.value = this.initial.uri;
    const addressRow = document.createElement('div');
    addressRow.className = 'dialog-row dialog-hyperlink-address';
    this.previewButton = document.createElement('button');
    this.previewButton.type = 'button';
    this.previewButton.id = 'hyperlink-preview';
    this.previewButton.className = 'dialog-btn dialog-hyperlink-preview';
    this.previewButton.title = '웹 주소 새 탭에서 열어보기';
    this.previewButton.setAttribute('aria-label', '웹 주소 새 탭에서 열어보기');
    // 텍스트 이모지 대신 currentColor SVG로 테마와 플랫폼에 관계없이 같은 지구본을 표시한다.
    this.previewButton.innerHTML = '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.6" aria-hidden="true"><circle cx="12" cy="12" r="9"/><ellipse cx="12" cy="12" rx="4" ry="9"/><path d="M3 12h18M5 7h14M5 17h14"/></svg>';
    this.previewButton.addEventListener('click', () => {
      try {
        const url = webHyperlinkUrl(this.uriInput.value);
        if (!url) throw new Error('http:// 또는 https://로 시작하는 웹 주소를 입력해 주세요.');
        const opened = window.open('about:blank', '_blank');
        if (!opened) throw new Error('팝업이 차단되었습니다. 이 사이트의 팝업을 허용해 주세요.');
        opened.opener = null;
        opened.location.replace(url);
        this.error.textContent = '';
      } catch (error) {
        this.error.textContent = error instanceof Error ? error.message : String(error);
      }
    });
    addressRow.append(this.uriInput, this.previewButton);
    panel.append(addressRow);
    targets.append(panel);
    body.append(targets);
    this.error = document.createElement('p');
    this.error.setAttribute('role', 'alert');
    this.error.id = 'hyperlink-error';
    this.uriInput.setAttribute('aria-describedby', this.error.id);
    body.append(this.error);
    return body;
  }

  protected onConfirm(): boolean {
    try {
      this.apply({ kind: 'save', uri: this.uriInput.value.trim(), text: this.textInput.value });
      return true;
    } catch (error) {
      this.error.textContent = error instanceof Error ? error.message : String(error);
      return false;
    }
  }

  override show(): void {
    super.show();
    this.dialog.classList.add('dialog-hyperlink');
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.setAttribute('aria-label', this.initial.existing ? '하이퍼링크 고치기' : '하이퍼링크');
    const confirm = this.dialog.querySelector<HTMLButtonElement>('.dialog-btn-primary')!;
    confirm.textContent = this.initial.existing ? '고치기' : '넣기';
    const update = () => {
      confirm.disabled = !this.uriInput.value.trim() || !this.textInput.value.trim();
      this.previewButton.disabled = webHyperlinkUrl(this.uriInput.value) === null;
    };
    this.uriInput.addEventListener('input', update);
    this.textInput.addEventListener('input', update);
    update();
    this.uriInput.focus();
    this.uriInput.select();
  }
}

class ExistingHyperlinkDialog extends ModalDialog {
  private accepted = false;
  constructor(private edit: () => void, private cancel: () => void) { super('하이퍼링크', 390); }
  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'dialog-hyperlink-confirm-body';
    body.textContent = '하이퍼링크가 이미 입력되어 있습니다.\n하이퍼링크를 고칠까요?';
    return body;
  }
  protected onConfirm(): void { this.accepted = true; }
  override show(): void {
    super.show();
    this.dialog.setAttribute('role', 'alertdialog');
    this.dialog.setAttribute('aria-label', '하이퍼링크');
    this.dialog.querySelector('.dialog-btn-primary')!.textContent = '고침';
  }
  override hide(): void {
    super.hide();
    if (this.accepted) this.edit();
    else this.cancel();
  }
}

export function confirmHyperlinkEdit(edit: () => void, cancel: () => void): void {
  new ExistingHyperlinkDialog(edit, cancel).show();
}
