// 컨텍스트 메뉴 관리
// - HWP/HWPX 링크 우클릭 → "rhwp로 열기"

import { openViewer } from './viewer-launcher.js';

const MENU_ID = 'rhwp-open-link';

/**
 * 컨텍스트 메뉴를 등록한다.
 * browser.runtime.onInstalled 에서 호출.
 */
export function setupContextMenus() {
  // 기존 메뉴 제거 후 재등록 (업데이트 시 중복 방지)
  browser.contextMenus.removeAll().then(() => {
    try {
      browser.contextMenus.create({
        id: MENU_ID,
        title: browser.i18n.getMessage('contextMenuOpen'),
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
    } catch (err) {
      console.error('[rhwp] 컨텍스트 메뉴 생성 오류:', err);
    }
  }).catch((err) => {
    console.error('[rhwp] 컨텍스트 메뉴 초기화 오류:', err);
  });

  browser.contextMenus.onClicked.addListener(handleMenuClick);
}

function handleMenuClick(info) {
  if (info.menuItemId === MENU_ID && info.linkUrl) {
    openViewer({ url: info.linkUrl });
  }
}
