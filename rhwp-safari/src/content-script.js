// rhwp Safari Web Extension - Content Script
// Safari Web Extension 표준 browser.* API 사용
// 웹페이지에서 HWP/HWPX 링크를 감지하고 뱃지 + 호버 카드 삽입

(() => {
  'use strict';

  const HWP_EXTENSIONS = /\.(hwp|hwpx)(\?.*)?$/i;
  const DOCUMENT_PATH_EXTENSIONS = /\.(hwp|hwpx)$/i;
  const GITHUB_NON_DOCUMENT_MARKERS = new Set(['edit', 'commits', 'blame', 'tree']);
  const BADGE_CLASS = 'rhwp-badge';
  const HOVER_CLASS = 'rhwp-hover-card';
  const PROCESSED_ATTR = 'data-rhwp-processed';

  let settings = { autoOpen: true, showBadges: true, hoverPreview: true };
  const thumbnailCache = new Map(); // URL → { dataUri, width, height } | null

  // 설정 로드
  browser.runtime.sendMessage({ type: 'get-settings' }).then((result) => {
    if (result) settings = { ...settings, ...result };
    init();
  }).catch(() => {
    init();
  });

  function init() {
    const shouldProcess = settings.showBadges || settings.hoverPreview || settings.autoOpen;
    if (shouldProcess) {
      processLinks();
      observeDynamicContent();
    }
  }

  // 확장 존재 알림 (N-03: 허용 도메인에서만 노출, 버전 정보 제거)
  const ALLOWED_ANNOUNCE_DOMAINS = ['.go.kr', '.or.kr', '.ac.kr', '.mil.kr', '.korea.kr'];
  const shouldAnnounce = ALLOWED_ANNOUNCE_DOMAINS.some(d => location.hostname.endsWith(d));
  if (shouldAnnounce) {
    document.documentElement.setAttribute('data-hwp-extension', 'rhwp');
    window.dispatchEvent(new CustomEvent('hwp-extension-ready', {
      detail: { name: 'rhwp', capabilities: ['preview'] }
    }));
  }

  // ─── 유틸리티 ───

  function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
  }

  function extractFilename(anchor) {
    // URL에서 파일명 추출
    try {
      const pathname = new URL(anchor.href).pathname;
      const name = decodeURIComponent(pathname.split('/').pop() || '');
      if (HWP_EXTENSIONS.test(name)) return name;
    } catch { /* ignore */ }
    // 링크 텍스트 폴백
    const text = anchor.textContent.trim();
    return text || anchor.href;
  }

  function formatSize(bytes) {
    if (bytes < 1024) return `${bytes}B`;
    if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)}KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)}MB`;
  }

  // ─── 링크 감지 ───

  function hasDocumentPath(pathname) {
    if (typeof pathname !== 'string') return false;
    try {
      return DOCUMENT_PATH_EXTENSIONS.test(decodeURIComponent(pathname).toLowerCase());
    } catch {
      return DOCUMENT_PATH_EXTENSIONS.test(pathname.toLowerCase());
    }
  }

  function classifyDocumentHref(href) {
    let parsed;
    try {
      parsed = new URL(href);
    } catch {
      return { status: 'unknown' };
    }

    if (parsed.protocol !== 'https:' && parsed.protocol !== 'http:') {
      return { status: 'unknown' };
    }

    if (parsed.hostname === 'raw.githubusercontent.com') {
      return hasDocumentPath(parsed.pathname)
        ? { status: 'openable' }
        : { status: 'not-document' };
    }

    if (parsed.hostname === 'github.com') {
      const segments = parsed.pathname.split('/').filter(Boolean);
      if (segments.length >= 3) {
        const marker = segments[2];
        if (marker === 'blob') {
          const pathParts = segments.slice(4);
          return pathParts.length > 0 && hasDocumentPath(pathParts.join('/'))
            ? { status: 'openable' }
            : { status: 'not-document' };
        }
        if (GITHUB_NON_DOCUMENT_MARKERS.has(marker)) {
          return { status: 'not-document' };
        }
      }
    }

    return hasDocumentPath(parsed.pathname)
      ? { status: 'openable' }
      : { status: 'unknown' };
  }

  function isHwpLink(anchor) {
    if (!anchor.href) return false;
    if (anchor.getAttribute('data-hwp') === 'true') return true;
    return classifyDocumentHref(anchor.href).status === 'openable';
  }

  function createBadge(anchor) {
    const badge = document.createElement('span');
    badge.className = BADGE_CLASS;
    badge.title = browser.i18n.getMessage('badgeTooltip') || 'rhwp로 열기';

    badge.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();
      openHwpViewer(anchor.href, extractFilename(anchor));
    });

    return badge;
  }

  // ─── 호버 미리보기 카드 ───

  const HOVER_SHOW_DELAY_MS = 250;
  const HOVER_HIDE_DELAY_MS = 150;
  let activeCard = null;
  let activeAnchor = null;
  let pendingAnchor = null;
  let showHoverTimeout = null;
  let hideHoverTimeout = null;

  function clearShowHoverTimer() {
    if (showHoverTimeout !== null) {
      clearTimeout(showHoverTimeout);
      showHoverTimeout = null;
    }
  }

  function clearHideHoverTimer() {
    if (hideHoverTimeout !== null) {
      clearTimeout(hideHoverTimeout);
      hideHoverTimeout = null;
    }
  }

  function removeActiveHoverCard() {
    if (activeCard) {
      activeCard.remove();
      activeCard = null;
      activeAnchor = null;
    }
  }

  function scheduleHideHoverCard() {
    clearHideHoverTimer();
    hideHoverTimeout = setTimeout(() => {
      hideHoverTimeout = null;
      hideHoverCard();
    }, HOVER_HIDE_DELAY_MS);
  }

  // 보안: 텍스트 길이 제한
  function truncate(str, max) {
    if (!str) return '';
    return str.length > max ? str.slice(0, max) + '…' : str;
  }

  // 보안: 안전한 이미지 URL인지 검증
  function isSafeImageUrl(url) {
    try {
      const parsed = new URL(url);
      return parsed.protocol === 'https:' || parsed.protocol === 'http:';
    } catch { return false; }
  }

  // DOM API로 안전하게 요소 생성 (innerHTML 미사용 — H-01 XSS 방어)
  function createDiv(className, text) {
    const div = document.createElement('div');
    div.className = className;
    if (text) div.textContent = text;
    return div;
  }

  function insertThumbnail(container, src) {
    const img = document.createElement('img');
    img.src = src;
    img.alt = '\uBBF8\uB9AC\uBCF4\uAE30';
    img.referrerPolicy = 'no-referrer';
    img.style.cssText = 'display:block;width:100%;height:auto;border-radius:4px;';
    container.appendChild(img);
    container.style.display = '';
  }

  function showHoverCard(anchor) {
    if (!settings.hoverPreview) {
      if (pendingAnchor === anchor) pendingAnchor = null;
      return;
    }
    if (pendingAnchor !== anchor) return;
    if (!anchor.isConnected) {
      pendingAnchor = null;
      return;
    }
    if (typeof anchor.matches === 'function' && !anchor.matches(':hover')) {
      pendingAnchor = null;
      return;
    }

    clearHideHoverTimer();
    removeActiveHoverCard();

    const card = document.createElement('div');
    card.className = HOVER_CLASS;

    const title = anchor.getAttribute('data-hwp-title');
    const filename = extractFilename(anchor);

    // 썸네일 영역 (비동기 로드)
    const thumbContainer = document.createElement('div');
    thumbContainer.className = 'rhwp-hover-thumb';
    thumbContainer.style.display = 'none';
    card.appendChild(thumbContainer);

    // data-hwp-thumbnail 또는 캐시 또는 background 추출
    const existingThumb = anchor.getAttribute('data-hwp-thumbnail');
    if (existingThumb && isSafeImageUrl(existingThumb)) {
      insertThumbnail(thumbContainer, existingThumb);
    } else if (anchor.href) {
      const cached = thumbnailCache.get(anchor.href);
      if (cached && cached.dataUri) {
        insertThumbnail(thumbContainer, cached.dataUri);
      } else if (cached === undefined) {
        // 아직 요청하지 않은 URL → background에 추출 요청
        browser.runtime.sendMessage({ type: 'extract-thumbnail', url: anchor.href })
          .then(response => {
            if (response && response.dataUri) {
              thumbnailCache.set(anchor.href, response);
              if (activeCard === card && activeAnchor === anchor) {
                insertThumbnail(thumbContainer, response.dataUri);
              }
            } else {
              thumbnailCache.set(anchor.href, null);
            }
          }).catch(() => { thumbnailCache.set(anchor.href, null); });
      }
    }

    if (title) {
      card.appendChild(createDiv('rhwp-hover-title', truncate(title, 200)));

      const meta = [];
      const format = anchor.getAttribute('data-hwp-format');
      const pages = anchor.getAttribute('data-hwp-pages');
      const size = anchor.getAttribute('data-hwp-size');
      if (format) meta.push(truncate(format.toUpperCase(), 10));
      if (pages) meta.push(`${truncate(pages, 10)}\uCABD`);
      if (size) meta.push(formatSize(Number(size)));
      if (meta.length > 0) {
        card.appendChild(createDiv('rhwp-hover-meta', meta.join(' \u00B7 ')));
      }

      const author = anchor.getAttribute('data-hwp-author');
      const date = anchor.getAttribute('data-hwp-date');
      if (author || date) {
        const info = [];
        if (author) info.push(truncate(author, 100));
        if (date) info.push(truncate(date, 20));
        card.appendChild(createDiv('rhwp-hover-info', info.join(' \u00B7 ')));
      }

      const category = anchor.getAttribute('data-hwp-category');
      if (category) {
        card.appendChild(createDiv('rhwp-hover-category', truncate(category, 50)));
      }

      const description = anchor.getAttribute('data-hwp-description');
      if (description) {
        card.appendChild(createDiv('rhwp-hover-desc', truncate(description, 500)));
      }
    } else {
      const ext = filename.match(/\.(hwp|hwpx)$/i)?.[1]?.toUpperCase() || 'HWP';
      card.appendChild(createDiv('rhwp-hover-title', truncate(filename, 200)));
      card.appendChild(createDiv('rhwp-hover-meta', `${ext} \uBB38\uC11C`));
    }

    // 풋터 바 — 카드 전체 클릭 영역 암시
    const footer = document.createElement('div');
    footer.className = 'rhwp-hover-action';
    const footerLabel = createDiv('rhwp-hover-action-label', '▶\u2002rhwp로 열기');
    const footerArrow = createDiv('rhwp-hover-action-arrow', '→');
    footer.appendChild(footerLabel);
    footer.appendChild(footerArrow);
    card.appendChild(footer);

    // 위치 계산: 링크 아래에 표시, 뷰포트 넘치면 위로
    const rect = anchor.getBoundingClientRect();
    const cardLeft = rect.left + window.scrollX;
    let cardTop = rect.bottom + window.scrollY + 4;

    // DOM에 추가하여 크기 측정
    card.style.visibility = 'hidden';
    card.style.left = `${cardLeft}px`;
    card.style.top = `${cardTop}px`;
    document.body.appendChild(card);

    const cardRect = card.getBoundingClientRect();
    if (cardRect.bottom > window.innerHeight) {
      cardTop = rect.top + window.scrollY - card.offsetHeight - 4;
    }
    const maxLeft = window.innerWidth + window.scrollX - card.offsetWidth - 8;
    card.style.left = `${Math.max(8, Math.min(cardLeft, maxLeft))}px`;
    card.style.top = `${Math.max(0, cardTop)}px`;
    card.style.visibility = '';

    activeCard = card;
    activeAnchor = anchor;
    pendingAnchor = null;

    card.addEventListener('mouseenter', () => clearHideHoverTimer());
    card.addEventListener('mouseleave', () => scheduleHideHoverCard());
    card.addEventListener('click', () => {
      hideHoverCard();
      browser.runtime.sendMessage({
        type: 'open-hwp',
        url: anchor.href,
        filename: extractFilename(anchor)
      });
    });
  }

  function hideHoverCard() {
    clearShowHoverTimer();
    clearHideHoverTimer();
    pendingAnchor = null;
    removeActiveHoverCard();
  }

  function attachHoverEvents(anchor) {
    if (!settings.hoverPreview) return;
    anchor.addEventListener('mouseenter', () => {
      clearShowHoverTimer();
      clearHideHoverTimer();
      if (activeAnchor === anchor && activeCard) {
        pendingAnchor = null;
        return;
      }
      pendingAnchor = anchor;
      removeActiveHoverCard();
      showHoverTimeout = setTimeout(() => {
        showHoverTimeout = null;
        showHoverCard(anchor);
      }, HOVER_SHOW_DELAY_MS);
    });
    anchor.addEventListener('mouseleave', () => {
      if (pendingAnchor === anchor) {
        pendingAnchor = null;
      }
      clearShowHoverTimer();
      if (activeAnchor === anchor && activeCard) {
        scheduleHideHoverCard();
      }
    });
  }

  // ─── 차단 사유별 메시지 (한글은 content-script에서 생성) ───

  function getBlockedMessage(reason, hostname) {
    switch (reason) {
      case 'private-ip':
        return {
          title: '\uB85C\uCEEC \uC11C\uBC84(' + (hostname || 'localhost') + ') \uC811\uADFC\uC774 \uCC28\uB2E8\uB418\uC5C8\uC2B5\uB2C8\uB2E4.',
          guide: '\uC124\uC815 \u2192 \uAC1C\uBC1C \uD0ED \u2192 "\uAC1C\uBC1C\uC790 \uB3C4\uAD6C"\uB97C \uCF1C\uBA74 \uB85C\uCEEC \uC11C\uBC84\uC5D0 \uC811\uADFC\uD560 \uC218 \uC788\uC2B5\uB2C8\uB2E4.'
        };
      case 'domain-blocked':
        return {
          title: '\uC774 \uC0AC\uC774\uD2B8(' + (hostname || '') + ')\uC5D0\uC11C \uC790\uB3D9 \uC5F4\uAE30\uAC00 \uBE44\uD65C\uC131\uD654\uB418\uC5B4 \uC788\uC2B5\uB2C8\uB2E4.',
          guide: '\uC124\uC815 \u2192 \uC0AC\uC774\uD2B8 \uD0ED\uC5D0\uC11C \uB3C4\uBA54\uC778\uC744 \uCD94\uAC00\uD558\uAC70\uB098, "\uBAA8\uB4E0 \uC0AC\uC774\uD2B8\uC5D0\uC11C \uD65C\uC131\uD654"\uB97C \uCF1C\uC8FC\uC138\uC694.'
        };
      default:
        return {
          title: '\uD30C\uC77C\uC744 \uC5F4 \uC218 \uC5C6\uC2B5\uB2C8\uB2E4.',
          guide: ''
        };
    }
  }

  // ─── 토스트 알림 (차단 시 사용자 안내) ───

  let toastTimer = null;

  function showToast(message, guide) {
    // 기존 토스트 제거
    const existing = document.getElementById('rhwp-toast');
    if (existing) existing.remove();
    clearTimeout(toastTimer);

    const toast = document.createElement('div');
    toast.id = 'rhwp-toast';
    toast.style.cssText = `
      position: fixed; bottom: 24px; right: 24px; z-index: 2147483647;
      max-width: 360px; padding: 14px 18px;
      background: #1d1d1f; color: #f5f5f7;
      border-radius: 14px; font-family: -apple-system, sans-serif;
      font-size: 13px; line-height: 1.5;
      box-shadow: 0 8px 32px rgba(0,0,0,0.3);
      animation: rhwp-toast-in 0.3s ease;
    `;

    const msgEl = document.createElement('div');
    msgEl.textContent = message;
    msgEl.style.fontWeight = '500';
    toast.appendChild(msgEl);

    if (guide) {
      const guideEl = document.createElement('div');
      guideEl.textContent = guide;
      guideEl.style.cssText = 'margin-top: 6px; font-size: 12px; color: #a1a1a6;';
      toast.appendChild(guideEl);
    }

    // 닫기 버튼
    const closeBtn = document.createElement('button');
    closeBtn.textContent = '\u2715';
    closeBtn.style.cssText = `
      position: absolute; top: 8px; right: 10px;
      background: none; border: none; color: #86868b;
      font-size: 14px; cursor: pointer; padding: 2px 4px;
    `;
    closeBtn.addEventListener('click', () => {
      toast.remove();
      clearTimeout(toastTimer);
    });
    toast.appendChild(closeBtn);

    // 애니메이션 스타일 삽입 (1회)
    if (!document.getElementById('rhwp-toast-style')) {
      const style = document.createElement('style');
      style.id = 'rhwp-toast-style';
      style.textContent = `
        @keyframes rhwp-toast-in {
          from { opacity: 0; transform: translateY(12px); }
          to { opacity: 1; transform: translateY(0); }
        }
      `;
      document.head.appendChild(style);
    }

    document.body.appendChild(toast);

    // 8초 후 자동 제거
    toastTimer = setTimeout(() => {
      toast.style.opacity = '0';
      toast.style.transition = 'opacity 0.3s';
      setTimeout(() => toast.remove(), 300);
    }, 8000);
  }

  // ─── HWP 링크 클릭 가로채기 ───
  // Safari는 downloads API가 없으므로, HWP 링크 클릭 시 뷰어도 함께 연다.
  // 다운로드는 정상 진행 (preventDefault 하지 않음)

  function interceptHwpClick(anchor) {
    if (!settings.autoOpen) return;
    anchor.addEventListener('click', () => {
      browser.runtime.sendMessage({
        type: 'open-hwp',
        url: anchor.href,
        filename: extractFilename(anchor)
      }).then(res => {
        if (res && !res.ok && res.reason) {
          const msg = getBlockedMessage(res.reason, res.hostname);
          showToast(msg.title, msg.guide);
        }
      }).catch(() => {});
    });
  }

  // ─── 링크 처리 ───

  function processLinks(root = document) {
    const anchors = root.querySelectorAll('a[href]');
    for (const anchor of anchors) {
      if (anchor.hasAttribute(PROCESSED_ATTR)) continue;
      if (!isHwpLink(anchor)) continue;

      anchor.setAttribute(PROCESSED_ATTR, 'true');

      if (settings.showBadges) {
        const badge = createBadge(anchor);
        anchor.style.position = anchor.style.position || 'relative';
        anchor.insertAdjacentElement('afterend', badge);
      }

      interceptHwpClick(anchor);
      attachHoverEvents(anchor);
    }
  }

  function observeDynamicContent() {
    const observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        for (const node of mutation.addedNodes) {
          if (node.nodeType === Node.ELEMENT_NODE) {
            processLinks(node);
          }
        }
      }
    });
    observer.observe(document.body, { childList: true, subtree: true });
  }

  // ─── HWP 뷰어 열기 (iOS: iframe 오버레이, macOS: 새 탭) ───

  function openHwpViewer(url, filename) {
    const isIOS = /iPhone|iPad|iPod/.test(navigator.userAgent);

    if (isIOS) {
      openViewerOverlay(url, filename);
    } else {
      // macOS: 기존 방식 (새 탭)
      browser.runtime.sendMessage({
        type: 'open-hwp',
        url: url,
        filename: filename
      }).then(res => {
        if (res && !res.ok && res.reason) {
          const msg = getBlockedMessage(res.reason, res.hostname);
          showToast(msg.title, msg.guide);
        }
      }).catch(() => {});
    }
  }

  function openViewerOverlay(url, filename) {
    // 기존 오버레이 제거
    const existing = document.getElementById('rhwp-viewer-overlay');
    if (existing) existing.remove();

    // 전체화면 오버레이 컨테이너
    const overlay = document.createElement('div');
    overlay.id = 'rhwp-viewer-overlay';
    overlay.style.cssText = `
      position: fixed; inset: 0; z-index: 2147483647;
      background: rgba(0,0,0,0.85);
      display: flex; flex-direction: column;
    `;

    // 상단 바 (파일명 + 닫기)
    const topBar = document.createElement('div');
    topBar.style.cssText = `
      display: flex; align-items: center; justify-content: space-between;
      padding: 8px 12px; background: #1d1d1f; color: #f5f5f7;
      font-family: -apple-system, sans-serif; font-size: 14px;
      flex-shrink: 0;
    `;
    const titleEl = document.createElement('span');
    titleEl.textContent = filename || 'HWP Viewer';
    const closeBtn = document.createElement('button');
    closeBtn.textContent = '\u2715';
    closeBtn.style.cssText = `
      background: none; border: none; color: #f5f5f7;
      font-size: 20px; cursor: pointer; padding: 4px 8px;
    `;
    closeBtn.addEventListener('click', () => overlay.remove());
    topBar.appendChild(titleEl);
    topBar.appendChild(closeBtn);
    overlay.appendChild(topBar);

    // iframe (확장의 viewer.html 로드)
    const viewerUrl = browser.runtime.getURL('viewer.html');
    const params = new URLSearchParams();
    params.set('url', url);
    if (filename) params.set('filename', filename);
    const fullUrl = viewerUrl + '?' + params.toString();

    const iframe = document.createElement('iframe');
    iframe.src = fullUrl;
    iframe.style.cssText = `
      flex: 1; border: none; width: 100%;
      background: white;
    `;
    iframe.setAttribute('allow', 'scripts');
    overlay.appendChild(iframe);

    document.body.appendChild(overlay);

    // ESC 키로 닫기
    const escHandler = (e) => {
      if (e.key === 'Escape') {
        overlay.remove();
        document.removeEventListener('keydown', escHandler);
      }
    };
    document.addEventListener('keydown', escHandler);
  }
})();
