// 뷰어 탭 관리
// - 뷰어 탭 열기/재활용
// - URL 파라미터로 파일 경로 전달

import { resolveDocumentUrl } from './document-url-resolver.js';

function buildViewerUrl(viewerBase, options = {}) {
  const params = new URLSearchParams();

  if (options.url) params.set('url', resolveDocumentUrl(options.url));
  if (options.filename) params.set('filename', options.filename);

  const query = params.toString();
  return query ? `${viewerBase}?${query}` : viewerBase;
}

/**
 * 뷰어 탭을 열어 HWP 파일을 표시한다.
 * @param {object} [options]
 * @param {string} [options.url] - HWP 파일 URL
 * @param {string} [options.filename] - 표시용 파일명
 */
export function openViewer(options = {}) {
  const viewerBase = chrome.runtime.getURL('viewer.html');
  const fullUrl = buildViewerUrl(viewerBase, options);

  chrome.tabs.create({ url: fullUrl });
}

/**
 * 기존 빈 뷰어 탭이 있으면 재활용, 없으면 새 탭 생성.
 * @param {object} options
 */
export async function openViewerOrReuse(options = {}) {
  const viewerBase = chrome.runtime.getURL('viewer.html');

  // 빈 뷰어 탭 검색
  const tabs = await chrome.tabs.query({ url: `${viewerBase}*` });
  const emptyTab = tabs.find(t => t.url === viewerBase);

  if (emptyTab) {
    // 기존 빈 탭에 파일 로드
    await chrome.tabs.update(emptyTab.id, {
      url: buildViewerUrl(viewerBase, options),
      active: true
    });
  } else {
    openViewer(options);
  }
}
