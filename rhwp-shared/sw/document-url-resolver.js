// 브라우저 확장 공통 문서 URL 해석기
//
// "링크 URL"과 "실제 파일 바이트 URL"이 다른 서비스(provider)를
// 호출부에서 직접 분기하지 않도록 이 모듈에서 정규화한다.

const DOCUMENT_EXTENSION_RE = /\.(hwp|hwpx)$/i;

/**
 * rhwp가 직접 열 수 있는 문서 경로인지 확인한다.
 *
 * 쿼리 문자열의 file=sample.hwp 같은 위장 케이스를 피하기 위해
 * URL pathname 기준으로만 판단한다.
 *
 * @param {string} pathname
 * @returns {boolean}
 */
export function isDocumentPath(pathname) {
  if (typeof pathname !== 'string') return false;
  if (pathname.includes('?') || pathname.includes('#')) return false;
  try {
    return DOCUMENT_EXTENSION_RE.test(decodeURIComponent(pathname).toLowerCase());
  } catch {
    return DOCUMENT_EXTENSION_RE.test(pathname.toLowerCase());
  }
}

/**
 * GitHub blob URL을 raw 파일 URL로 변환한다.
 *
 * 지원 형태:
 *   https://github.com/{owner}/{repo}/blob/{ref}/{path}.hwp[x]
 *
 * 이번 단계에서는 GitHub URL 구조상 ref segment 1개를 우선 지원한다.
 * slash 포함 ref는 ambiguous 하므로 후속 provider 확장에서 다룬다.
 *
 * @param {URL} parsed
 * @returns {string|null}
 */
export function resolveGithubBlobUrl(parsed) {
  if (!(parsed instanceof URL)) return null;
  if (parsed.protocol !== 'https:') return null;
  if (parsed.hostname !== 'github.com') return null;

  const segments = parsed.pathname.split('/').filter(Boolean);
  if (segments.length < 5) return null;

  const [owner, repo, marker, ref, ...pathParts] = segments;
  if (!owner || !repo || marker !== 'blob' || !ref || pathParts.length === 0) {
    return null;
  }

  const encodedPath = pathParts.join('/');
  if (!isDocumentPath(encodedPath)) return null;

  return `https://raw.githubusercontent.com/${owner}/${repo}/${ref}/${encodedPath}`;
}

/**
 * 실제 문서 fetch에 사용할 URL을 해석한다.
 *
 * provider adapter가 매칭되지 않거나 URL 파싱에 실패하면 원본 문자열을
 * 그대로 반환한다. 호출부의 기존 동작을 보존하면서 알려진 파일 상세
 * 페이지 URL만 실제 파일 URL로 바꾼다.
 *
 * @param {string} url
 * @returns {string}
 */
export function resolveDocumentUrl(url) {
  if (!url || typeof url !== 'string') return url;

  let parsed;
  try {
    parsed = new URL(url);
  } catch {
    return url;
  }

  return resolveGithubBlobUrl(parsed) || url;
}
