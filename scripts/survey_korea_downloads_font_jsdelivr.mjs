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
const NPM_SEARCH_URL = 'https://registry.npmjs.org/-/v1/search';
const JSD_DELIVR_DATA_URL = 'https://data.jsdelivr.com/v1/package/npm';
const CDN_ROOT = 'https://cdn.jsdelivr.net';
const REQUEST_TIMEOUT_MS = 20_000;

function usage() {
  return `사용법: node scripts/survey_korea_downloads_font_jsdelivr.mjs [옵션]

옵션:
  --input <경로>        HWP/HWPX 파일 하나 또는 코퍼스 디렉터리 (필수)
  --rhwp <경로>         devel rhwp 실행 파일 (기본: ${DEFAULT_RHWP})
  --report <경로>       Markdown 보고서 (기본: ${DEFAULT_REPORT})
  --details <경로>      전수 TSV (기본: ${DEFAULT_DETAILS})
  --threads <수>        rhwp batch 파싱 병렬도 (기본: 8)
  --concurrency <수>    네트워크 조사 병렬도 (기본: 8)
  --no-npm-search       Fontsource/명시 매핑만 확인
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
    if (arg === '--no-npm-search') {
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

function compact(value) {
  return normalized(value).replace(/[\s._'"()\[\]{}\-]/g, '');
}

function familyKey(value) {
  let key = normalized(value).replace(/^[#\-]+/, '');
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
  return String(value ?? '').replace(/\|/g, '\\|').replace(/[\r\n]+/g, ' ');
}

async function walkDocuments(root) {
  const files = [];
  async function visit(directory) {
    const entries = await fs.readdir(directory, { withFileTypes: true });
    for (const entry of entries) {
      const path = resolve(directory, entry.name);
      if (entry.isDirectory()) {
        await visit(path);
      } else if (entry.isFile() && /\.hwp(x)?$/iu.test(entry.name)) {
        files.push(path);
      }
    }
  }
  await visit(root);
  return files.sort((left, right) => left.localeCompare(right, 'ko'));
}

async function inputDocuments(input) {
  const entry = await fs.stat(input);
  if (entry.isFile()) {
    if (!/\.hwp(x)?$/iu.test(input)) {
      throw new Error(`입력 파일은 .hwp 또는 .hwpx여야 합니다: ${input}`);
    }
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
          .map(value => String(value).replace(/\u0000/g, '').normalize('NFC').trim())
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

function knownCdn(font) {
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
  ['kopub돋움체', '@noonnu/kopubworlddotum'],
  ['kopub바탕체', '@noonnu/kopubworldbatang'],
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

function jsDelivrPackagePath(packageName) {
  return encodeURIComponent(packageName);
}

async function packageFontFile(packageName, source, licenseHint = '미확인') {
  const packagePath = jsDelivrPackagePath(packageName);
  const packageInfo = await fetchJson(`${JSD_DELIVR_DATA_URL}/${packagePath}`);
  const version = packageInfo?.tags?.latest;
  if (!version) return null;
  const flat = await fetchJson(
    `${JSD_DELIVR_DATA_URL}/${packagePath}@${encodeURIComponent(version)}/flat`,
  );
  const files = Array.isArray(flat?.files) ? flat.files : [];
  const fontFile = files
    .map(file => String(file.name ?? '').replace(/^\//, ''))
    .find(file => /\.(woff2?|ttf|otf)$/iu.test(file));
  if (!fontFile) return null;
  const url = `${CDN_ROOT}/npm/${packageName}@${version}/${fontFile}`;
  if (!(await confirmsDownload(url))) return null;
  return { packageName, version, file: fontFile, url, source, license: licenseHint };
}

function npmCandidateMatches(font, packageInfo) {
  const family = familyKey(font);
  if (family.length < 3) return false;
  const packageName = String(packageInfo?.name ?? '');
  const text = [
    packageName,
    packageInfo?.description ?? '',
    ...(Array.isArray(packageInfo?.keywords) ? packageInfo.keywords : []),
  ].join(' ');
  if (familyKey(text).includes(family)) return true;
  const asciiFamily = normalized(font).replace(/[^a-z0-9]+/g, '');
  const asciiPackage = normalized(packageName).replace(/[^a-z0-9]+/g, '');
  return asciiFamily.length >= 4 && asciiPackage.includes(asciiFamily);
}

async function npmCandidates(font) {
  const query = new URLSearchParams({ text: font, size: '20' });
  const body = await fetchJson(`${NPM_SEARCH_URL}?${query}`);
  return (Array.isArray(body?.objects) ? body.objects : [])
    .map(entry => entry.package)
    .filter(candidate => candidate?.name && npmCandidateMatches(font, candidate));
}

async function resolveFont(font, documentCount, catalog, npmSearchEnabled) {
  const direct = knownCdn(font);
  if (direct) {
    const url = `${CDN_ROOT}/gh/${direct.packageName}@${direct.version}/${direct.file}`;
    try {
      if (await confirmsDownload(url)) {
        return {
          font,
          documentCount,
          status: 'available',
          delivery: 'jsDelivr GitHub',
          packageName: direct.packageName,
          version: direct.version,
          license: direct.license,
          url,
          note: 'rhwp-studio font-loader.ts에 이미 등록된 배포본',
        };
      }
      return { font, documentCount, status: 'not-found', delivery: '', packageName: direct.packageName, version: direct.version, license: direct.license, url, note: '등록된 jsDelivr URL이 현재 응답하지 않음' };
    } catch (error) {
      return { font, documentCount, status: 'lookup-error', delivery: '', packageName: direct.packageName, version: direct.version, license: direct.license, url, note: `GitHub CDN 확인 실패: ${error.message}` };
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
    return { font, documentCount, status: 'not-found', delivery: '', packageName: attempts.join(', '), version: '', license: '', url: '', note: 'npm 전문 검색을 생략함' };
  }

  try {
    const candidates = await npmCandidates(font);
    for (const candidate of candidates) {
      if (candidatePackages.some(item => item.packageName === candidate.name)) continue;
      attempts.push(candidate.name);
      const result = await packageFontFile(
        candidate.name,
        'npm 전문 검색',
        candidate.license ?? '패키지 메타데이터 미확인',
      );
      if (result) return { font, documentCount, status: 'available', ...result, note: 'npm 전문 검색 후 실제 CDN 글꼴 파일 응답 확인' };
    }
    return { font, documentCount, status: 'not-found', delivery: '', packageName: attempts.join(', '), version: '', license: '', url: '', note: 'Fontsource·명시 별칭·npm 전문 검색에서 검증 가능한 패키지를 찾지 못함' };
  } catch (error) {
    return { font, documentCount, status: 'lookup-error', delivery: '', packageName: attempts.join(', '), version: '', license: '', url: '', note: `npm 전문 검색 실패: ${error.message}` };
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
  const header = ['font', 'document_count', 'status', 'delivery', 'package', 'version', 'license', 'download_url', 'note'];
  return [
    header.join('\t'),
    ...rows.map(row => [
      row.font,
      row.documentCount,
      row.status,
      row.delivery,
      row.packageName,
      row.version,
      row.license,
      row.url,
      row.note,
    ].map(tsv).join('\t')),
    '',
  ].join('\n');
}

function writeReport(options, scan, rows, catalogCount) {
  const statusCounts = new Map();
  for (const row of rows) statusCounts.set(row.status, (statusCounts.get(row.status) ?? 0) + 1);
  const available = rows.filter(row => row.status === 'available');
  const topFonts = [...rows]
    .sort((left, right) => right.documentCount - left.documentCount || left.font.localeCompare(right.font, 'ko'))
    .slice(0, 30);
  const failureRows = [...scan.failures.entries()]
    .sort((left, right) => right[1] - left[1])
    .map(([kind, count]) => `| ${markdownCell(kind)} | ${count} |`)
    .join('\n') || '| 없음 | 0 |';
  const availableRows = available
    .sort((left, right) => right.documentCount - left.documentCount || left.font.localeCompare(right.font, 'ko'))
    .map(row => `| ${markdownCell(row.font)} | ${row.documentCount} | ${markdownCell(row.delivery)} | \`${markdownCell(row.packageName)}\` | [파일](${row.url}) |`)
    .join('\n') || '| 없음 | 0 | - | - | - |';
  const topRows = topFonts
    .map(row => `| ${markdownCell(row.font)} | ${row.documentCount} | ${row.status} |`)
    .join('\n');
  const detailsPath = relative(dirname(options.report), options.details).replaceAll('\\', '/');
  const npmScope = options.npmSearch
    ? '${npmScope}'
    : 'npm 전문 검색은 레지스트리 요청 제한으로 생략했고, Fontsource 카탈로그와 기존 등록 GitHub 배포본만 실제 CDN 파일까지 확인했다.';
  const searchScope = options.npmSearch
    ? '${searchScope}'
    : '공개 Fontsource 카탈로그와 기존 등록 GitHub 배포본을 이 스크립트의 동일 알고리즘으로 확인했을 때';
  return `# korea_downloads HWP/HWPX 글꼴과 jsDelivr 전수 조사\n\n- **생성 시각**: ${new Date().toISOString()}\n- **기준 커밋**: \`${gitHead()}\` (local \`devel\`)\n- **입력**: \`${options.inputRoot}\`의 HWP/HWPX ${scan.records.toLocaleString('ko-KR')}건\n- **파서**: \`${options.rhwp}\`의 \`batch info --json --threads ${options.threads}\`\n- **글꼴 범위**: HWP/HWPX DOCINFO의 한글·영어·한자·일어·기타·기호·사용자 7개 글꼴군 전체. 문서 내부 중복은 문서별 1회만 센다.\n- **jsDelivr 판정**: Fontsource 카탈로그 ${catalogCount.toLocaleString('ko-KR')}건, \`font-loader.ts\`에 등록된 jsDelivr GitHub 글꼴, ${npmScope}\n\n## 결과\n\n| 지표 | 건수 |\n| --- | ---: |\n| 입력 문서 | ${scan.records.toLocaleString('ko-KR')} |\n| 파싱 성공 | ${scan.parsedDocuments.toLocaleString('ko-KR')} |\n| 파싱 실패 | ${(scan.records - scan.parsedDocuments).toLocaleString('ko-KR')} |\n| 고유 선언 글꼴 | ${rows.length.toLocaleString('ko-KR')} |\n| jsDelivr에서 다운로드 확인 | ${(statusCounts.get('available') ?? 0).toLocaleString('ko-KR')} |\n| 검증 가능한 배포본 미발견 | ${(statusCounts.get('not-found') ?? 0).toLocaleString('ko-KR')} |\n| 조회 오류 | ${(statusCounts.get('lookup-error') ?? 0).toLocaleString('ko-KR')} |\n\n\`미발견\`은 인터넷의 임의 GitHub 저장소까지 부정하는 판정이 아니다. ${searchScope}, **글꼴 바이트 파일을 실제로 내려받을 수 있는 jsDelivr URL을 검증하지 못했다**는 뜻이다. 패키지가 존재해도 원 글꼴과 동일한 서체인지, 라이선스가 해당 사용 목적을 허용하는지는 각 배포본의 라이선스를 별도로 확인해야 한다.\n\n## 파싱 실패\n\n| 분류 | 문서 수 |\n| --- | ---: |\n${failureRows}\n\n## jsDelivr 다운로드 확인 글꼴\n\n| 글꼴 | 사용 문서 | 배포 경로 | 패키지 | 파일 |\n| --- | ---: | --- | --- | --- |\n${availableRows}\n\n## 사용 빈도 상위 30개\n\n| 글꼴 | 사용 문서 | jsDelivr 판정 |\n| --- | ---: | --- |\n${topRows}\n\n## 전수 목록과 재현\n\n전체 ${rows.length.toLocaleString('ko-KR')}개 글꼴의 사용 문서 수, 패키지·버전·라이선스 표기, 검증 URL, 판정 사유는 [TSV 상세 목록](${detailsPath})에 기록했다.\n\n\`node scripts/survey_korea_downloads_font_jsdelivr.mjs --input <HWP|HWPX|디렉터리>\`를 \`devel\`에서 실행하면 원시 임시 파일 없이 위 Markdown·TSV를 직접 다시 만든다. 실행 전에는 최신 바이너리를 만들기 위해 \`cargo build --release\`가 필요하다.\n`;
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  if (!options.inputRoot) throw new Error('--input <HWP|HWPX|디렉터리>가 필요합니다.');
  await fs.access(options.inputRoot);
  await fs.access(options.rhwp);
  const documents = await inputDocuments(options.inputRoot);
  if (documents.length === 0) throw new Error(`HWP/HWPX 파일이 없습니다: ${options.inputRoot}`);

  console.log(`1/3 문서 ${documents.length.toLocaleString('ko-KR')}건의 선언 글꼴을 집계합니다.`);
  const scan = await collectDeclaredFonts({ ...options, documents });
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
  const rows = await pool(fonts, options.concurrency, ({ font, documentCount }, index) => {
    if ((index + 1) % 50 === 0 || index + 1 === fonts.length) {
      console.log(`  jsDelivr 조사 진행: ${index + 1}/${fonts.length}`);
    }
    return resolveFont(font, documentCount, catalog, options.npmSearch);
  });

  console.log('3/3 Markdown·TSV 보고서를 devel 작업 트리에 기록합니다.');
  await fs.mkdir(dirname(options.report), { recursive: true });
  await fs.mkdir(dirname(options.details), { recursive: true });
  await fs.writeFile(options.details, writeDetails(rows), 'utf8');
  await fs.writeFile(options.report, writeReport(options, scan, rows, catalog.count), 'utf8');
  const available = rows.filter(row => row.status === 'available').length;
  const errors = rows.filter(row => row.status === 'lookup-error').length;
  console.log(`완료: ${rows.length}개 중 ${available}개 다운로드 확인, ${errors}개 조회 오류`);
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
