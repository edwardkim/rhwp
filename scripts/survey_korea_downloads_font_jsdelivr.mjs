#!/usr/bin/env node

/**
 * korea_downloads HWP/HWPX 선언 글꼴과 jsDelivr 배포 가능 여부를 조사한다.
 *
 * 원시 NDJSON 또는 중간 목록 파일을 만들지 않는다. 현재 rhwp 바이너리의
 * `batch info --json` 스트림을 메모리에서 집계한 뒤, 사람이 읽는 Markdown 요약과
 * 전수 TSV만 지정한 경로에 직접 기록한다.
 *
 * 실행 예:
 *   cargo build --release
 *   node scripts/survey_korea_downloads_font_jsdelivr.mjs
 */

import { promises as fs } from 'node:fs';
import { dirname, relative, resolve } from 'node:path';
import { spawn, execFileSync } from 'node:child_process';
import { createInterface } from 'node:readline';

const REPOSITORY_ROOT = resolve(import.meta.dirname, '..');
const DEFAULT_RHWP = resolve(REPOSITORY_ROOT, 'target/release/rhwp');
const DEFAULT_REPORT = resolve(
  REPOSITORY_ROOT,
  'mydocs/report/survey_korea_downloads_font_jsdelivr_20260815.md',
);
const DEFAULT_DETAILS = resolve(
  REPOSITORY_ROOT,
  'mydocs/report/assets/survey_korea_downloads_font_jsdelivr_20260815.tsv',
);
const FONTSOURCE_CATALOG_URL = 'https://api.fontsource.org/v1/fonts';
const JSD_DELIVR_DATA_URL = 'https://data.jsdelivr.com/v1/package/npm';
const ONLINE_WEBFONTS_SEARCH_URL = 'https://www.onlinewebfonts.com/fonts/';
const ONLINE_WEBFONTS_DOWNLOAD_URL = 'https://www.onlinewebfonts.com/download/';
const ONLINE_WEBFONTS_CDN_URL = 'https://db.onlinewebfonts.com/t/';
const NOONNU_SEARCH_URL = 'https://noonnu.cc/search/';
const NOONNU_FONT_PAGE_URL = 'https://noonnu.cc/font_page/';
const GOOGLE_FONTS_CSS_URL = 'https://fonts.googleapis.com/css2';
const CDN_ROOT = 'https://cdn.jsdelivr.net';
const REQUEST_TIMEOUT_MS = 20_000;
// jsDelivr 웹의 `?query=` 화면이 쓰는 공개 Algolia npm 검색 인덱스다.
// 검색 결과만으로 배포 여부를 판단하지 않고, 아래에서 Data API와 CDN 파일을 검증한다.
const JSDELIVR_SEARCH_URL = 'https://OFCNCOG2CU-dsn.algolia.net/1/indexes/npm-search/query';
const JSDELIVR_SEARCH_APP_ID = 'OFCNCOG2CU';
const JSDELIVR_SEARCH_API_KEY = 'f54e21fa3a2a0160595bb058179bfb1e';
const PACKAGE_SEARCH_INTERVAL_MS = 125;
const PACKAGE_SEARCH_MAX_RETRIES = 4;
const MAX_JSDELIVR_CANDIDATES = 3;
const ONLINE_WEBFONTS_REQUEST_INTERVAL_MS = 250;
const NOONNU_REQUEST_INTERVAL_MS = 250;
const GOOGLE_FONTS_REQUEST_INTERVAL_MS = 250;
let nextPackageSearchAt = 0;
let nextOnlineWebFontsAt = 0;
let nextNoonnuAt = 0;
let nextGoogleFontsAt = 0;
const jsDelivrSearchCache = new Map();
const onlineWebFontsSearchCache = new Map();
const noonnuSearchCache = new Map();
const googleFontsSearchCache = new Map();
const progressStartedAt = Date.now();
let progressPhase = '초기화';

function setProgressPhase(phase) {
  progressPhase = phase;
  console.log(`[단계] ${phase}`);
}

const progressHeartbeat = setInterval(() => {
  const elapsedSeconds = Math.floor((Date.now() - progressStartedAt) / 1_000);
  console.log(`[진행] ${progressPhase} (${elapsedSeconds}초 경과)`);
}, 30_000);
// 작업이 끝난 뒤에는 이벤트 루프를 붙잡지 않는다.
progressHeartbeat.unref();

function usage() {
  return `사용법: node scripts/survey_korea_downloads_font_jsdelivr.mjs [옵션]

옵션:
  --input <경로>        HWP/HWPX 파일 하나 또는 코퍼스 디렉터리 (필수)
  --rhwp <경로>         devel rhwp 실행 파일 (기본: ${DEFAULT_RHWP})
  --report <경로>       Markdown 보고서 (기본: ${DEFAULT_REPORT})
  --details <경로>      전수 TSV (기본: ${DEFAULT_DETAILS})
  --threads <수>        rhwp batch 파싱 병렬도 (기본: 8)
  --concurrency <수>    네트워크 조사 병렬도 (기본: 8)
  --no-jsdelivr-search  jsDelivr npm 웹 검색만 생략 (Google Fonts 등 다른 공급 경로는 확인)
  --help                이 도움말 표시
`;
}

function parseArgs(argv) {
  const options = {
    inputRoot: null,
    rhwp: DEFAULT_RHWP,
    report: DEFAULT_REPORT,
    details: DEFAULT_DETAILS,
    threads: 8,
    concurrency: 8,
    npmSearch: true,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--help') {
      console.log(usage());
      process.exit(0);
    }
    if (arg === '--no-jsdelivr-search' || arg === '--no-npm-search') {
      options.npmSearch = false;
      continue;
    }
    const rawKey = arg === '--input' ? '--input-root' : arg;
    const key = rawKey.slice(2).replace(/-([a-z])/g, (_, char) => char.toUpperCase());
    if (!arg.startsWith('--') || !(key in options)) {
      throw new Error(`알 수 없는 옵션: ${arg}`);
    }
    const value = argv[index + 1];
    if (!value || value.startsWith('--')) {
      throw new Error(`${arg} 값이 필요합니다.`);
    }
    index += 1;
    if (key === 'threads' || key === 'concurrency') {
      const parsed = Number.parseInt(value, 10);
      if (!Number.isInteger(parsed) || parsed < 1 || parsed > 32) {
        throw new Error(`${arg}는 1부터 32 사이 정수여야 합니다.`);
      }
      options[key] = parsed;
    } else {
      options[key] = resolve(value);
    }
  }
  return options;
}

function normalized(value) {
  return value
    .replace(/\u0000/g, '')
    .normalize('NFC')
    .trim()
    .replace(/\s+/g, ' ')
    .toLocaleLowerCase('en-US');
}

