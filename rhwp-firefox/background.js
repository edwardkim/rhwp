import { openViewer } from './sw/viewer-launcher.js';
import { setupContextMenus } from './sw/context-menus.js';
import { setupDownloadObserver } from './sw/download-observer.js';
import { setupMessageRouter } from './sw/message-router.js';

browser.runtime.onInstalled.addListener((details) => {
  setupContextMenus();

  if (details.reason === 'install') {
    browser.storage.sync.set({
      autoOpen: true,
      showBadges: true,
      hoverPreview: true,
    });
  }
});

browser.action.onClicked.addListener(() => {
  openViewer();
});

setupDownloadObserver();
setupMessageRouter();
