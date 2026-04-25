import { openViewer } from './viewer-launcher.js';

const HWP_EXTENSION_RE = /\.(hwp|hwpx)(\?|$)/i;
const HWP_MIME_HINTS = ['haansoft', 'x-hwp', 'hwp+zip'];
const NON_REFETCHABLE_PATTERNS = [
  /\/dext5handler\.[a-z0-9]+/i,
];

export function shouldOpenDownload(item) {
  if (!item) return false;

  const url = item.url || '';
  const referrer = item.referrer || '';
  if (NON_REFETCHABLE_PATTERNS.some((re) => re.test(url) || re.test(referrer))) {
    return false;
  }

  const filename = item.filename || '';
  if (HWP_EXTENSION_RE.test(filename)) return true;
  if (HWP_EXTENSION_RE.test(url)) return true;

  const finalUrl = item.finalUrl || '';
  if (finalUrl !== url && HWP_EXTENSION_RE.test(finalUrl)) return true;

  const mime = (item.mime || '').toLowerCase();
  return HWP_MIME_HINTS.some((hint) => mime.includes(hint));
}

export function setupDownloadObserver() {
  if (!browser.downloads?.onCreated) return;

  browser.downloads.onCreated.addListener((item) => {
    if (shouldOpenDownload(item)) {
      handleHwpDownload(item);
    }
  });
}

async function handleHwpDownload(item) {
  try {
    const settings = await browser.storage.sync.get({ autoOpen: true });
    if (!settings.autoOpen) return;

    openViewer({
      url: item.finalUrl || item.url,
      filename: item.filename,
    });
  } catch (err) {
    console.error('[rhwp] Firefox download observer failed:', err);
  }
}
