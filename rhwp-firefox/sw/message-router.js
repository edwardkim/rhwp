// Content Script ↔ Service Worker 메시지 라우팅
// - Content Script에서 파일 열기 요청
// - 뷰어 탭에서 파일 fetch 요청 (CORS 우회)
// - 향후: 호버 미리보기, 파일 캐싱 등

import { openViewer } from './viewer-launcher.js';
import { extractThumbnailFromUrl } from './thumbnail-extractor.js';
import {
  fetchDocumentWithPolicy,
  isTrustedExtensionPageSender,
  isWebPageSender
} from './fetch-security.js';

/**
 * 메시지 라우터를 설정한다.
 */
export function setupMessageRouter() {
  browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
    const handler = messageHandlers[message.type];
    if (handler) {
      const result = handler(message, sender);
      // async 핸들러 지원
      if (result instanceof Promise) {
        result.then(sendResponse).catch(err => sendResponse({ error: err.message }));
        return true; // 비동기 sendResponse 사용 신호
      }
      sendResponse(result);
    }
  });
}

const messageHandlers = {
  /**
   * Content Script → Service Worker: HWP 파일 열기 요청
   */
  'open-hwp': (message) => {
    openViewer({ url: message.url, filename: message.filename });
    return { ok: true };
  },

  /**
   * 뷰어 탭 → Service Worker: CORS 우회 파일 fetch
   * Service Worker의 fetch는 host_permissions에 의해 CORS 제한 없음
   */
  'fetch-file': async (message, sender) => {
    try {
      if (!isTrustedExtensionPageSender(sender, browser)) {
        return { error: 'Unauthorized sender' };
      }
      const response = await fetchDocumentWithPolicy(message.url);
      if (!response.ok) {
        return { error: `HTTP ${response.status}: ${response.statusText}` };
      }
      const buffer = await response.arrayBuffer();
      // ArrayBuffer는 structured clone으로 전달
      return { data: Array.from(new Uint8Array(buffer)) };
    } catch (err) {
      return { error: err.message };
    }
  },

  /**
   * Content Script → Service Worker: HWP 썸네일 추출
   * Service Worker에서 fetch + CFB PrvImage 추출 (CORS 우회)
   */
  'extract-thumbnail': async (message, sender) => {
    try {
      if (!isWebPageSender(sender)) {
        return { error: 'Unauthorized sender' };
      }
      const result = await extractThumbnailFromUrl(message.url, {
        allowDownloadUrl: message.allowDownloadUrl === true
      });
      return result || { error: 'PrvImage not found' };
    } catch (err) {
      return { error: err.message };
    }
  },

  /**
   * Content Script → Service Worker: 설정 조회
   */
  'get-settings': async () => {
    const settings = await browser.storage.sync.get({
      autoOpen: true,
      showBadges: true,
      hoverPreview: true,
      disableExternalWebFonts: false
    });
    return settings;
  }
};
