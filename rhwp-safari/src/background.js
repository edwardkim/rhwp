// rhwp Safari Web Extension - Background Script
// Safari Web Extension은 비영속적 배경 페이지로 동작
// browser.* (WebExtension 표준) API 사용
// 보안 모듈: rhwp-shared/security/ 참조 (빌드 시 인라인)

'use strict';

// ─── 보안: URL 검증 ───

const DEFAULT_ALLOWED_DOMAINS = ['.go.kr', '.or.kr', '.ac.kr', '.mil.kr', '.korea.kr', '.sc.kr'];
const PRIVATE_IP_PATTERNS = [
  /^127\./, /^10\./, /^192\.168\./, /^172\.(1[6-9]|2\d|3[01])\./,
  /^169\.254\./, /^0\./, /^\[::1\]/, /^localhost$/i, /\.local$/i,
];
const HWP_SIGNATURE = [0xD0, 0xCF, 0x11, 0xE0];
const HWPX_SIGNATURE = [0x50, 0x4B, 0x03, 0x04];
const MAX_FILE_SIZE = 20 * 1024 * 1024; // 20MB

function isPrivateHost(hostname) {
  return PRIVATE_IP_PATTERNS.some(re => re.test(hostname));
}

function validateUrl(urlString) {
  if (!urlString || typeof urlString !== 'string') return { valid: false, reason: 'URL 비어있음' };
  let parsed;
  try { parsed = new URL(urlString); } catch { return { valid: false, reason: 'URL 파싱 실패' }; }
  if (parsed.protocol !== 'https:' && parsed.protocol !== 'http:') {
    return { valid: false, reason: `차단된 프로토콜: ${parsed.protocol}` };
  }
  if (parsed.username || parsed.password) {
    return { valid: false, reason: 'URL에 userinfo(@) 포함' };
  }
  // 내부 IP는 validateUrl 단계에서 차단하지 않고, 호출부에서 devMode 체크 후 판단
  return { valid: true, parsed, isPrivate: isPrivateHost(parsed.hostname) };
}

function isAllowedDomain(hostname, domains) {
  return domains.some(d => hostname.endsWith(d));
}

function hasHwpExtension(parsed) {
  const p = parsed.pathname.toLowerCase();
  return p.endsWith('.hwp') || p.endsWith('.hwpx');
}

function isDownloadEndpoint(parsed) {
  const p = parsed.pathname.toLowerCase();
  return /\.(do|action|jsp|aspx|php)$/i.test(p) || /download|filedown|attach/i.test(p);
}

async function getAllowedDomains() {
  try {
    const s = await browser.storage.local.get({ allowedDomains: DEFAULT_ALLOWED_DOMAINS });
    return s.allowedDomains;
  } catch { return DEFAULT_ALLOWED_DOMAINS; }
}

function verifyHwpSignature(bytes) {
  if (bytes.length < 4) return { isHwp: false, format: null };
  const b = new Uint8Array(bytes.slice(0, 4));
  if (HWP_SIGNATURE.every((v, i) => b[i] === v)) return { isHwp: true, format: 'hwp' };
  if (HWPX_SIGNATURE.every((v, i) => b[i] === v)) return { isHwp: true, format: 'hwpx' };
  return { isHwp: false, format: null };
}

// ─── 보안: 파일명 새니타이즈 ───

function sanitizeFilename(filename) {
  if (!filename || typeof filename !== 'string') return '';
  let safe = filename;
  if (typeof safe.normalize === 'function') safe = safe.normalize('NFC');
  try { safe = decodeURIComponent(safe); try { safe = decodeURIComponent(safe); } catch {} } catch {}
  safe = safe.replace(/\0/g, '').replace(/\.\./g, '').replace(/[/\\]/g, '_');
  safe = safe.replace(/[^a-zA-Z0-9가-힣ㄱ-ㅎㅏ-ㅣ.\-_ ]/g, '');
  safe = safe.replace(/^[\s.]+|[\s.]+$/g, '');
  return safe.slice(0, 255) || 'document';
}

// ─── 보안: 발신자 검증 ───

function isInternalPage(sender) {
  return sender?.url?.startsWith(browser.runtime.getURL('')) || false;
}

function isContentScript(sender) {
  return !!(sender?.tab?.id != null);
}

