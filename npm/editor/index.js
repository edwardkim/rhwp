/**
 * @rhwp/editor — HWP 에디터를 iframe으로 임베드
 *
 * 사용법:
 *   import { createEditor } from '@rhwp/editor';
 *   const editor = await createEditor('#container');
 *   await editor.loadFile(buffer, 'document.hwp');
 *
 * 본 제품은 한글과컴퓨터의 한글 문서 파일(.hwp) 공개 문서를 참고하여 개발하였습니다.
 */

import { EditorTransport } from './transport.js';

const DEFAULT_STUDIO_URL = 'https://edwardkim.github.io/rhwp/';

/**
 * HWP 에디터를 생성하여 지정된 컨테이너에 마운트합니다.
 *
 * @param container - CSS 셀렉터 또는 HTMLElement
 * @param options - 에디터 옵션
 * @returns RhwpEditor 인스턴스
 *
 * @example
 * ```javascript
 * const editor = await createEditor('#editor');
 * await editor.loadFile(hwpBuffer, 'sample.hwp');
 * console.log(await editor.pageCount());
 * ```
 */
export async function createEditor(container, options = {}) {
  const el = typeof container === 'string'
    ? document.querySelector(container)
    : container;

  if (!el) {
    throw new Error(`Container not found: ${container}`);
  }

  const studioUrl = options.studioUrl || DEFAULT_STUDIO_URL;

  // iframe 생성
  const iframe = document.createElement('iframe');
  iframe.src = studioUrl;
  iframe.style.width = options.width || '100%';
  iframe.style.height = options.height || '100%';
  iframe.style.border = 'none';
  iframe.allow = 'clipboard-read; clipboard-write';
  el.appendChild(iframe);

  // iframe 로드 대기
  await new Promise((resolve) => {
    iframe.addEventListener('load', resolve, { once: true });
  });

  // WASM 초기화 대기 (ready 메서드로 확인)
  let transport;
  try {
    transport = new EditorTransport(iframe, studioUrl, {
      requestTimeoutMs: options.requestTimeoutMs,
      handshakeTimeoutMs: options.handshakeTimeoutMs,
    });
    await transport.connect();
    const editor = new RhwpEditor(iframe, transport);
    await editor._waitReady();
    return editor;
  } catch (error) {
    transport?.destroy();
    iframe.remove();
    throw error;
  }
}

/**
 * HWP 에디터 인스턴스
 *
 * iframe 내부의 rhwp-studio와 postMessage로 통신합니다.
 */
class RhwpEditor {
  constructor(iframe, transport) {
    this._iframe = iframe;
    this._transport = transport;
  }

  /**
   * iframe에 요청을 보내고 응답을 기다립니다.
   * @internal
   */
  _request(method, params = {}) {
    return this._transport.request(method, params);
  }

  /** WASM 초기화 완료 대기 @internal */
  async _waitReady() {
    for (let i = 0; i < 30; i++) {
      try {
        const result = await this._request('ready');
        if (result) return;
      } catch {
        // 아직 준비 안 됨 — 재시도
      }
      await new Promise((r) => setTimeout(r, 500));
    }
    throw new Error('Editor initialization timeout');
  }

  /**
   * HWP 파일을 로드합니다.
   *
   * @param data - HWP 파일의 ArrayBuffer 또는 Uint8Array
   * @param fileName - 파일 이름 (선택)
   * @returns { pageCount: number }
   *
   * @example
   * ```javascript
   * const resp = await fetch('document.hwp');
   * const buffer = await resp.arrayBuffer();
   * const result = await editor.loadFile(buffer, 'document.hwp');
   * console.log(`${result.pageCount}페이지`);
   * ```
   */
  async loadFile(data, fileName = 'document.hwp') {
    return this._request('loadFile', { data, fileName });
  }

  /**
   * 현재 문서의 페이지 수를 반환합니다.
   * @returns 페이지 수
   */
  async pageCount() {
    return this._request('pageCount');
  }

  /**
   * 특정 페이지를 SVG 문자열로 렌더링합니다.
   * @param page - 0부터 시작하는 페이지 번호
   * @returns SVG 문자열
   */
  async getPageSvg(page = 0) {
    return this._request('getPageSvg', { page });
  }

  /**
   * 현재 문서를 HWP 바이너리로 내보냅니다.
   * @returns {Promise<Uint8Array>} HWP 파일 bytes
   */
  async exportHwp() {
    const result = await this._request('exportHwp');
    return result instanceof Uint8Array ? result : new Uint8Array(result || []);
  }

  /**
   * 현재 문서를 HWPX(ZIP+XML) 바이너리로 내보냅니다.
   * @returns {Promise<Uint8Array>} HWPX 파일 bytes
   */
  async exportHwpx() {
    const result = await this._request('exportHwpx');
    return result instanceof Uint8Array ? result : new Uint8Array(result || []);
  }

  /**
   * HWP 직렬화 + 자기 재로드 검증 메타데이터를 반환합니다 (#178).
   *
   * 검증 메타데이터만 반환하며, 실제 HWP bytes 가 필요하면 `exportHwp()` 를 별도 호출하세요.
   *
   * @returns {Promise<{bytesLen: number, pageCountBefore: number, pageCountAfter: number, recovered: boolean}>}
   */
  async exportHwpVerify() {
    return this._request('exportHwpVerify');
  }

  /**
   * iframe 엘리먼트를 반환합니다.
   */
  get element() {
    return this._iframe;
  }

  /**
   * 에디터를 제거합니다.
   */
  destroy() {
    this._transport.destroy();
    this._iframe.remove();
  }
}
