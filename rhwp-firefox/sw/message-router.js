// Content Script ↔ Service Worker 메시지 라우팅
// - Content Script에서 파일 열기 요청
// - 뷰어 탭에서 파일 fetch 요청 (CORS 우회)
// - 향후: 호버 미리보기, 파일 캐싱 등

import { openViewer } from './viewer-launcher.js';
import { extractThumbnailFromUrl } from './thumbnail-extractor.js';
import { fetchDocumentWithPolicy, isTrustedExtensionPageSender, isWebPageSender } from './fetch-security.js';

export function setupMessageRouter() {
  browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
    const handler = messageHandlers[message.type];
    if (handler) {
      const result = handler(message, sender);
      if (result instanceof Promise) {
        result.then(sendResponse).catch(err => sendResponse({ error: err.message }));
        return true;
      }
      sendResponse(result);
    }
  });
}

const DEFAULTS = { autoOpen: true, showBadges: true, hoverPreview: true, disableExternalWebFonts: false };

function isAllDefaults(s) {
  return s.autoOpen === DEFAULTS.autoOpen
    && s.showBadges === DEFAULTS.showBadges
    && s.hoverPreview === DEFAULTS.hoverPreview
    && s.disableExternalWebFonts === DEFAULTS.disableExternalWebFonts;
}

const messageHandlers = {
  'open-hwp': (message) => { openViewer({ url: message.url, filename: message.filename }); return { ok: true }; },
  'fetch-file': async (message, sender) => {
    try {
      if (!isTrustedExtensionPageSender(sender, browser)) return { error: 'Unauthorized sender' };
      const response = await fetchDocumentWithPolicy(message.url);
      if (!response.ok) return { error: `HTTP ${response.status}: ${response.statusText}` };
      const buffer = await response.arrayBuffer();
      return { data: Array.from(new Uint8Array(buffer)) };
    } catch (err) { return { error: err.message }; }
  },
  'extract-thumbnail': async (message, sender) => {
    try {
      if (!isWebPageSender(sender)) return { error: 'Unauthorized sender' };
      const result = await extractThumbnailFromUrl(message.url, { allowDownloadUrl: message.allowDownloadUrl === true });
      return result || { error: 'PrvImage not found' };
    } catch (err) { return { error: err.message }; }
  },
  'get-settings': async () => {
    try {
      const syncSettings = await browser.storage.sync.get(DEFAULTS);
      if (!isAllDefaults(syncSettings)) return syncSettings;
    } catch (err) { console.error('[rhwp] Sync 설정 읽기 오류:', err); }
    try {
      const localSettings = await browser.storage.local.get(DEFAULTS);
      if (!isAllDefaults(localSettings)) return localSettings;
    } catch (err) { console.error('[rhwp] Local 설정 읽기 오류:', err); }
    return { ...DEFAULTS };
  }
};
