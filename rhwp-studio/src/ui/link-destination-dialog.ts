import { ModalDialog } from './dialog';
import type { LinkDestination } from '../core/link-navigation';

class LinkDestinationDialog extends ModalDialog {
  constructor(private readonly target: LinkDestination) { super('연결된 문서 또는 앱 열기', 500); }
  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    const help = document.createElement('p');
    help.textContent = this.target.kind === 'registeredApplication'
      ? '이 주소를 처리하는 앱이 기기에 설치되어 있어야 합니다. 연결된 앱에서 열기를 요청할 수 있습니다.'
      : '브라우저는 로컬 경로를 직접 읽을 수 없습니다. 아래 경로에 해당하는 문서를 선택해 여세요. 연결된 호스트에서는 파일 처리기를 등록할 수 있습니다.';
    const uri = document.createElement('textarea'); uri.readOnly = true; uri.value = this.target.uri; uri.rows = 3;
    uri.style.width = '100%'; uri.setAttribute('aria-label', '연결 주소');
    const copy = document.createElement('button'); copy.textContent = '주소 복사'; copy.className = 'dialog-btn';
    copy.onclick = () => { void navigator.clipboard.writeText(this.target.uri).catch(() => { uri.focus(); uri.select(); }); };
    body.append(help, uri, copy); return body;
  }
  protected onConfirm(): boolean {
    if (this.target.kind === 'registeredApplication') window.location.assign(this.target.uri);
    else document.getElementById('file-input')?.click();
    return true;
  }
  override show(): void {
    super.show(); this.dialog.classList.add('dialog-link-destination');
    this.dialog.querySelector('.dialog-btn-primary')!.textContent = this.target.kind === 'registeredApplication' ? '앱에서 열기' : '파일 선택';
  }
}
export function showLinkDestinationDialog(target: LinkDestination): void { new LinkDestinationDialog(target).show(); }
