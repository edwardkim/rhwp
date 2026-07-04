// rhwp Firefox Extension - Background Script (Entry Point)
// Firefox MV3 Event Page: background.scripts 배열로 로드
// 브라우저 API는 browser.* 네임스페이스 사용 (Firefox 네이티브)
// browser.* 는 Promise를 반환하여 async/await와 자연스럽게 동작

import { openViewer } from './sw/viewer-launcher.js';
import { setupContextMenus } from './sw/context-menus.js';
import { setupDownloadInterceptor } from './sw/download-interceptor.js';
import { setupMessageRouter } from './sw/message-router.js';

// 확장 설치/업데이트 시 초기화
browser.runtime.onInstalled.addListener((details) => {
  setupContextMenus();

  if (details.reason === 'install') {
    // 최초 설치 시 기본 설정 저장 (fire-and-forget, 실패 시 로깅만)
    browser.storage.sync.set({
      autoOpen: true,
      showBadges: true,
      hoverPreview: true,
      disableExternalWebFonts: false
    }).catch((err) => {
      console.error('[rhwp] 초기 설정 저장 오류:', err);
    });
  }
});

// 확장 아이콘 클릭 → 빈 뷰어 탭 열기
browser.action.onClicked.addListener(() => {
  openViewer();
});

// 다운로드 가로채기
setupDownloadInterceptor();

// Content Script ↔ Background 메시지 라우팅
setupMessageRouter();