// ─── 보안: 이벤트 로깅 ───

async function logSecurity(type, url, reason) {
  try {
    const s = await browser.storage.local.get({ securityLog: false });
    if (!s.securityLog) return;
    const data = await browser.storage.local.get({ securityEvents: [] });
    const events = data.securityEvents;
    events.push({ time: new Date().toISOString(), type, url: (url || '').slice(0, 500), reason });
    while (events.length > 250) events.shift();
    await browser.storage.local.set({ securityEvents: events });
  } catch {}
}

// ─── 뷰어 탭 관리 ───

/**
 * 뷰어 탭을 연다.
 * @param {object} options
 * @param {string} [options.url] - HWP 파일 URL
 * @param {string} [options.filename] - 파일명
 * @param {boolean} [options.explicit] - 사용자 명시적 행위 (배지 클릭, 컨텍스트 메뉴)
 */
/**
 * 뷰어 탭을 연다.
 * @returns {{ ok: boolean, blocked?: string, guide?: string }}
 */
async function openViewer(options = {}) {
  const viewerBase = browser.runtime.getURL('viewer.html');
  const params = new URLSearchParams();

  if (options.url) {
    // URL 검증 (C-02): 프로토콜, userinfo 검사는 항상 수행
    const result = validateUrl(options.url);
    if (!result.valid) {
      console.warn('[rhwp] URL 차단:', result.reason, options.url);
      await logSecurity('url-blocked', options.url, result.reason);
      return { ok: false, blocked: result.reason };
    }
    const parsed = result.parsed;

    // 내부 IP 차단 (devMode 시 허용)
    const devCheck = await browser.storage.local.get({ devMode: false });
    if (result.isPrivate && !devCheck.devMode) {
      console.warn('[rhwp] 내부 IP:', parsed.hostname);
      await logSecurity('url-blocked', options.url, 'private-ip');
      return { ok: false, reason: 'private-ip', hostname: parsed.hostname };
    }

    // 도메인 제한: 자동 동작에만 적용
    if (!options.explicit) {
      const domains = await getAllowedDomains();
      const allSites = await browser.storage.local.get({ allSitesEnabled: false });

      if (!hasHwpExtension(parsed) && !allSites.allSitesEnabled) {
        if (!isAllowedDomain(parsed.hostname, domains) && !isDownloadEndpoint(parsed)) {
          console.warn('[rhwp] domain-blocked:', parsed.hostname);
          await logSecurity('url-blocked', options.url, 'domain-blocked');
          return { ok: false, reason: 'domain-blocked', hostname: parsed.hostname };
        }
      }
    }

    params.set('url', options.url);
  }

  if (options.filename) {
    params.set('filename', sanitizeFilename(options.filename));
  }

  const query = params.toString();
  const fullUrl = query ? `${viewerBase}?${query}` : viewerBase;
  browser.tabs.create({ url: fullUrl });
  return { ok: true };
}

// ─── 컨텍스트 메뉴 ───

const MENU_ID = 'rhwp-open-link';

function setupContextMenus() {
  browser.contextMenus.removeAll(() => {
    browser.contextMenus.create({
      id: MENU_ID,
      title: browser.i18n.getMessage('contextMenuOpen') || 'rhwp로 열기',
      contexts: ['link'],
    });
  });
}

browser.contextMenus.onClicked.addListener((info) => {
  if (info.menuItemId !== MENU_ID || !info.linkUrl) return;
  openViewer({ url: info.linkUrl, explicit: true });
});

// ─── 메시지 라우팅 ───

browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
  console.log('[rhwp-bg] 메시지 수신:', message.type, 'sender:', sender?.tab?.id, sender?.url);
  switch (message.type) {
    case 'open-hwp': {
      // 발신자 검증 (H-02)
      console.log('[rhwp-bg] sender 검증:', { tab: sender?.tab, url: sender?.url });
      if (!isContentScript(sender)) {
        console.warn('[rhwp-bg] sender 거부: content script 아님');
        logSecurity('sender-blocked', message.url, 'open-hwp: content script가 아닌 발신자');
        sendResponse({ error: 'Unauthorized' });
        return;
      }
      console.log('[rhwp-bg] openViewer 호출:', message.url);
      openViewer({ url: message.url, filename: message.filename, explicit: true })
        .then(result => sendResponse(result || { ok: true }))
        .catch(err => sendResponse({ ok: false, blocked: err.message }));
      return true; // 비동기 응답
    }

    case 'fetch-file': {
      // 발신자 검증: 내부 페이지만 (H-02)
      if (!isInternalPage(sender)) {
        logSecurity('sender-blocked', message.url, 'fetch-file: 외부 발신자');
        sendResponse({ error: 'Unauthorized' });
        return;
      }

      // URL 검증 (C-01)
      const urlResult = validateUrl(message.url);
      if (!urlResult.valid) {
        logSecurity('fetch-blocked', message.url, urlResult.reason);
        sendResponse({ error: urlResult.reason });
        return;
      }

      // fetch 실행 (리다이렉트 수동 처리, 쿠키 미전송)
      (async () => {
        try {
          const settings = await browser.storage.local.get({ allowHttp: true, maxFileSize: 20, devMode: false });

          // 내부 IP 이중 체크 (devMode 시 허용)
          if (urlResult.isPrivate && !settings.devMode) {
            logSecurity('fetch-blocked', message.url, '내부 IP');
            sendResponse({ error: '내부 네트워크 접근 차단' });
            return;
          }
          const maxSize = (settings.maxFileSize || 20) * 1024 * 1024;

          // HTTP 처리
          let fetchUrl = message.url;
          if (urlResult.parsed.protocol === 'http:' && !settings.allowHttp) {
            sendResponse({ error: 'HTTP 차단 (설정에서 비허용)' });
            return;
          }

          const res = await fetch(fetchUrl, {
            credentials: 'omit',
            redirect: 'manual',
          });

          // 리다이렉트 처리: 대상 URL 재검증
          if (res.type === 'opaqueredirect' || (res.status >= 300 && res.status < 400)) {
            const location = res.headers.get('location');
            if (location) {
              const redirectResult = validateUrl(new URL(location, fetchUrl).href);
              if (!redirectResult.valid || isPrivateHost(redirectResult.parsed.hostname)) {
                logSecurity('fetch-blocked', location, '리다이렉트 대상 차단');
                sendResponse({ error: '리다이렉트 대상이 안전하지 않음' });
                return;
              }
              // 재검증 통과 시 리다이렉트 따라가기
              const res2 = await fetch(new URL(location, fetchUrl).href, { credentials: 'omit' });
              if (!res2.ok) throw new Error(`HTTP ${res2.status}`);
              const buf = await res2.arrayBuffer();
              if (buf.byteLength > maxSize) throw new Error('파일 크기 초과');
              const sig = verifyHwpSignature(buf);
              if (!sig.isHwp) {
                logSecurity('signature-blocked', fetchUrl, '매직 넘버 불일치');
                sendResponse({ error: 'HWP 파일이 아닙니다' });
                return;
              }
              sendResponse({ data: buf });
              return;
            }
          }

          if (!res.ok) throw new Error(`HTTP ${res.status}`);

          // Content-Type 검증
          const ct = (res.headers.get('content-type') || '').toLowerCase();
          if (ct.includes('text/html') || ct.includes('application/json') || ct.includes('text/javascript')) {
            logSecurity('fetch-blocked', fetchUrl, `차단된 Content-Type: ${ct}`);
            sendResponse({ error: `예상치 않은 응답 유형: ${ct}` });
            return;
          }

          const buf = await res.arrayBuffer();

          // 크기 제한
          if (buf.byteLength > maxSize) {
            sendResponse({ error: `파일 크기 초과 (${Math.round(buf.byteLength / 1024 / 1024)}MB > ${settings.maxFileSize}MB)` });
            return;
          }

          // 매직 넘버 검증
          const sig = verifyHwpSignature(buf);
          if (!sig.isHwp) {
            logSecurity('signature-blocked', fetchUrl, '매직 넘버 불일치');
            sendResponse({ error: 'HWP 파일이 아닙니다' });
            return;
          }

          // ArrayBuffer 직접 전달 (N-04 메모리 폭발 방지)
          sendResponse({ data: buf });
        } catch (err) {
          sendResponse({ error: err.message });
        }
      })();
      return true; // 비동기 응답
    }

    case 'extract-thumbnail': {
      if (!isContentScript(sender)) {
        sendResponse({ error: 'Unauthorized' });
        return;
      }
      extractThumbnailFromUrl(message.url)
        .then(result => sendResponse(result || { error: 'PrvImage not found' }))
        .catch(err => sendResponse({ error: err.message }));
      return true;
    }

    case 'get-settings': {
      browser.storage.local.get({
        autoOpen: true, showBadges: true, hoverPreview: true, disableExternalWebFonts: false,
        allowHttp: true, httpWarning: true, devMode: false,
        allowedDomains: DEFAULT_ALLOWED_DOMAINS, allSitesEnabled: false,
      }).then(s => sendResponse(s)).catch(() => sendResponse({
        autoOpen: true,
        showBadges: true,
        hoverPreview: true,
        disableExternalWebFonts: false,
      }));
      return true;
    }

    default:
      break;
  }
});

