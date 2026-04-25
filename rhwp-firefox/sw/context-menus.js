import { openViewer } from './viewer-launcher.js';

const MENU_ID = 'rhwp-open-link';

export async function setupContextMenus() {
  try {
    await browser.contextMenus.removeAll();
  } catch {
    // Continue with create. Firefox may reject during extension shutdown/reload.
  }

  browser.contextMenus.create({
    id: MENU_ID,
    title: browser.i18n.getMessage('contextMenuOpen') || 'Open with rhwp',
    contexts: ['link'],
    targetUrlPatterns: [
      '*://*/*.hwp',
      '*://*/*.hwp?*',
      '*://*/*.hwpx',
      '*://*/*.hwpx?*',
    ],
  });

  if (!browser.contextMenus.onClicked.hasListener(handleMenuClick)) {
    browser.contextMenus.onClicked.addListener(handleMenuClick);
  }
}

function handleMenuClick(info) {
  if (info.menuItemId === MENU_ID && info.linkUrl) {
    openViewer({ url: info.linkUrl });
  }
}
