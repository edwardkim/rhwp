// rhwp Chrome Extension - Background Service Worker (Entry Point)
// MV3 Service Worker는 비영속적: 유휴 시 종료, 이벤트 시 재시작
// 모든 상태는 chrome.storage로 관리, 전역 변수 사용 금지

import { openViewer } from './sw/viewer-launcher.js';
import { setupContextMenus } from './sw/context-menus.js';
import { setupDownloadInterceptor } from './sw/download-interceptor.js';
import { setupMessageRouter } from './sw/message-router.js';

// 확장 설치/업데이트 시 초기화
chrome.runtime.onInstalled.addListener((details) => {
  setupContextMenus();

  if (details.reason === 'install') {
    // 최초 설치 시 기본 설정 저장
    chrome.storage.sync.set({
      autoOpen: true,
      showBadges: true,
      hoverPreview: true,
      disableExternalWebFonts: false
    });
  }
});

// 확장 아이콘 클릭 → 빈 뷰어 탭 열기
chrome.action.onClicked.addListener(() => {
  openViewer();
});

// 다운로드 가로채기
setupDownloadInterceptor();

// Content Script ↔ Service Worker 메시지 라우팅
setupMessageRouter();