// ─── HWP 썸네일 추출 (Task #88, Chrome #86 포팅) ───

const THUMBNAIL_CACHE = new Map();
const CACHE_MAX_SIZE = 100;

async function extractThumbnailFromUrl(url) {
  if (THUMBNAIL_CACHE.has(url)) return THUMBNAIL_CACHE.get(url);
  try {
    const settings = await browser.storage.local.get({ devMode: false });
    const urlResult = validateUrl(url);
    if (!urlResult.valid) return null;
    if (urlResult.isPrivate && !settings.devMode) return null;

    const response = await fetch(url, { credentials: 'omit' });
    if (!response.ok) return null;
    const buffer = await response.arrayBuffer();
    const data = new Uint8Array(buffer);

    const isZip = data.length >= 4 && data[0] === 0x50 && data[1] === 0x4B;
    const result = isZip ? await extractPrvImageFromZip(data) : extractPrvImageFromCFB(data);
    if (result) {
      if (THUMBNAIL_CACHE.size >= CACHE_MAX_SIZE) {
        THUMBNAIL_CACHE.delete(THUMBNAIL_CACHE.keys().next().value);
      }
      THUMBNAIL_CACHE.set(url, result);
    }
    return result;
  } catch { return null; }
}

function extractPrvImageFromCFB(data) {
  if (data.length < 512 || data[0] !== 0xD0 || data[1] !== 0xCF || data[2] !== 0x11 || data[3] !== 0xE0) return null;
  const sectorSize = 1 << (data[30] | (data[31] << 8));
  const dirStartSector = _u32(data, 48);
  const dirOffset = (dirStartSector + 1) * sectorSize;
  for (let i = 0; i < 128; i++) {
    const eo = dirOffset + i * 128;
    if (eo + 128 > data.length) break;
    const nameLen = _u16(data, eo + 64);
    if (nameLen === 0 || nameLen > 64) continue;
    let name = '';
    for (let j = 0; j < nameLen - 2; j += 2) {
      const c = data[eo + j] | (data[eo + j + 1] << 8);
      if (c === 0) break;
      name += String.fromCharCode(c);
    }
    if (name !== 'PrvImage') continue;
    const startSector = _u32(data, eo + 116);
    const streamSize = _u32(data, eo + 120);
    if (streamSize === 0 || streamSize > 10 * 1024 * 1024) continue;
    const fatSectors = [];
    for (let j = 0; j < 109; j++) {
      const fs = _u32(data, 76 + j * 4);
      if (fs >= 0xFFFFFFFE) break;
      fatSectors.push(fs);
    }
    const fat = [];
    for (const fs of fatSectors) {
      const fo = (fs + 1) * sectorSize;
      for (let j = 0; j < sectorSize / 4; j++) {
        if (fo + j * 4 + 4 > data.length) break;
        fat.push(_u32(data, fo + j * 4));
      }
    }
    const result = new Uint8Array(streamSize);
    let sec = startSector, read = 0;
    for (let s = 0; s < 10000 && read < streamSize; s++) {
      if (sec >= 0xFFFFFFFE) break;
      const off = (sec + 1) * sectorSize;
      const len = Math.min(sectorSize, streamSize - read);
      if (off + len > data.length) break;
      result.set(data.subarray(off, off + len), read);
      read += len;
      sec = sec < fat.length ? fat[sec] : 0xFFFFFFFE;
    }
    if (read >= streamSize) return _parseImage(result);
  }
  return null;
}

