(function () {
  'use strict';

  const VERSION = '0.2.1';
  const HWP_EXTENSIONS = /\.(hwp|hwpx)(\?.*)?$/i;
  const BADGE_CLASS = 'rhwp-badge';
  const HOVER_CLASS = 'rhwp-hover-card';
  const PROCESSED_ATTR = 'data-rhwp-processed';
  const PREFETCH_CONCURRENCY = 3;

  let settings = { autoOpen: true, showBadges: true, hoverPreview: true };
  let activeCard = null;
  let hoverTimeout = null;
  let prefetchQueue = [];
  let prefetchActive = 0;
  const thumbnailCache = new Map();

  browser.runtime.sendMessage({ type: 'get-settings' })
    .then((result) => {
      if (result) settings = { ...settings, ...result };
    })
    .catch(() => {})
    .finally(init);

  function init() {
    announceExtension();
    injectDevTools();

    if (settings.showBadges) {
      processLinks();
      observeDynamicContent();
    } else if (settings.autoOpen || settings.hoverPreview) {
      processLinks();
      observeDynamicContent();
    }

    if (settings.hoverPreview) {
      prefetchThumbnails();
    }
  }

  function announceExtension() {
    document.documentElement.setAttribute('data-hwp-extension', 'rhwp');
    document.documentElement.setAttribute('data-hwp-extension-version', VERSION);
    window.dispatchEvent(new CustomEvent('hwp-extension-ready', {
      detail: { name: 'rhwp', version: VERSION, capabilities: ['preview', 'edit', 'print'] },
    }));
  }

  function injectDevTools() {
    const devScript = document.createElement('script');
    devScript.src = browser.runtime.getURL('dev-tools-inject.js');
    (document.head || document.documentElement).appendChild(devScript);
    devScript.onload = () => devScript.remove();
  }

  function createEl(tag, className, text) {
    const el = document.createElement(tag);
    if (className) el.className = className;
    if (text != null) el.textContent = text;
    return el;
  }

  function truncate(str, max) {
    if (!str) return '';
    return str.length > max ? `${str.slice(0, max)}...` : str;
  }

  function formatSize(bytes) {
    if (!Number.isFinite(bytes)) return '';
    if (bytes < 1024) return `${bytes}B`;
    if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)}KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)}MB`;
  }

  function isHwpLink(anchor) {
    if (!anchor.href) return false;
    if (anchor.getAttribute('data-hwp') === 'true') return true;
    return HWP_EXTENSIONS.test(anchor.href);
  }

  function filenameFromAnchor(anchor) {
    try {
      const url = new URL(anchor.href);
      const name = decodeURIComponent(url.pathname.split('/').pop() || '');
      if (name) return name;
    } catch {
      // Fall through to text fallback.
    }
    return anchor.textContent.trim() || 'document.hwp';
  }

  function isSafeImageUrl(url) {
    try {
      const parsed = new URL(url);
      return parsed.protocol === 'https:' || parsed.protocol === 'http:';
    } catch {
      return false;
    }
  }

  function openHwp(anchor) {
    return browser.runtime.sendMessage({
      type: 'open-hwp',
      url: anchor.href,
      filename: filenameFromAnchor(anchor),
    });
  }

  function createBadge(anchor) {
    const badge = document.createElement('span');
    badge.className = BADGE_CLASS;

    const title = anchor.getAttribute('data-hwp-title');
    const pages = anchor.getAttribute('data-hwp-pages');
    const size = anchor.getAttribute('data-hwp-size');

    if (title && pages && size) {
      badge.title = browser.i18n.getMessage(
        'badgeTooltipWithInfo',
        [title, pages, formatSize(Number(size))],
      );
    } else {
      badge.title = title || browser.i18n.getMessage('badgeTooltip') || 'Open with rhwp';
    }

    badge.addEventListener('click', (event) => {
      event.preventDefault();
      event.stopPropagation();
      openHwp(anchor).catch(() => {});
    });

    return badge;
  }

  function attachClickInterceptor(anchor) {
    if (!settings.autoOpen) return;

    anchor.addEventListener('click', (event) => {
      if (event.defaultPrevented) return;
      if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;

      event.preventDefault();
      openHwp(anchor).catch(() => {
        window.location.href = anchor.href;
      });
    });
  }

  function insertThumbnailImg(thumbDiv, dataUri) {
    const img = document.createElement('img');
    img.src = new URL(dataUri).href;
    img.alt = 'Preview';
    img.referrerPolicy = 'no-referrer';
    thumbDiv.textContent = '';
    thumbDiv.className = 'rhwp-hover-thumb';
    thumbDiv.appendChild(img);
  }

  function showHoverCard(anchor) {
    if (!settings.hoverPreview) return;

    hideHoverCard();

    const card = document.createElement('div');
    card.className = HOVER_CLASS;

    const thumbnail = anchor.getAttribute('data-hwp-thumbnail');
    const thumbContainer = document.createElement('div');
    if (thumbnail && isSafeImageUrl(thumbnail)) {
      insertThumbnailImg(thumbContainer, thumbnail);
    } else {
      thumbContainer.className = 'rhwp-hover-thumb rhwp-thumb-loading';
      thumbContainer.appendChild(createEl('span', 'rhwp-thumb-spinner', '...'));
    }
    card.appendChild(thumbContainer);

    const title = anchor.getAttribute('data-hwp-title') || filenameFromAnchor(anchor);
    card.appendChild(createEl('div', 'rhwp-hover-title', truncate(title, 200)));

    const meta = [];
    const format = anchor.getAttribute('data-hwp-format');
    const pages = anchor.getAttribute('data-hwp-pages');
    const size = anchor.getAttribute('data-hwp-size');
    if (format) meta.push(truncate(format.toUpperCase(), 10));
    if (pages && !Number.isNaN(Number(pages))) meta.push(`${pages} pages`);
    if (size && !Number.isNaN(Number(size))) meta.push(formatSize(Number(size)));
    if (meta.length > 0) {
      card.appendChild(createEl('div', 'rhwp-hover-meta', meta.join(' \u00B7 ')));
    }

    const author = anchor.getAttribute('data-hwp-author');
    const date = anchor.getAttribute('data-hwp-date');
    if (author || date) {
      const info = [];
      if (author) info.push(truncate(author, 100));
      if (date) info.push(truncate(date, 20));
      card.appendChild(createEl('div', 'rhwp-hover-info', info.join(' \u00B7 ')));
    }

    const category = anchor.getAttribute('data-hwp-category');
    if (category) {
      card.appendChild(createEl('div', 'rhwp-hover-category', truncate(category, 50)));
    }

    const description = anchor.getAttribute('data-hwp-description');
    if (description) {
      card.appendChild(createEl('div', 'rhwp-hover-desc', truncate(description, 500)));
    }

    const footer = document.createElement('div');
    footer.className = 'rhwp-hover-action';
    footer.appendChild(createEl('span', 'rhwp-hover-action-label', 'Open with rhwp'));
    footer.appendChild(createEl('span', 'rhwp-hover-action-arrow', '>'));
    card.appendChild(footer);

    card.addEventListener('click', () => {
      openHwp(anchor).catch(() => {});
      hideHoverCard();
    });

    const rect = anchor.getBoundingClientRect();
    document.body.appendChild(card);
    activeCard = card;

    const cardHeight = card.offsetHeight;
    const spaceBelow = window.innerHeight - rect.bottom;
    const spaceAbove = rect.top;

    let left = rect.left + window.scrollX;
    let top;

    if (spaceBelow >= cardHeight + 8) {
      top = rect.bottom + window.scrollY + 4;
    } else if (spaceAbove >= cardHeight + 8) {
      top = rect.top + window.scrollY - cardHeight - 4;
    } else {
      top = window.scrollY + window.innerHeight - cardHeight - 8;
    }

    const cardWidth = card.offsetWidth;
    if (left + cardWidth > window.scrollX + window.innerWidth - 8) {
      left = window.scrollX + window.innerWidth - cardWidth - 8;
    }
    if (left < window.scrollX + 8) left = window.scrollX + 8;

    card.style.left = `${left}px`;
    card.style.top = `${top}px`;

    card.addEventListener('mouseenter', () => clearTimeout(hoverTimeout));
    card.addEventListener('mouseleave', () => hideHoverCard());

    loadThumbnail(anchor, card, thumbnail);
  }

  function loadThumbnail(anchor, card, thumbnail) {
    if (thumbnail || !anchor.href) return;

    const cached = thumbnailCache.get(anchor.href);
    if (cached) {
      const thumbDiv = card.querySelector('.rhwp-thumb-loading');
      if (thumbDiv) insertThumbnailImg(thumbDiv, cached.dataUri);
      return;
    }

    if (cached === null) {
      const thumbDiv = card.querySelector('.rhwp-thumb-loading');
      if (thumbDiv) thumbDiv.remove();
      return;
    }

    browser.runtime.sendMessage({ type: 'extract-thumbnail', url: anchor.href })
      .then((response) => {
        if (response?.dataUri) {
          thumbnailCache.set(anchor.href, response);
          if (activeCard === card) {
            const thumbDiv = card.querySelector('.rhwp-thumb-loading');
            if (thumbDiv) insertThumbnailImg(thumbDiv, response.dataUri);
          }
        } else {
          thumbnailCache.set(anchor.href, null);
          if (activeCard === card) {
            const thumbDiv = card.querySelector('.rhwp-thumb-loading');
            if (thumbDiv) thumbDiv.remove();
          }
        }
      })
      .catch(() => thumbnailCache.set(anchor.href, null));
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
      clearTimeout(hoverTimeout);
      hideHoverCard();
      hoverTimeout = setTimeout(() => showHoverCard(anchor), 300);
    });
    anchor.addEventListener('mouseleave', () => {
      hoverTimeout = setTimeout(() => hideHoverCard(), 200);
    });
  }

  function prefetchThumbnails() {
    setTimeout(() => {
      const anchors = document.querySelectorAll('a[href]');
      for (const anchor of anchors) {
        if (!isHwpLink(anchor)) continue;
        if (anchor.getAttribute('data-hwp-thumbnail')) continue;
        if (thumbnailCache.has(anchor.href)) continue;
        prefetchQueue.push(anchor.href);
      }
      prefetchQueue = [...new Set(prefetchQueue)];
      drainPrefetchQueue();
    }, 1000);
  }

  function drainPrefetchQueue() {
    while (prefetchActive < PREFETCH_CONCURRENCY && prefetchQueue.length > 0) {
      const url = prefetchQueue.shift();
      if (thumbnailCache.has(url)) continue;

      prefetchActive++;
      browser.runtime.sendMessage({ type: 'extract-thumbnail', url })
        .then((response) => {
          thumbnailCache.set(url, response?.dataUri ? response : null);
        })
        .catch(() => thumbnailCache.set(url, null))
        .finally(() => {
          prefetchActive--;
          drainPrefetchQueue();
        });
    }
  }

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

      attachClickInterceptor(anchor);
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
