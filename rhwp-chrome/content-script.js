// rhwp Chrome Extension - Content Script
// 웹페이지에서 HWP/HWPX 링크를 자동 감지하고 rhwp 아이콘을 삽입

(() => {
  'use strict';

  const HWP_EXTENSIONS = /\.(hwp|hwpx)(\?.*)?$/i;
  const BADGE_CLASS = 'rhwp-badge';
  const HOVER_CLASS = 'rhwp-hover-card';
  const PROCESSED_ATTR = 'data-rhwp-processed';

  // 사용자 설정 (Service Worker에서 로드)
  let settings = { showBadges: true, hoverPreview: true };

  chrome.runtime.sendMessage({ type: 'get-settings' }, (result) => {
    if (result) settings = result;
    // 설정 로드 후 초기 스캔
    if (settings.showBadges) {
      processLinks();
      observeDynamicContent();
    }
    // HWP 링크 썸네일 프리페치 (페이지 로드 후 유휴 시간에)
    if (settings.hoverPreview) {
      prefetchThumbnails();
    }
  });

  // 확장 존재 알림
  document.documentElement.setAttribute('data-hwp-extension', 'rhwp');
  document.documentElement.setAttribute('data-hwp-extension-version', '0.2.2');
  window.dispatchEvent(new CustomEvent('hwp-extension-ready', {
    detail: { name: 'rhwp', version: '0.2.2', capabilities: ['preview', 'edit', 'print'] }
  }));

  // 개발자 도구 주입 (페이지 컨텍스트에 rhwpDev 노출)
  const devScript = document.createElement('script');
  devScript.src = chrome.runtime.getURL('dev-tools-inject.js');
  (document.head || document.documentElement).appendChild(devScript);
  devScript.onload = () => devScript.remove();

  // ─── 유틸리티 ───

  // DOM API로 안전하게 텍스트 요소 생성 (innerHTML 미사용 — H-01 XSS 방어)
  function createEl(tag, className, text) {
    const el = document.createElement(tag);
    if (className) el.className = className;
    if (text != null) el.textContent = text;
    return el;
  }

  // 보안: 텍스트 길이 제한
  function truncate(str, max) {
    if (!str) return '';
    return str.length > max ? str.slice(0, max) + '…' : str;
  }

  // 보안: 안전한 이미지 URL인지 검증 (javascript: 등 차단)
  function isSafeImageUrl(url) {
    try {
      const parsed = new URL(url);
      return parsed.protocol === 'https:' || parsed.protocol === 'http:';
    } catch { return false; }
  }

  function insertThumbnailImg(thumbDiv, dataUri) {
    const img = document.createElement('img');
    img.src = new URL(dataUri).href;
    img.alt = '미리보기';
    img.referrerPolicy = 'no-referrer';
    thumbDiv.textContent = '';
    thumbDiv.className = 'rhwp-hover-thumb';
    thumbDiv.appendChild(img);
  }

  // ─── 링크 감지 ───

  function isHwpLink(anchor) {
    if (!anchor.href) return false;
    if (anchor.getAttribute('data-hwp') === 'true') return true;
    return HWP_EXTENSIONS.test(anchor.href);
  }

  function createBadge(anchor) {
    const badge = document.createElement('span');
    badge.className = BADGE_CLASS;

    const title = anchor.getAttribute('data-hwp-title');
    const pages = anchor.getAttribute('data-hwp-pages');
    const size = anchor.getAttribute('data-hwp-size');

    let tooltip;
    if (title && pages && size) {
      tooltip = chrome.i18n.getMessage('badgeTooltipWithInfo', [title, pages, formatSize(Number(size))]);
    } else if (title) {
      tooltip = title;
    } else {
      tooltip = chrome.i18n.getMessage('badgeTooltip');
    }
    badge.title = tooltip;

    badge.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();
      chrome.runtime.sendMessage({ type: 'open-hwp', url: anchor.href });
    });

    return badge;
  }

  // ─── 호버 미리보기 카드 ───

  let activeCard = null;
  let hoverTimeout = null;
  const thumbnailCache = new Map(); // URL → dataUri 캐시 (content-script 측)

  function showHoverCard(anchor) {
    if (!settings.hoverPreview) return;

    hideHoverCard();

    const card = document.createElement('div');
    card.className = HOVER_CLASS;

    const title = anchor.getAttribute('data-hwp-title');
    const pages = anchor.getAttribute('data-hwp-pages');
    const size = anchor.getAttribute('data-hwp-size');
    const author = anchor.getAttribute('data-hwp-author');
    const date = anchor.getAttribute('data-hwp-date');
    const category = anchor.getAttribute('data-hwp-category');
    const description = anchor.getAttribute('data-hwp-description');
    const format = anchor.getAttribute('data-hwp-format');
    const thumbnail = anchor.getAttribute('data-hwp-thumbnail');

    // 썸네일 영역 (사전 지정 또는 자동 추출 후 삽입)
    // DOM API로 안전하게 생성 — innerHTML 미사용 (H-01 XSS 방어)
    const thumbContainer = document.createElement('div');
    if (thumbnail && isSafeImageUrl(thumbnail)) {
      insertThumbnailImg(thumbContainer, thumbnail);
    } else {
      // 자동 추출 플레이스홀더
      thumbContainer.className = 'rhwp-hover-thumb rhwp-thumb-loading';
      thumbContainer.appendChild(createEl('span', 'rhwp-thumb-spinner', '⏳'));
    }
    card.appendChild(thumbContainer);

    const titleText = title || anchor.href.split('/').pop().split('?')[0];
    card.appendChild(createEl('div', 'rhwp-hover-title', truncate(titleText, 200)));

    const meta = [];
    if (format) meta.push(truncate(format.toUpperCase(), 10));
    if (pages && !isNaN(Number(pages))) meta.push(`${pages}쪽`);
    if (size && !isNaN(Number(size))) meta.push(formatSize(Number(size)));
    if (meta.length > 0) {
      card.appendChild(createEl('div', 'rhwp-hover-meta', meta.join(' · ')));
    }

    if (author || date) {
      const info = [];
      if (author) info.push(truncate(author, 100));
      if (date) info.push(truncate(date, 20));
      card.appendChild(createEl('div', 'rhwp-hover-info', info.join(' · ')));
    }

    if (category) {
      card.appendChild(createEl('div', 'rhwp-hover-category', truncate(category, 50)));
    }

    if (description) {
      card.appendChild(createEl('div', 'rhwp-hover-desc', truncate(description, 500)));
    }

    // 풋터 바 — 카드 전체 클릭 영역 암시
    const footer = document.createElement('div');
    footer.className = 'rhwp-hover-action';
    const footerLabel = createEl('span', 'rhwp-hover-action-label', '▶\u2002rhwp로 열기');
    const footerArrow = createEl('span', 'rhwp-hover-action-arrow', '→');
    footer.appendChild(footerLabel);
    footer.appendChild(footerArrow);
    card.appendChild(footer);

    // 카드 전체 클릭 시 HWP 열기
    card.addEventListener('click', () => {
      chrome.runtime.sendMessage({ type: 'open-hwp', url: anchor.href });
      hideHoverCard();
    });

    // 위치 계산 — 뷰포트 하단을 넘으면 링크 위쪽에 표시
    const rect = anchor.getBoundingClientRect();
    document.body.appendChild(card);
    activeCard = card;

    const cardHeight = card.offsetHeight;
    const spaceBelow = window.innerHeight - rect.bottom;
    const spaceAbove = rect.top;

    let left = rect.left + window.scrollX;
    let top;

    if (spaceBelow >= cardHeight + 8) {
      // 아래에 공간 충분 → 링크 아래에 표시
      top = rect.bottom + window.scrollY + 4;
    } else if (spaceAbove >= cardHeight + 8) {
      // 위에 공간 충분 → 링크 위에 표시
      top = rect.top + window.scrollY - cardHeight - 4;
    } else {
      // 양쪽 다 부족 → 뷰포트 하단에 맞춤
      top = window.scrollY + window.innerHeight - cardHeight - 8;
    }

    // 좌우 넘침 방지
    const cardWidth = card.offsetWidth;
    if (left + cardWidth > window.scrollX + window.innerWidth - 8) {
      left = window.scrollX + window.innerWidth - cardWidth - 8;
    }
    if (left < window.scrollX + 8) {
      left = window.scrollX + 8;
    }

    card.style.left = `${left}px`;
    card.style.top = `${top}px`;

    // 카드에 마우스 올리면 유지
    card.addEventListener('mouseenter', () => clearTimeout(hoverTimeout));
    card.addEventListener('mouseleave', () => hideHoverCard());

    // data-hwp-thumbnail이 없으면 캐시 확인 또는 Service Worker에 추출 요청
    if (!thumbnail && anchor.href) {
      const cached = thumbnailCache.get(anchor.href);
      if (cached) {
        // 캐시 히트: 즉시 표시
        const thumbDiv = card.querySelector('.rhwp-thumb-loading');
        if (thumbDiv) {
          insertThumbnailImg(thumbDiv, cached.dataUri);
        }
      } else if (cached === null) {
        // 이전에 추출 실패한 URL: 플레이스홀더 제거
        const thumbDiv = card.querySelector('.rhwp-thumb-loading');
        if (thumbDiv) thumbDiv.remove();
      } else {
        // 캐시 미스: Service Worker에 추출 요청
        chrome.runtime.sendMessage(
          { type: 'extract-thumbnail', url: anchor.href },
          (response) => {
            if (response && response.dataUri) {
              thumbnailCache.set(anchor.href, response);
              if (activeCard === card) {
                const thumbDiv = card.querySelector('.rhwp-thumb-loading');
                if (thumbDiv) {
                  insertThumbnailImg(thumbDiv, response.dataUri);
                }
              }
            } else {
              thumbnailCache.set(anchor.href, null); // 실패 기록
              if (activeCard === card) {
                const thumbDiv = card.querySelector('.rhwp-thumb-loading');
                if (thumbDiv) thumbDiv.remove();
              }
            }
          }
        );
      }
    }
  }

  function hideHoverCard() {
    if (activeCard) {
      activeCard.remove();
      activeCard = null;
    }
    clearTimeout(hoverTimeout);
  }

  function attachHoverEvents(anchor) {
    if (!settings.hoverPreview) return;

    anchor.addEventListener('mouseenter', () => {
      clearTimeout(hoverTimeout); // 이전 디바운스 타이머 취소
      hideHoverCard(); // 이전 카드 제거
      hoverTimeout = setTimeout(() => showHoverCard(anchor), 300);
    });
    anchor.addEventListener('mouseleave', () => {
      hoverTimeout = setTimeout(() => hideHoverCard(), 200);
    });
  }

  // ─── 썸네일 프리페치 ───

  const PREFETCH_CONCURRENCY = 3; // 동시 fetch 최대 수
  let prefetchQueue = [];
  let prefetchActive = 0;

  function prefetchThumbnails() {
    // 페이지 로드 후 1초 대기 (렌더링 우선)
    setTimeout(() => {
      const anchors = document.querySelectorAll('a[href]');
      for (const anchor of anchors) {
        if (!isHwpLink(anchor)) continue;
        if (anchor.getAttribute('data-hwp-thumbnail')) continue; // 사전 지정된 것은 제외
        if (thumbnailCache.has(anchor.href)) continue; // 이미 캐시됨
        prefetchQueue.push(anchor.href);
      }
      // 중복 제거
      prefetchQueue = [...new Set(prefetchQueue)];
      drainPrefetchQueue();
    }, 1000);
  }

  function drainPrefetchQueue() {
    while (prefetchActive < PREFETCH_CONCURRENCY && prefetchQueue.length > 0) {
      const url = prefetchQueue.shift();
      if (thumbnailCache.has(url)) continue;
      prefetchActive++;
      chrome.runtime.sendMessage(
        { type: 'extract-thumbnail', url },
        (response) => {
          prefetchActive--;
          if (response && response.dataUri) {
            thumbnailCache.set(url, response);
          } else {
            thumbnailCache.set(url, null);
          }
          drainPrefetchQueue();
        }
      );
    }
  }

  // ─── 유틸리티 ───

  function formatSize(bytes) {
    if (bytes < 1024) return `${bytes}B`;
    if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)}KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)}MB`;
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
})();
