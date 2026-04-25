import { openViewer } from './viewer-launcher.js';
import { extractThumbnailFromUrl } from './thumbnail-extractor.js';

export function setupMessageRouter() {
  browser.runtime.onMessage.addListener((message, sender) => {
    const handler = messageHandlers[message?.type];
    if (!handler) return undefined;

    return Promise.resolve(handler(message, sender))
      .catch((err) => ({ error: err.message || String(err) }));
  });
}

const messageHandlers = {
  'open-hwp': (message) => {
    openViewer({ url: message.url, filename: message.filename });
    return { ok: true };
  },

  'fetch-file': async (message) => {
    const response = await fetch(message.url);
    if (!response.ok) {
      return { error: `HTTP ${response.status}: ${response.statusText}` };
    }

    const buffer = await response.arrayBuffer();
    return { data: Array.from(new Uint8Array(buffer)) };
  },

  'extract-thumbnail': async (message) => {
    const result = await extractThumbnailFromUrl(message.url);
    return result || { error: 'PrvImage not found' };
  },

  'get-settings': async () => browser.storage.sync.get({
    autoOpen: true,
    showBadges: true,
    hoverPreview: true,
  }),
};