function declaredFontName(value) {
  let name = String(value)
    .replace(/\u0000/g, '')
    .normalize('NFC')
    .trim()
    .replace(/\\([0-9a-f]{1,6})\s?/giu, (_, codepoint) => String.fromCodePoint(Number.parseInt(codepoint, 16)))
    .replace(/\\,/gu, ',')
    .replace(/&quot;?/giu, '"');
  if (/^[\u0000-\u00ff]+$/u.test(name) && /[\u0080-\u00ff]/u.test(name)) {
    try {
      const recovered = new TextDecoder('euc-kr', { fatal: true }).decode(
        Uint8Array.from(name, character => character.codePointAt(0)),
      );
      if (/\p{L}|\p{N}/u.test(recovered)) name = recovered;
    } catch {
      // CP949 바이트열로 되돌릴 수 없는 이름은 원문을 보존한다.
    }
  }
  name = name
    .replace(/^\((?:한|환)\)\s*/u, '')
    .replace(/^[\p{P}\p{S}]+/u, '')
    .replace(/^['"]+|['"]+$/gu, '')
    .split(',', 1)[0]
    .trim()
    .replace(/(?:[-_ ]+identity-[hv])$/iu, '');
  return name;
}

function compact(value) {
  return normalized(value).replace(/[\s._'"()\[\]{}\-]/g, '');
}

function familyKey(value) {
  let key = normalized(declaredFontName(value));
  key = key.replace(
    /\s+(thin|extralight|extra light|light|regular|medium|semibold|semi bold|bold|extrabold|extra bold|black|italic|oblique|book|demi|m|l|r|b)$/u,
    '',
  );
  return compact(key);
}

function tsv(value) {
  return String(value ?? '').replace(/[\t\r\n]+/g, ' ').trim();
}

function markdownCell(value) {
  return String(value ?? '')
    .replace(/\\/g, '\\\\')
    .replace(/\|/g, '\\|')
    .replace(/[\r\n]+/g, ' ');
}

function webfontAvailability(row) {
  if (row.status === 'lookup-error') {
    return { download: '확인 불가', usable: '확인 불가', reason: '공급 경로 조회 오류' };
  }
  if (!row.url) {
    return { download: '불가', usable: '불가', reason: '검증 가능한 웹폰트 파일 미발견' };
  }
  if (row.status === 'license-review') {
    return { download: '가능', usable: '라이선스 검토 필요', reason: 'CDN 응답은 확인했지만 원 권리자의 웹 사용 허가를 확인하지 못함' };
  }
  if (row.delivery === 'Noonnu CDN') {
    return { download: '가능', usable: '가능', reason: 'Noonnu 상세 페이지의 웹사이트 사용 가능 요약과 CDN 응답 확인' };
  }
  if (/미확인/u.test(row.license)) {
    return { download: '가능', usable: '라이선스 검토 필요', reason: 'CDN 응답은 확인했지만 패키지 라이선스 근거가 부족함' };
  }
  return { download: '가능', usable: '가능', reason: '공급 경로의 라이선스 단서와 CDN 응답 확인' };
}

async function walkDocuments(root) {
  const files = [];
  let directories = 0;
  let entriesSeen = 0;
  console.log(`[입력 목록화] 시작: ${root}`);
  async function visit(directory) {
    directories += 1;
    console.log(`[입력 목록화] 디렉터리 ${directories} 진입: ${directory}`);
    const childDirectories = [];
    let directoryEntries = 0;
    const directoryHandle = await fs.opendir(directory);
    for await (const entry of directoryHandle) {
      directoryEntries += 1;
      entriesSeen += 1;
      const path = resolve(directory, entry.name);
      const type = entry.isDirectory() ? '디렉터리' : entry.isFile() ? '파일' : '기타';
      console.log(`[입력 목록화] 항목 ${entriesSeen} (${type}): ${path}`);
      if (entry.isDirectory()) {
        childDirectories.push(path);
      } else if (entry.isFile() && /\.hwp(x)?$/iu.test(entry.name)) {
        files.push(path);
        console.log(`[입력 목록화] HWP/HWPX ${files.length}건 수집: ${path}`);
      }
    }
    console.log(
      `[입력 목록화] 디렉터리 ${directories} 읽기 완료: 항목 ${directoryEntries}건, 누적 항목 ${entriesSeen}건`,
    );
    for (const childDirectory of childDirectories) await visit(childDirectory);
  }
  await visit(root);
  console.log(
    `[입력 목록화] 완료: 디렉터리 ${directories}건, 항목 ${entriesSeen}건, HWP/HWPX ${files.length}건`,
  );
  return files.sort((left, right) => left.localeCompare(right, 'ko'));
}

async function inputDocuments(input) {
  const entry = await fs.stat(input);
  if (entry.isFile()) {
    if (!/\.hwp(x)?$/iu.test(input)) {
      throw new Error(`입력 파일은 .hwp 또는 .hwpx여야 합니다: ${input}`);
    }
    console.log(`[입력 목록화] 단일 HWP/HWPX 입력: ${input}`);
    return [input];
  }
  if (entry.isDirectory()) return walkDocuments(input);
  throw new Error(`입력은 HWP/HWPX 파일 또는 디렉터리여야 합니다: ${input}`);
}

function failureKind(message) {
  if (/빈 파일/u.test(message)) return '빈 파일';
  if (/비밀번호/u.test(message)) return '암호 문서';
  if (/DRM/u.test(message)) return 'DRM 보호';
  if (/알 수 없는 파일 형식/u.test(message)) return '미지원 형식';
  return '기타 파싱 실패';
}

async function collectDeclaredFonts({ rhwp, threads, documents }) {
  const child = spawn(rhwp, ['batch', 'info', '--json', '--threads', String(threads)], {
    cwd: REPOSITORY_ROOT,
    stdio: ['pipe', 'pipe', 'pipe'],
  });
  const fontDocumentCounts = new Map();
  const failures = new Map();
  const stderrTail = [];
  let records = 0;
  let parsedDocuments = 0;

  const stderr = createInterface({ input: child.stderr, crlfDelay: Infinity });
  const stderrPump = (async () => {
    for await (const line of stderr) {
      if (stderrTail.length === 12) stderrTail.shift();
      stderrTail.push(line);
    }
  })();

  const stdout = createInterface({ input: child.stdout, crlfDelay: Infinity });
  const stdoutPump = (async () => {
    for await (const line of stdout) {
      if (!line.trim()) continue;
      records += 1;
      let record;
      try {
        record = JSON.parse(line);
      } catch (error) {
        throw new Error(`batch info 출력이 NDJSON이 아닙니다: ${error.message}`);
      }
      if (record.error) {
        const kind = failureKind(String(record.error));
        failures.set(kind, (failures.get(kind) ?? 0) + 1);
        continue;
      }
      parsedDocuments += 1;
      const documentFonts = new Set(
        (Array.isArray(record.fonts) ? record.fonts : [])
          .map(declaredFontName)
          .filter(Boolean),
      );
      for (const font of documentFonts) {
        fontDocumentCounts.set(font, (fontDocumentCounts.get(font) ?? 0) + 1);
      }
    }
  })();

  for (const path of documents) child.stdin.write(`${path}\n`);
  child.stdin.end();

  const exitCode = await new Promise((resolveExit, rejectExit) => {
    child.once('error', rejectExit);
    child.once('close', resolveExit);
  });
  await Promise.all([stdoutPump, stderrPump]);
  if (records !== documents.length) {
    throw new Error(`batch info 레코드 수 불일치: 입력 ${documents.length}, 출력 ${records}`);
  }
  return { fontDocumentCounts, failures, parsedDocuments, records, exitCode, stderrTail };
}

async function fetchWithTimeout(url, init = {}) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);
  try {
    return await fetch(url, {
      ...init,
      signal: controller.signal,
      headers: {
        'user-agent': 'rhwp-korea-font-jsdelivr-survey/1.0',
        ...(init.headers ?? {}),
      },
    });
  } finally {
    clearTimeout(timer);
  }
}

async function fetchJson(url) {
  const response = await fetchWithTimeout(url);
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return response.json();
}

async function fetchText(url) {
  const response = await fetchWithTimeout(url);
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return response.text();
}

function sleep(milliseconds) {
  return new Promise(resolveSleep => setTimeout(resolveSleep, milliseconds));
}

async function waitForPackageSearchSlot() {
  const now = Date.now();
  const slot = Math.max(now, nextPackageSearchAt);
  nextPackageSearchAt = slot + PACKAGE_SEARCH_INTERVAL_MS;
  if (slot > now) await sleep(slot - now);
}

async function waitForOnlineWebFontsSlot() {
  const now = Date.now();
  const slot = Math.max(now, nextOnlineWebFontsAt);
  nextOnlineWebFontsAt = slot + ONLINE_WEBFONTS_REQUEST_INTERVAL_MS;
  if (slot > now) await sleep(slot - now);
}

async function waitForNoonnuSlot() {
  const now = Date.now();
  const slot = Math.max(now, nextNoonnuAt);
  nextNoonnuAt = slot + NOONNU_REQUEST_INTERVAL_MS;
  if (slot > now) await sleep(slot - now);
}

async function waitForGoogleFontsSlot() {
  const now = Date.now();
  const slot = Math.max(now, nextGoogleFontsAt);
  nextGoogleFontsAt = slot + GOOGLE_FONTS_REQUEST_INTERVAL_MS;
  if (slot > now) await sleep(slot - now);
}

function retryAfterMilliseconds(response, attempt) {
  const value = response.headers.get('retry-after');
  const seconds = Number.parseFloat(value ?? '');
  if (Number.isFinite(seconds) && seconds > 0) return Math.ceil(seconds * 1000);
  return 1_000 * (attempt + 1);
}

async function searchJsDelivrPackages(query) {
  const cacheKey = normalized(query);
  const cached = jsDelivrSearchCache.get(cacheKey);
  if (cached) return cached;
  const request = (async () => {
    for (let attempt = 0; attempt < PACKAGE_SEARCH_MAX_RETRIES; attempt += 1) {
      await waitForPackageSearchSlot();
      const response = await fetchWithTimeout(JSDELIVR_SEARCH_URL, {
        method: 'POST',
        headers: {
          'content-type': 'application/json',
          'x-algolia-application-id': JSDELIVR_SEARCH_APP_ID,
          'x-algolia-api-key': JSDELIVR_SEARCH_API_KEY,
        },
        body: JSON.stringify({ query, hitsPerPage: 20, page: 0 }),
      });
      if (response.ok) return response.json();
      if (response.status !== 429) throw new Error(`HTTP ${response.status}`);
      const delay = retryAfterMilliseconds(response, attempt);
      nextPackageSearchAt = Math.max(nextPackageSearchAt, Date.now() + delay);
    }
    throw new Error(`HTTP 429 재시도 ${PACKAGE_SEARCH_MAX_RETRIES}회 초과`);
  })();
  jsDelivrSearchCache.set(cacheKey, request);
  try {
    return await request;
  } catch (error) {
    jsDelivrSearchCache.delete(cacheKey);
    throw error;
  }
}

async function confirmsDownload(url) {
  let response = await fetchWithTimeout(url, { method: 'HEAD' });
  if (response.ok) return true;
  if (![405, 501].includes(response.status)) return false;
  response = await fetchWithTimeout(url, { headers: { Range: 'bytes=0-0' } });
  return response.ok;
}

async function pool(items, concurrency, task) {
  const results = new Array(items.length);
  let nextIndex = 0;
  async function worker() {
    while (true) {
      const index = nextIndex;
      nextIndex += 1;
      if (index >= items.length) return;
      results[index] = await task(items[index], index);
    }
  }
  await Promise.all(Array.from({ length: Math.min(concurrency, items.length) }, worker));
  return results;
}

function knownKoPubCdn(font) {
  const match = compact(font).match(/^kopub(dotum|batang|돋움체|바탕체)(light|medium|bold)$/u);
  if (!match) return null;

  const family = match[1] === 'dotum' || match[1] === '돋움체' ? 'Dotum' : 'Batang';
  const weight = `${match[2][0].toUpperCase()}${match[2].slice(1)}`;
  return {
    packageType: 'npm',
    packageName: 'font-kopub',
    version: '1.0.2',
    file: `fonts/KoPub${family}-${weight}.woff`,
    license: 'KOPUS-Custom (패키지 메타데이터 표기)',
    delivery: 'jsDelivr npm',
    note: 'KoPub 돋움·바탕의 요청 굵기와 일치하는 font-kopub WOFF 확인',
  };
}

function knownKoPubWorldCdn(font) {
  const match = compact(font).match(/^kopubworld(dotum|batang|돋움체|바탕체)(light|medium|bold)$/u);
  if (!match) return null;

  const family = match[1] === 'dotum' || match[1] === '돋움체' ? 'Dotum' : 'Batang';
  const weight = `${match[2][0].toUpperCase()}${match[2].slice(1)}`;
  return {
    packageType: 'npm',
    packageName: 'font-kopubworld',
    version: '1.0.3',
    file: `fonts/KoPubWorld-${family}-${weight}.otf`,
    license: 'KOPUS-Custom (사용 등록 후 상업적·온라인 사용 가능)',
    delivery: 'jsDelivr npm',
    note: 'KOPUS 공식 공개 글꼴 안내에 따른 KoPubWorld 요청 굵기 일치 OTF 확인',
  };
}

function knownGovernmentSymbolCdn(font) {
  const normalized = String(font)
    .normalize('NFKC')
    .toLocaleLowerCase('en-US')
    .replace(/\.(?:ttf|otf|woff2?)$/u, '')
    .replace(/[\s_.-]+/gu, '');
  if (!new Set(['government16040911', '정부상징부처명16040911']).has(normalized)) return null;

  return {
    packageType: 'github',
    owner: 'jangster77',
    repo: 'korea-government-symbol-font',
    ref: 'v1.0.0',
    file: 'fonts/Government_16040911.ttf',
    license: '공공누리 제4유형 (출처표시+상업적 이용금지+변경금지)',
    delivery: 'jsDelivr GitHub',
    note: '문화체육관광부 대한민국정부상징서체 원본 TTF의 고정 태그 CDN 확인',
  };
}

function knownCdn(font) {
  const koPub = knownKoPubCdn(font);
  if (koPub) return koPub;
  const koPubWorld = knownKoPubWorldCdn(font);
  if (koPubWorld) return koPubWorld;
  const governmentSymbol = knownGovernmentSymbolCdn(font);
  if (governmentSymbol) return governmentSymbol;

  const key = familyKey(font);
  const batang = new Set(['함초롬바탕', '함초롱바탕', '한컴바탕', '새바탕'].map(familyKey));
  const dotum = new Set(['함초롬돋움', '함초롱돋움', '한컴돋움', '한컴산뜻돋움', '새돋움'].map(familyKey));
  if (batang.has(key)) {
    return {
      packageName: 'projectnoonnu/noonfonts_2104',
      version: '1.0',
      file: 'HANBatang.woff',
      license: '한컴 라이선스(비상업적 사용 허용 문구는 font-loader.ts 참조)',
    };
  }
  if (dotum.has(key)) {
    return {
      packageName: 'projectnoonnu/noonfonts_four',
      version: '1.0',
      file: 'HCRDotum.woff',
      license: '한컴 라이선스(비상업적 사용 허용 문구는 font-loader.ts 참조)',
    };
  }
  return null;
}

const PACKAGE_ALIASES = new Map([
  ['나눔고딕', '@fontsource/nanum-gothic'],
  ['나눔명조', '@fontsource/nanum-myeongjo'],
  ['나눔고딕코딩', '@fontsource/nanum-gothic-coding'],
  ['고운바탕', '@fontsource/gowun-batang'],
  ['고운돋움', '@fontsource/gowun-dodum'],
  ['본고딕', '@fontsource/noto-sans-kr'],
  ['본명조', '@fontsource/noto-serif-kr'],
  ['pretendard', 'pretendard'],
  ['d2coding', 'd2coding'],
  ['디투코딩', 'd2coding'],
  ['spoqa hansans', 'spoqa-han-sans'],
  ['spoqa han sans', 'spoqa-han-sans'],
  ['스포카한산스', 'spoqa-han-sans'],
].map(([font, packageName]) => [familyKey(font), packageName]));

function fontsourceCandidate(font, catalog) {
  const key = familyKey(font);
  const alias = PACKAGE_ALIASES.get(key);
  if (alias?.startsWith('@fontsource/')) return catalog.byId.get(alias.slice('@fontsource/'.length));
  return catalog.byFamily.get(key) ?? null;
}

async function getFontsourceCatalog() {
  const rows = await fetchJson(FONTSOURCE_CATALOG_URL);
  const byFamily = new Map();
  const byId = new Map();
  for (const row of rows) {
    if (!row?.id || !row?.family) continue;
    const value = { id: row.id, family: row.family, license: row.license ?? '미확인' };
    byId.set(row.id, value);
    byFamily.set(familyKey(row.family), value);
  }
  return { byFamily, byId, count: rows.length };
}

function onlineWebFontsFamily(font) {
  return normalized(declaredFontName(font))
    .replace(/^a\d{3}\s*/iu, '')
    .replace(/[_-]+/gu, ' ')
    .replace(
    /\s+(thin|extralight|extra light|light|regular|medium|semibold|semi bold|bold|extrabold|extra bold|black|italic|oblique|book|demi|m|l|r|b)$/u,
    '',
    )
    .replace(/[\p{Script=Hangul}](?:m|l|r|b)$/u, match => match.slice(0, -1));
}

function onlineWebFontsSearchPath(font) {
  const family = onlineWebFontsFamily(font).replace(/\s+/g, '_');
  return `${ONLINE_WEBFONTS_SEARCH_URL}${encodeURIComponent(family)}`;
}

function googleFontsCssPath(font) {
  const family = onlineWebFontsFamily(font);
  return `${GOOGLE_FONTS_CSS_URL}?family=${encodeURIComponent(family)}&display=swap`;
}

async function googleFontsCandidate(font) {
  const family = onlineWebFontsFamily(font);
  if (!family) return null;
  const cacheKey = normalized(family);
  const cached = googleFontsSearchCache.get(cacheKey);
  if (cached) return cached;
  const request = (async () => {
    const cssUrl = googleFontsCssPath(family);
    console.log(`[Google Fonts] CSS 요청: ${family}`);
    await waitForGoogleFontsSlot();
    const response = await fetchWithTimeout(cssUrl, {
      headers: {
        'user-agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36',
      },
    });
    if ([400, 404].includes(response.status)) {
      console.log(`[Google Fonts] 제공하지 않는 family: ${family}`);
      return null;
    }
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const css = await response.text();
    const families = [...css.matchAll(/font-family:\s*['"]([^'"]+)['"]/giu)]
      .map(match => match[1]);
    const expected = familyKey(family);
    if (!families.some(candidate => familyKey(candidate) === expected)) {
      console.log(`[Google Fonts] family 불일치: ${family}`);
      return null;
    }
    const urls = [...new Set(
      [...css.matchAll(/url\((https:\/\/fonts\.gstatic\.com\/[^)]+)\)/giu)]
        .map(match => match[1]),
    )];
    for (const url of urls) {
      console.log(`[Google Fonts] WOFF2 응답 확인: ${url}`);
      if (!(await confirmsDownload(url))) continue;
      return {
        packageName: `google-fonts:${family}`,
        version: 'Google Fonts CSS API',
        file: url.split('/').at(-1).split('?')[0],
        url,
        delivery: 'Google Fonts CSS API',
        license: 'Google Fonts 오픈소스 라이선스',
        note: `Google Fonts 공식 CSS API와 fonts.gstatic.com 응답 확인 (${cssUrl})`,
      };
    }
    console.log(`[Google Fonts] WOFF2 파일 응답 미확인: ${family}`);
    return null;
  })();
  googleFontsSearchCache.set(cacheKey, request);
  try {
    return await request;
  } catch (error) {
    googleFontsSearchCache.delete(cacheKey);
    throw error;
  }
}

function decodeHtml(value) {
  const entities = {
    '&amp;': '&',
    '&quot;': '"',
    '&#39;': "'",
    '&lt;': '<',
    '&gt;': '>',
  };
  return value
    .replace(/&(amp|quot|#39|lt|gt);/giu, entity => entities[entity.toLocaleLowerCase('en-US')])
    .trim();
}

function onlineWebFontsSearchResults(html) {
  const results = [];
  const pattern = /<a href="(?:https:\/\/www\.onlinewebfonts\.com)?\/download\/([a-f0-9]{32})" class="a">([^<]+)<\/a>/giu;
  for (const match of html.matchAll(pattern)) {
    results.push({ hash: match[1], title: decodeHtml(match[2]) });
  }
  return results;
}

async function onlineWebFontsCandidates(font) {
  const family = onlineWebFontsFamily(font);
  const cacheKey = normalized(family);
  const cached = onlineWebFontsSearchCache.get(cacheKey);
  if (cached) return cached;
  const request = (async () => {
    console.log(`[OnlineWebFonts] 검색 요청: ${family}`);
    await waitForOnlineWebFontsSlot();
    const html = await fetchText(onlineWebFontsSearchPath(family));
    const expected = familyKey(family);
    const matches = onlineWebFontsSearchResults(html)
      .filter(candidate => familyKey(candidate.title) === expected);
    console.log(`[OnlineWebFonts] 검색 응답: ${family}, 정확 일치 ${matches.length}건`);
    return matches;
  })();
  onlineWebFontsSearchCache.set(cacheKey, request);
  try {
    return await request;
  } catch (error) {
    onlineWebFontsSearchCache.delete(cacheKey);
    throw error;
  }
}

async function onlineWebFontsCandidate(font) {
  const candidates = await onlineWebFontsCandidates(font);
  for (const candidate of candidates) {
    const pageUrl = `${ONLINE_WEBFONTS_DOWNLOAD_URL}${candidate.hash}`;
    console.log(`[OnlineWebFonts] 글꼴 페이지 확인: ${candidate.title} (${candidate.hash})`);
    await waitForOnlineWebFontsSlot();
    const page = await fetchText(pageUrl);
    const woff2 = page.match(/https:\/\/db\.onlinewebfonts\.com\/t\/([a-f0-9]{32})\.woff2/iu);
    if (!woff2) continue;
    const url = `${ONLINE_WEBFONTS_CDN_URL}${woff2[1]}.woff2`;
    console.log(`[OnlineWebFonts] WOFF2 응답 확인: ${url}`);
    if (!(await confirmsDownload(url))) continue;
    return {
      packageName: `onlinewebfonts:${candidate.hash}`,
      version: candidate.hash,
      file: `${woff2[1]}.woff2`,
      url,
      delivery: 'OnlineWebFonts',
      license: '원 권리자 웹 사용 허가 미확인',
      note: `OnlineWebFonts WOFF2 응답 확인 (${pageUrl}); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요`,
    };
  }
  return null;
}

function htmlText(html) {
  return decodeHtml(html.replace(/<[^>]*>/giu, ' ').replace(/\s+/gu, ' '));
}

async function noonnuPageIds(font) {
  const family = onlineWebFontsFamily(font);
  const cacheKey = normalized(family);
  const cached = noonnuSearchCache.get(cacheKey);
  if (cached) return cached;
  const request = (async () => {
    console.log(`[Noonnu] 검색 요청: ${family}`);
    await waitForNoonnuSlot();
    const html = await fetchText(`${NOONNU_SEARCH_URL}${encodeURIComponent(family)}`);
    const pageIds = [...new Set(
      [...html.matchAll(/href="\/font_page\/(\d+)"/giu)].map(match => match[1]),
    )];
    console.log(`[Noonnu] 검색 응답: ${family}, 글꼴 페이지 ${pageIds.length}건`);
    return pageIds;
  })();
  noonnuSearchCache.set(cacheKey, request);
  try {
    return await request;
  } catch (error) {
    noonnuSearchCache.delete(cacheKey);
    throw error;
  }
}

async function noonnuCandidate(font) {
  const expected = familyKey(onlineWebFontsFamily(font));
  for (const pageId of await noonnuPageIds(font)) {
    const pageUrl = `${NOONNU_FONT_PAGE_URL}${pageId}`;
    console.log(`[Noonnu] 글꼴 페이지 확인: ${pageUrl}`);
    await waitForNoonnuSlot();
    const page = await fetchText(pageUrl);
    const title = page.match(/<h2[^>]*>\s*([^<]+?)\s*<\/h2>/iu);
    if (!title || familyKey(decodeHtml(title[1])) !== expected) continue;
    const licenseText = htmlText(page);
    if (!/웹사이트[\s\S]{0,400}사용 가능/iu.test(licenseText)) {
      console.log(`[Noonnu] 웹사이트 사용 허가 미확인: ${decodeHtml(title[1])}`);
      continue;
    }
    const source = decodeHtml(page).match(/src:\s*url\(['"]([^'"]+\.woff2?[^'"]*)['"]\)/iu);
    if (!source) continue;
    const url = source[1];
    console.log(`[Noonnu] 웹폰트 CDN 응답 확인: ${url}`);
    if (!(await confirmsDownload(url))) continue;
    return {
      packageName: `noonnu:font_page/${pageId}`,
      version: pageId,
      file: url.split('/').at(-1).split('?')[0],
      url,
      delivery: 'Noonnu CDN',
      license: 'Noonnu 요약표: 웹사이트 사용 가능',
      note: `Noonnu 상세 페이지에서 웹사이트 사용 허가와 CDN 응답 확인 (${pageUrl})`,
    };
  }
  return null;
}

async function finishUnmatchedFont(font, documentCount, attempts, status, note) {
  try {
    const googleFonts = await googleFontsCandidate(font);
    if (googleFonts) return { font, documentCount, status: 'available', ...googleFonts };
  } catch (error) {
    note = `${note}; Google Fonts 조회 실패: ${error.message}`;
  }
  try {
    const noonnu = await noonnuCandidate(font);
    if (noonnu) return { font, documentCount, status: 'available', ...noonnu };
  } catch (error) {
    note = `${note}; Noonnu 조회 실패: ${error.message}`;
  }
  try {
    const onlineWebFonts = await onlineWebFontsCandidate(font);
    if (onlineWebFonts) return { font, documentCount, status: 'license-review', ...onlineWebFonts };
  } catch (error) {
    note = `${note}; OnlineWebFonts 조회 실패: ${error.message}`;
  }
  return {
    font,
    documentCount,
    status,
    delivery: '',
    packageName: attempts.join(', '),
    version: '',
    license: '',
    url: '',
    note,
  };
}

function jsDelivrPackagePath(packageName) {
  return encodeURIComponent(packageName);
}

function requestedFontFileTerms(font) {
  const terms = new Set(
    normalized(font)
      .match(/[a-z0-9]+/giu)
      ?.filter(token => token.length >= 3) ?? [],
  );
  const key = familyKey(font);
  if (key.includes('돋움')) terms.add('dotum');
  if (key.includes('바탕')) terms.add('batang');
  return [...terms];
}

function requestedFontWeight(font) {
  const value = normalized(font);
  if (/\bbold\b/u.test(value)) return 'bold';
  if (/\bmedium\b/u.test(value)) return 'medium';
  if (/\blight\b/u.test(value)) return 'light';
  return null;
}

function matchingPackageFontFile(files, font) {
  const terms = requestedFontFileTerms(font);
  if (terms.length === 0) return null;
  const weight = requestedFontWeight(font);
  return files
    .map(file => String(file.name ?? '').replace(/^\//, ''))
    .filter(file => /\.(woff2?|ttf|otf)$/iu.test(file))
    .filter(file => {
      const key = compact(file);
      return terms.every(term => key.includes(compact(term)));
    })
    .filter(file => !weight || compact(file).includes(weight))
    .sort((left, right) => left.localeCompare(right, 'en'))[0] ?? null;
}

async function packageFontFile(
  packageName,
  source,
  licenseHint = '미확인',
  expectedFont = null,
  requireFilenameMatch = false,
) {
  const packagePath = jsDelivrPackagePath(packageName);
  const packageInfo = await fetchJson(`${JSD_DELIVR_DATA_URL}/${packagePath}`);
  const version = packageInfo?.tags?.latest;
  if (!version) return null;
  const flat = await fetchJson(
    `${JSD_DELIVR_DATA_URL}/${packagePath}@${encodeURIComponent(version)}/flat`,
  );
  const files = Array.isArray(flat?.files) ? flat.files : [];
  const anyFontFile = files
    .map(file => String(file.name ?? '').replace(/^\//, ''))
    .find(file => /\.(woff2?|ttf|otf)$/iu.test(file));
  const fontFile = requireFilenameMatch
    ? matchingPackageFontFile(files, expectedFont)
    : anyFontFile;
  if (!fontFile) return null;
  const url = `${CDN_ROOT}/npm/${packageName}@${version}/${fontFile}`;
  if (!(await confirmsDownload(url))) return null;
  return { packageName, version, file: fontFile, url, delivery: source, license: licenseHint };
}

function jsDelivrSearchQueries(font) {
  const family = normalized(font).replace(
    /\s+(thin|extralight|extra light|light|regular|medium|semibold|semi bold|bold|extrabold|extra bold|black|italic|oblique|book|demi|m|l|r|b)$/u,
    '',
  );
  const queries = new Set([font, family]);
  for (const token of family.match(/[a-z][a-z0-9._-]*/giu) ?? []) {
    if (token.length >= 4) queries.add(token);
  }
  return [...queries].filter(query => query.trim()).slice(0, 4);
}

function jsDelivrCandidateMatches(font, query, packageInfo) {
  const family = familyKey(font);
  const packageName = String(packageInfo?.name ?? '');
  const text = [
    packageName,
    packageInfo?.description ?? '',
    ...(Array.isArray(packageInfo?.keywords) ? packageInfo.keywords : []),
  ].join(' ');
  const searchable = familyKey(text);
  if (family.length >= 3 && searchable.includes(family)) return true;
  const asciiFamily = normalized(query).replace(/[^a-z0-9]+/g, '');
  const asciiPackage = normalized(packageName).replace(/[^a-z0-9]+/g, '');
  const queryKey = compact(query);
  if (queryKey.length >= 4 && asciiPackage.includes(queryKey)) return true;
  return asciiFamily.length >= 4 && asciiPackage.includes(asciiFamily);
}

async function jsDelivrCandidates(font) {
  const candidates = new Map();
  for (const query of jsDelivrSearchQueries(font)) {
    const body = await searchJsDelivrPackages(query);
    for (const candidate of Array.isArray(body?.hits) ? body.hits : []) {
      if (candidate?.name && jsDelivrCandidateMatches(font, query, candidate)) {
        candidates.set(candidate.name, candidate);
      }
    }
  }
  return [...candidates.values()].slice(0, MAX_JSDELIVR_CANDIDATES);
}

async function resolveFont(font, documentCount, catalog, npmSearchEnabled) {
  const direct = knownCdn(font);
  if (direct) {
    const url = direct.packageType === 'npm'
      ? `${CDN_ROOT}/npm/${direct.packageName}@${direct.version}/${direct.file}`
      : `${CDN_ROOT}/gh/${direct.packageName}@${direct.version}/${direct.file}`;
    try {
      if (await confirmsDownload(url)) {
        return {
          font,
          documentCount,
          status: 'available',
          delivery: direct.delivery ?? 'jsDelivr GitHub',
          packageName: direct.packageName,
          version: direct.version,
          license: direct.license,
          url,
          note: direct.note ?? 'rhwp-studio font-loader.ts에 이미 등록된 배포본',
        };
      }
      return { font, documentCount, status: 'not-found', delivery: '', packageName: direct.packageName, version: direct.version, license: direct.license, url, note: '등록된 jsDelivr URL이 현재 응답하지 않음' };
    } catch (error) {
      return { font, documentCount, status: 'lookup-error', delivery: '', packageName: direct.packageName, version: direct.version, license: direct.license, url, note: `jsDelivr CDN 확인 실패: ${error.message}` };
    }
  }

  const attempts = [];
  const candidatePackages = [];
  const sourceFont = fontsourceCandidate(font, catalog);
  if (sourceFont) {
    candidatePackages.push({
      packageName: `@fontsource/${sourceFont.id}`,
      source: 'Fontsource npm',
      license: sourceFont.license,
    });
  }
  const explicit = PACKAGE_ALIASES.get(familyKey(font));
  if (explicit && !candidatePackages.some(candidate => candidate.packageName === explicit)) {
    candidatePackages.push({ packageName: explicit, source: '명시 npm 별칭', license: '패키지 메타데이터 미확인' });
  }

  for (const candidate of candidatePackages) {
    attempts.push(candidate.packageName);
    try {
      const result = await packageFontFile(candidate.packageName, candidate.source, candidate.license);
      if (result) return { font, documentCount, status: 'available', ...result, note: '패키지 메타데이터와 실제 CDN 글꼴 파일 응답 확인' };
    } catch (error) {
      attempts.push(`${candidate.packageName} (${error.message})`);
    }
  }

  if (!npmSearchEnabled) {
    return finishUnmatchedFont(font, documentCount, attempts, 'not-found', 'jsDelivr 웹 검색을 생략함');
  }

  try {
    const candidates = await jsDelivrCandidates(font);
    for (const candidate of candidates) {
      if (candidatePackages.some(item => item.packageName === candidate.name)) continue;
      attempts.push(candidate.name);
      const result = await packageFontFile(
        candidate.name,
        'jsDelivr 웹 검색',
        candidate.license ?? '패키지 메타데이터 미확인',
        font,
        true,
      );
      if (result) return { font, documentCount, status: 'available', ...result, note: 'jsDelivr 웹 검색 후 실제 CDN 글꼴 파일 응답 확인' };
    }
    return finishUnmatchedFont(font, documentCount, attempts, 'not-found', 'Fontsource·명시 별칭·jsDelivr 웹 검색에서 검증 가능한 패키지를 찾지 못함');
  } catch (error) {
    return finishUnmatchedFont(font, documentCount, attempts, 'lookup-error', `jsDelivr 웹 검색 실패: ${error.message}`);
  }
}

function gitHead() {
  try {
    return execFileSync('git', ['rev-parse', '--short=12', 'HEAD'], {
      cwd: REPOSITORY_ROOT,
      encoding: 'utf8',
    }).trim();
  } catch {
    return '확인 불가';
  }
}

function writeDetails(rows) {
  const header = ['font', 'search_name', 'document_count', 'status', 'download_available', 'webfont_usable', 'webfont_usable_reason', 'delivery', 'package', 'version', 'license', 'download_url', 'note'];
  return [
    header.join('\t'),
    ...rows.map(row => {
      const availability = webfontAvailability(row);
      return [
        row.font,
        onlineWebFontsFamily(row.font),
        row.documentCount,
        row.status,
        availability.download,
        availability.usable,
        availability.reason,
        row.delivery,
        row.packageName,
        row.version,
        row.license,
        row.url,
        row.note,
      ].map(tsv).join('\t');
    }),
    '',
  ].join('\n');
}

function writeReport(options, scan, rows, catalogCount) {
  const statusCounts = new Map();
  for (const row of rows) statusCounts.set(row.status, (statusCounts.get(row.status) ?? 0) + 1);
  const downloadAvailable = rows.filter(row => webfontAvailability(row).download === '가능');
  const webfontUsable = rows.filter(row => webfontAvailability(row).usable === '가능');
  const webfontLicenseReview = rows.filter(row => webfontAvailability(row).usable === '라이선스 검토 필요');
  const available = rows.filter(row => row.status === 'available');
  const licenseReview = rows.filter(row => row.status === 'license-review');
  const topFonts = [...rows]
    .sort((left, right) => right.documentCount - left.documentCount || left.font.localeCompare(right.font, 'ko'))
    .slice(0, 30);
  const failureRows = [...scan.failures.entries()]
    .sort((left, right) => right[1] - left[1])
    .map(([kind, count]) => '| ' + markdownCell(kind) + ' | ' + count + ' |')
    .join('\n') || '| 없음 | 0 |';
  const availableRows = available
    .sort((left, right) => right.documentCount - left.documentCount || left.font.localeCompare(right.font, 'ko'))
    .map(row => {
      const availability = webfontAvailability(row);
      return '| ' + markdownCell(row.font) + ' | ' + row.documentCount + ' | ' + availability.download + ' | ' + availability.usable + ' | ' + markdownCell(row.delivery) + ' | `' + markdownCell(row.packageName) + '` | [파일](' + row.url + ') |';
    })
    .join('\n') || '| 없음 | 0 | - | - | - | - | - |';
  const licenseReviewRows = licenseReview
    .sort((left, right) => right.documentCount - left.documentCount || left.font.localeCompare(right.font, 'ko'))
    .map(row => {
      const availability = webfontAvailability(row);
      return '| ' + markdownCell(row.font) + ' | ' + row.documentCount + ' | ' + availability.download + ' | ' + availability.usable + ' | ' + markdownCell(row.delivery) + ' | [파일](' + row.url + ') | ' + markdownCell(row.note) + ' |';
    })
    .join('\n') || '| 없음 | 0 | - | - | - | - | - |';
  const topRows = topFonts
    .map(row => {
      const availability = webfontAvailability(row);
      return '| ' + markdownCell(row.font) + ' | ' + row.documentCount + ' | ' + availability.download + ' | ' + availability.usable + ' | ' + row.status + ' |';
    })
    .join('\n');
  const detailsPath = relative(dirname(options.report), options.details).replaceAll('\\', '/');
  const webfontScope = options.npmSearch
    ? 'jsDelivr 웹 검색 후보를 조사하고, 후보는 jsDelivr Data API의 파일 목록과 실제 CDN 글꼴 파일 응답까지 확인했다. Google Fonts는 공식 CSS API의 family 일치와 fonts.gstatic.com 응답을 확인했다. 동일 이름 Noonnu 후보는 상세 페이지의 웹사이트 사용 허가 요약과 CDN 응답을 함께 확인했다. OnlineWebFonts 후보는 CDN 응답만 확인하고 라이선스 검토 상태로 분리했다.'
    : 'jsDelivr 웹 검색은 생략했고, Fontsource 카탈로그와 기존 등록 GitHub 배포본, Google Fonts 공식 CSS API, 동일 이름 Noonnu 후보의 웹사이트 사용 허가 요약·CDN 응답, 동일 이름 OnlineWebFonts 후보의 CDN 응답을 확인했다.';
  const searchScope = options.npmSearch
    ? '공개 Fontsource 카탈로그와 jsDelivr 웹 검색, 기존 등록 GitHub 배포본을 이 스크립트의 동일 알고리즘으로 확인했을 때'
    : '공개 Fontsource 카탈로그와 기존 등록 GitHub 배포본을 이 스크립트의 동일 알고리즘으로 확인했을 때';

  return [
    '# korea_downloads HWP/HWPX 글꼴과 웹폰트 전수 조사',
    '',
    '- **생성 시각**: ' + new Date().toISOString(),
    '- **기준 커밋**: `' + gitHead() + '` (local `devel`)',
    '- **입력**: `' + options.inputRoot + '`의 HWP/HWPX ' + scan.records.toLocaleString('ko-KR') + '건',
    '- **파서**: `' + options.rhwp + '`의 `batch info --json --threads ' + options.threads + '`',
    '- **글꼴 범위**: HWP/HWPX DOCINFO의 한글·영어·한자·일어·기타·기호·사용자 7개 글꼴군 전체. 문서 내부 중복은 문서별 1회만 센다.',
    '- **웹폰트 판정**: Fontsource 카탈로그 ' + catalogCount.toLocaleString('ko-KR') + '건, `font-loader.ts`에 등록된 jsDelivr GitHub 글꼴, ' + webfontScope,
    '',
    '## 결과',
    '',
    '| 지표 | 건수 |',
    '| --- | ---: |',
    '| 입력 문서 | ' + scan.records.toLocaleString('ko-KR') + ' |',
    '| 파싱 성공 | ' + scan.parsedDocuments.toLocaleString('ko-KR') + ' |',
    '| 파싱 실패 | ' + (scan.records - scan.parsedDocuments).toLocaleString('ko-KR') + ' |',
    '| 고유 선언 글꼴 | ' + rows.length.toLocaleString('ko-KR') + ' |',
    '| 글꼴 파일 다운로드 가능 | ' + downloadAvailable.length.toLocaleString('ko-KR') + ' |',
    '| 웹폰트 사용 가능 | ' + webfontUsable.length.toLocaleString('ko-KR') + ' |',
    '| 웹폰트 사용 라이선스 검토 필요 | ' + webfontLicenseReview.length.toLocaleString('ko-KR') + ' |',
    '| 웹폰트 공급 경로·CDN 응답 확인 | ' + (statusCounts.get('available') ?? 0).toLocaleString('ko-KR') + ' |',
    '| CDN 응답 확인·원 권리자 라이선스 검토 필요 | ' + (statusCounts.get('license-review') ?? 0).toLocaleString('ko-KR') + ' |',
    '| 검증 가능한 배포본 미발견 | ' + (statusCounts.get('not-found') ?? 0).toLocaleString('ko-KR') + ' |',
    '| 조회 오류 | ' + (statusCounts.get('lookup-error') ?? 0).toLocaleString('ko-KR') + ' |',
    '',
    '`미발견`은 인터넷의 임의 GitHub 저장소까지 부정하는 판정이 아니다. ' + searchScope + ', **글꼴 바이트 파일을 실제로 내려받을 수 있는 웹폰트 URL을 검증하지 못했다**는 뜻이다. Noonnu의 `웹사이트 사용 가능` 표기는 Noonnu가 제공하는 요약 정보이므로 실제 배포 전 해당 글꼴의 최신 원 라이선스를 확인한다. OnlineWebFonts 응답 확인은 원 권리자의 웹 사용 허가를 뜻하지 않으며, `원 권리자 라이선스 검토 필요` 행은 서비스 배포에 사용하면 안 된다.',
    '',
    '## 파싱 실패',
    '',
    '| 분류 | 문서 수 |',
    '| --- | ---: |',
    failureRows,
    '',
    '## 웹폰트 공급 경로·CDN 응답 확인 글꼴',
    '',
    '| 글꼴 | 사용 문서 | 다운로드 | 웹폰트 사용 | 배포 경로 | 패키지 | 파일 |',
    '| --- | ---: | --- | --- | --- | --- | --- |',
    availableRows,
    '',
    '## CDN 응답 확인·원 권리자 라이선스 검토 필요',
    '',
    '| 글꼴 | 사용 문서 | 다운로드 | 웹폰트 사용 | 공급 경로 | 파일 | 비고 |',
    '| --- | ---: | --- | --- | --- | --- | --- |',
    licenseReviewRows,
    '',
    '## 사용 빈도 상위 30개',
    '',
    '| 글꼴 | 사용 문서 | 다운로드 | 웹폰트 사용 | 웹폰트 판정 |',
    '| --- | ---: | --- | --- | --- |',
    topRows,
    '',
    '## 전수 목록과 재현',
    '',
    '전체 ' + rows.length.toLocaleString('ko-KR') + '개 글꼴의 사용 문서 수, 다운로드 가능 여부, 웹폰트 사용 가능 여부와 근거, 패키지·버전·라이선스 표기, 검증 URL, 판정 사유는 [TSV 상세 목록](' + detailsPath + ')에 기록했다.',
    '',
    '`node scripts/survey_korea_downloads_font_jsdelivr.mjs --input <HWP|HWPX|디렉터리>`를 `devel`에서 실행하면 원시 임시 파일 없이 위 Markdown·TSV를 직접 다시 만든다. 실행 전에는 최신 바이너리를 만들기 위해 `cargo build --release`가 필요하다.',
    '',
  ].join('\n');
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  if (!options.inputRoot) throw new Error('--input <HWP|HWPX|디렉터리>가 필요합니다.');
  await fs.access(options.inputRoot);
  await fs.access(options.rhwp);
  setProgressPhase('입력 파일 목록화');
  const documents = await inputDocuments(options.inputRoot);
  if (documents.length === 0) throw new Error(`HWP/HWPX 파일이 없습니다: ${options.inputRoot}`);

  setProgressPhase('문서 선언 글꼴 집계');
  console.log(`1/3 문서 ${documents.length.toLocaleString('ko-KR')}건의 선언 글꼴을 집계합니다.`);
  const scan = await collectDeclaredFonts({ ...options, documents });
  setProgressPhase('Fontsource 카탈로그 및 jsDelivr 글꼴 조사 준비');
  console.log(`2/3 고유 글꼴 ${scan.fontDocumentCounts.size.toLocaleString('ko-KR')}개를 jsDelivr에서 조사합니다.`);
  let catalog;
  try {
    catalog = await getFontsourceCatalog();
  } catch (error) {
    console.error(`Fontsource 카탈로그 조회 실패: ${error.message}`);
    catalog = { byFamily: new Map(), byId: new Map(), count: 0 };
  }
  const fonts = [...scan.fontDocumentCounts.entries()]
    .map(([font, documentCount]) => ({ font, documentCount }))
    .sort((left, right) => left.font.localeCompare(right.font, 'ko'));
  setProgressPhase('jsDelivr·Google Fonts·Noonnu·OnlineWebFonts 웹폰트 사용 가능성 검증');
  const rows = await pool(fonts, options.concurrency, ({ font, documentCount }, index) => {
    if ((index + 1) % 50 === 0 || index + 1 === fonts.length) {
      console.log(`  웹폰트 조사 진행: ${index + 1}/${fonts.length}`);
    }
    return resolveFont(font, documentCount, catalog, options.npmSearch);
  });

  setProgressPhase('Markdown·TSV 증적 기록');
  console.log('3/3 Markdown·TSV 보고서를 devel 작업 트리에 기록합니다.');
  await fs.mkdir(dirname(options.report), { recursive: true });
  await fs.mkdir(dirname(options.details), { recursive: true });
  await fs.writeFile(options.details, writeDetails(rows), 'utf8');
  await fs.writeFile(options.report, writeReport(options, scan, rows, catalog.count), 'utf8');
  const downloadAvailable = rows.filter(row => webfontAvailability(row).download === '가능').length;
  const webfontUsable = rows.filter(row => webfontAvailability(row).usable === '가능').length;
  const webfontLicenseReview = rows.filter(row => webfontAvailability(row).usable === '라이선스 검토 필요').length;
  const errors = rows.filter(row => row.status === 'lookup-error').length;
  console.log(`완료: ${rows.length}개 중 다운로드 ${downloadAvailable}개 가능, 웹폰트 사용 ${webfontUsable}개 가능, ${webfontLicenseReview}개 라이선스 검토 필요, ${errors}개 조회 오류`);
  console.log(`보고서: ${options.report}`);
  console.log(`상세 TSV: ${options.details}`);
  if (scan.exitCode !== 0) {
    console.log(`참고: batch info 종료 코드 ${scan.exitCode} (개별 파싱 실패 ${scan.records - scan.parsedDocuments}건을 NDJSON으로 보존하고 계속 진행함)`);
  }
}

main().catch(error => {
  console.error(`조사 실패: ${error.stack ?? error.message}`);
  process.exitCode = 1;
});
