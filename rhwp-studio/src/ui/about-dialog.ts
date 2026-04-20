/**
 * 제품 정보 / 라이센스 다이얼로그
 *
 * HWP 공개 스펙(hwp_spec_5.0) 저작권 조항에 따른 필수 고지 문구를 포함한다.
 * 사용된 외부 크레이트의 오픈소스 라이선스 목록도 표시한다.
 */
import { ModalDialog } from './dialog';

/** 외부 크레이트 라이선스 정보 */
const THIRD_PARTY_LICENSES = [
  { name: 'wasm-bindgen', license: 'MIT / Apache-2.0' },
  { name: 'web-sys', license: 'MIT / Apache-2.0' },
  { name: 'js-sys', license: 'MIT / Apache-2.0' },
  { name: 'cfb', license: 'MIT' },
  { name: 'flate2', license: 'MIT / Apache-2.0' },
  { name: 'byteorder', license: 'MIT / Unlicense' },
  { name: 'base64', license: 'MIT / Apache-2.0' },
  { name: 'console_error_panic_hook', license: 'MIT / Apache-2.0' },
];

export class AboutDialog extends ModalDialog {
  constructor() {
    super('제품 정보', 460);
  }

  protected createBody(): HTMLElement {
    const body = document.createElement('div');
    body.className = 'about-body';

    // 제품 영문명
    const titleEn = document.createElement('div');
    titleEn.className = 'about-product-name';
    titleEn.textContent = 'HWP 5.0 Compatible Module for Rust';
    body.appendChild(titleEn);

    // 제품 한글명
    const titleKo = document.createElement('div');
    titleKo.className = 'about-product-name-ko';
    titleKo.textContent = 'HWP 오픈소스 편집';
    body.appendChild(titleKo);

    // 버전
    const version = document.createElement('div');
    version.className = 'about-version';
    version.textContent = `Version ${__APP_VERSION__}`;
    body.appendChild(version);

    // 기술 스택
    const tech = document.createElement('div');
    tech.className = 'about-tech';
    tech.textContent = 'Rust + WebAssembly + TypeScript';
    body.appendChild(tech);

    // HWP 스펙 고지 문구 (필수)
    const notice = document.createElement('div');
    notice.className = 'about-notice';
    notice.textContent =
      '본 제품은 한글과컴퓨터의 한글 문서 파일(.hwp) 공개 문서를 참고하여 개발하였습니다.';
    body.appendChild(notice);

    // 오픈소스 라이선스
    const licenseTitle = document.createElement('div');
    licenseTitle.className = 'about-license-title';
    licenseTitle.textContent = '오픈소스 라이선스';
    body.appendChild(licenseTitle);

    const licenseTable = document.createElement('table');
    licenseTable.className = 'about-license-table';
    for (const lib of THIRD_PARTY_LICENSES) {
      const tr = document.createElement('tr');
      const tdName = document.createElement('td');
      tdName.textContent = lib.name;
      const tdLicense = document.createElement('td');
      tdLicense.textContent = lib.license;
      tr.appendChild(tdName);
      tr.appendChild(tdLicense);
      licenseTable.appendChild(tr);
    }
    body.appendChild(licenseTable);

    // 저작권
    const copyright = document.createElement('div');
    copyright.className = 'about-copyright';
    copyright.textContent = '\u00A9 2026 rhwp: Edward Kim';
    body.appendChild(copyright);

    return body;
  }

  protected onConfirm(): void {
    // 정보 표시 전용 — 확인 동작 없음
  }

  override show(): void {
    super.show();
    // footer를 "닫기" 버튼 하나로 교체
    const footer = this.dialog.querySelector('.dialog-footer');
    if (footer) {
      footer.innerHTML = '';
      const closeBtn = document.createElement('button');
      closeBtn.className = 'dialog-btn dialog-btn-primary';
      closeBtn.textContent = '닫기';
      closeBtn.addEventListener('click', () => this.hide());
      footer.appendChild(closeBtn);
    }
  }
}
