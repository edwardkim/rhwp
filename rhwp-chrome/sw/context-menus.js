// 컨텍스트 메뉴 관리
// - HWP/HWPX 링크 우클릭 → "rhwp로 열기"

import { openViewer } from './viewer-launcher.js';

const MENU_ID = 'rhwp-open-link';

/**
 * 컨텍스트 메뉴를 등록한다.
 * chrome.runtime.onInstalled 에서 호출.
 */
export function setupContextMenus() {
  // 기존 메뉴 제거 후 재등록 (업데이트 시 중복 방지)
  chrome.contextMenus.removeAll(() => {
    chrome.contextMenus.create({
      id: MENU_ID,
      title: chrome.i18n.getMessage('contextMenuOpen'),
      contexts: ['link'],
      targetUrlPatterns: [
        '*://*/*.hwp',
        '*://*/*.hwp?*',
        '*://*/*.hwpx',
        '*://*/*.hwpx?*',
        '*://*/*.hml',
        '*://*/*.hml?*'
      ]
    });
  });

  chrome.contextMenus.onClicked.addListener(handleMenuClick);
}

function handleMenuClick(info) {
  if (info.menuItemId === MENU_ID && info.linkUrl) {
    openViewer({ url: info.linkUrl });
  }
}