async function extractPrvImageFromZip(data) {
  let eocd = -1;
  for (let i = data.length - 22; i >= 0 && i >= data.length - 65558; i--) {
    if (data[i] === 0x50 && data[i+1] === 0x4B && data[i+2] === 0x05 && data[i+3] === 0x06) { eocd = i; break; }
  }
  if (eocd < 0) return null;
  const cdOff = _u32(data, eocd + 16);
  const cdCount = _u16(data, eocd + 10);
  let off = cdOff;
  for (let i = 0; i < cdCount && off + 46 < data.length; i++) {
    if (data[off] !== 0x50 || data[off+1] !== 0x4B || data[off+2] !== 0x01 || data[off+3] !== 0x02) break;
    const comp = _u16(data, off + 10);
    const compSz = _u32(data, off + 20);
    const uncSz = _u32(data, off + 24);
    const nLen = _u16(data, off + 28);
    const eLen = _u16(data, off + 30);
    const cLen = _u16(data, off + 32);
    const locOff = _u32(data, off + 42);
    const name = new TextDecoder().decode(data.subarray(off + 46, off + 46 + nLen));
    if (name.startsWith('Preview/PrvImage')) {
      const lnLen = _u16(data, locOff + 26);
      const leLen = _u16(data, locOff + 28);
      const ds = locOff + 30 + lnLen + leLen;
      if (comp === 0) return _parseImage(data.subarray(ds, ds + uncSz));
      if (comp === 8) {
        try {
          const dec = new DecompressionStream('raw');
          const w = dec.writable.getWriter();
          w.write(data.slice(ds, ds + compSz));
          w.close();
          const r = dec.readable.getReader();
          const chunks = [];
          while (true) { const { done, value } = await r.read(); if (done) break; chunks.push(value); }
          const total = chunks.reduce((s, c) => s + c.length, 0);
          const buf = new Uint8Array(total);
          let o = 0;
          for (const c of chunks) { buf.set(c, o); o += c.length; }
          return _parseImage(buf);
        } catch { return null; }
      }
    }
    off += 46 + nLen + eLen + cLen;
  }
  return null;
}

function _parseImage(d) {
  let mime, w = 0, h = 0;
  if (d.length >= 8 && d[0] === 0x89 && d[1] === 0x50) { mime = 'image/png'; if (d.length >= 24) { w = (d[16]<<24)|(d[17]<<16)|(d[18]<<8)|d[19]; h = (d[20]<<24)|(d[21]<<16)|(d[22]<<8)|d[23]; } }
  else if (d.length >= 2 && d[0] === 0x42 && d[1] === 0x4D) { mime = 'image/bmp'; if (d.length >= 26) { w = _u32(d, 18); h = Math.abs(_u32(d, 22) | 0); } }
  else if (d.length >= 3 && d[0] === 0x47 && d[1] === 0x49) { mime = 'image/gif'; if (d.length >= 10) { w = _u16(d, 6); h = _u16(d, 8); } }
  else return null;
  let bin = '';
  for (let i = 0; i < d.length; i++) bin += String.fromCharCode(d[i]);
  return { dataUri: `data:${mime};base64,${btoa(bin)}`, width: w, height: h, mime };
}

function _u16(d, o) { return d[o] | (d[o+1] << 8); }
function _u32(d, o) { return (d[o] | (d[o+1] << 8) | (d[o+2] << 16) | (d[o+3] << 24)) >>> 0; }

// ─── 초기화 ───

browser.runtime.onInstalled.addListener((details) => {
  setupContextMenus();
  if (details.reason === 'install') {
    browser.storage.local.set({
      autoOpen: true, showBadges: true, hoverPreview: true, disableExternalWebFonts: false,
      allowHttp: true, httpWarning: true, devMode: false, securityLog: false,
      allowedDomains: DEFAULT_ALLOWED_DOMAINS, allSitesEnabled: false,
      maxFileSize: 20,
    });
  }
});

// 확장 아이콘 클릭 → 빈 뷰어 탭
browser.action.onClicked.addListener(() => {
  openViewer();
});
