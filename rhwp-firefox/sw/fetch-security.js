// Extension-side document fetch security policy.
//
// Content scripts can observe untrusted web pages. Any URL they pass to the
// service worker must be revalidated here before the extension performs a
// privileged fetch.

import { isDocumentPath, resolveDocumentUrl } from './document-url-resolver.js';

const MAX_REDIRECTS = 5;

const BLOCKED_HOST_SUFFIXES = [
  '.localhost',
  '.local',
  '.localdomain',
  '.internal',
  '.intranet',
  '.lan',
  '.home',
  '.corp'
];

export class FetchSecurityError extends Error {
  constructor(reason, message) {
    super(message || reason);
    this.name = 'FetchSecurityError';
    this.reason = reason;
  }
}

export function isTrustedExtensionPageSender(sender, runtimeApi, allowedPages = ['viewer.html']) {
  const senderUrl = sender?.url || sender?.tab?.url;
  if (!senderUrl || !runtimeApi?.runtime?.getURL) return false;

  try {
    const parsed = new URL(senderUrl);
    const extensionRoot = new URL(runtimeApi.runtime.getURL(''));
    if (parsed.origin !== extensionRoot.origin) return false;

    return allowedPages.some((page) => {
      const normalized = page.startsWith('/') ? page : `/${page}`;
      return parsed.pathname === normalized || parsed.pathname.endsWith(normalized);
    });
  } catch {
    return false;
  }
}

export function isWebPageSender(sender) {
  const senderUrl = sender?.url || sender?.tab?.url;
  if (!senderUrl || !sender?.tab?.id) return false;

  try {
    const parsed = new URL(senderUrl);
    return parsed.protocol === 'https:' || parsed.protocol === 'http:';
  } catch {
    return false;
  }
}

export function validateDocumentFetchUrl(url, options = {}) {
  const {
    allowHttp = true,
    requireDocumentPath = false
  } = options;

  if (!url || typeof url !== 'string') {
    throw new FetchSecurityError('invalid-url', 'URL이 비어 있습니다.');
  }

  let parsed;
  try {
    parsed = new URL(resolveDocumentUrl(url));
  } catch {
    throw new FetchSecurityError('invalid-url', 'URL 형식이 올바르지 않습니다.');
  }

  if (parsed.username || parsed.password) {
    throw new FetchSecurityError('userinfo-blocked', '사용자 정보가 포함된 URL은 차단됩니다.');
  }

  if (parsed.protocol !== 'https:' && !(allowHttp && parsed.protocol === 'http:')) {
    throw new FetchSecurityError('scheme-blocked', '허용되지 않은 URL scheme입니다.');
  }

  if (isBlockedHost(parsed.hostname)) {
    throw new FetchSecurityError('private-host-blocked', '로컬 또는 내부 네트워크 URL은 차단됩니다.');
  }

  if (requireDocumentPath && !isDocumentPath(parsed.pathname)) {
    throw new FetchSecurityError('not-document-path', '문서 파일 경로가 아닙니다.');
  }

  return parsed;
}

export async function fetchDocumentWithPolicy(url, options = {}) {
  let current = validateDocumentFetchUrl(url, options);

  for (let redirectCount = 0; redirectCount <= MAX_REDIRECTS; redirectCount++) {
    const response = await fetch(current.href, {
      ...options.fetchOptions,
      credentials: 'omit',
      redirect: 'manual'
    });

    if (!isRedirectStatus(response.status)) {
      return response;
    }

    const location = response.headers.get('Location');
    if (!location) {
      throw new FetchSecurityError('redirect-location-hidden', 'redirect 대상을 검증할 수 없습니다.');
    }

    current = validateDocumentFetchUrl(new URL(location, current.href).href, options);
  }

  throw new FetchSecurityError('too-many-redirects', 'redirect 횟수가 너무 많습니다.');
}

function isRedirectStatus(status) {
  return status === 301 || status === 302 || status === 303 || status === 307 || status === 308;
}

function isBlockedHost(hostname) {
  if (!hostname) return true;

  const host = hostname.toLowerCase().replace(/\.$/, '').replace(/^\[/, '').replace(/\]$/, '');

  if (host === 'localhost' || host === '0' || host === '0.0.0.0') return true;
  if (BLOCKED_HOST_SUFFIXES.some((suffix) => host.endsWith(suffix))) return true;

  // Single-label hosts are usually intranet names. Public document fetches
  // should use a fully qualified host.
  if (!host.includes('.') && !host.includes(':')) return true;

  if (isPrivateIPv4(host)) return true;
  if (isPrivateIPv6(host)) return true;

  return false;
}

function isPrivateIPv4(host) {
  if (!/^\d{1,3}(?:\.\d{1,3}){3}$/.test(host)) return false;

  const parts = host.split('.').map(Number);
  if (parts.some((part) => part < 0 || part > 255)) return true;

  const [a, b] = parts;
  return (
    a === 0 ||
    a === 10 ||
    a === 127 ||
    (a === 100 && b >= 64 && b <= 127) ||
    (a === 169 && b === 254) ||
    (a === 172 && b >= 16 && b <= 31) ||
    (a === 192 && b === 168) ||
    a >= 224
  );
}

function isPrivateIPv6(host) {
  if (!host.includes(':')) return false;

  // URL.hostname canonicalizes dotted mapped IPv4 into hexadecimal words.
  // Expand the address before classifying; textual prefix checks miss that form.
  const halves = host.toLowerCase().split('::');
  if (halves.length > 2) return true;
  const words = part => part ? part.split(':').map(word =>
    /^[0-9a-f]{1,4}$/.test(word) ? parseInt(word, 16) : NaN) : [];
  const left = words(halves[0]);
  const right = halves.length === 2 ? words(halves[1]) : [];
  const missing = 8 - left.length - right.length;
  if (left.concat(right).some(Number.isNaN) ||
      (halves.length === 2 ? missing < 1 : missing !== 0)) return true;
  const address = halves.length === 2
    ? [...left, ...Array(missing).fill(0), ...right] : left;

  if (address.slice(0, 5).every(word => word === 0) && address[5] === 0xffff) {
    const high = address[6];
    const low = address[7];
    return isPrivateIPv4(`${high >> 8}.${high & 255}.${low >> 8}.${low & 255}`);
  }
  // Unspecified, loopback and deprecated IPv4-compatible ::/96 addresses;
  // link-local fe80::/10, unique-local fc00::/7 and multicast ff00::/8.
  return address.slice(0, 6).every(word => word === 0) ||
    (address[0] & 0xffc0) === 0xfe80 ||
    (address[0] & 0xfe00) === 0xfc00 ||
    (address[0] & 0xff00) === 0xff00;
}
