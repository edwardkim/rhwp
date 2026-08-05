#!/usr/bin/env node
/**
 * svg_native_wasm_diff.mjs — native CLI SVG ↔ WASM SVG 문자열 패리티 하네스
 *
 * 같은 문서·같은 페이지를 두 경로로 렌더해 byte 단위로 비교한다:
 *   - native 축: `rhwp export-svg --json` (render_page_svg_native, legacy 경로)
 *   - wasm 축:   pkg/rhwp.js 를 Node 에서 직접 로드해 HwpDocument.renderPageSvg()
 * 두 축 모두 동일한 Rust 함수(rendering.rs render_page_svg_native)를 타므로,
 * 차이가 나면 원인은 측정기 분기(EmbeddedTextMeasurer vs WasmTextMeasurer),
 * 환경변수 분기(WASM 은 항상 미설정), cfg(target_arch) 분기 중 하나다.
 *
 * 사용:
 *   node scripts/svg_native_wasm_diff.mjs <문서|디렉터리>... [옵션]
 *
 * 옵션:
 *   --out <dir>        산출물 디렉터리 (기본 output/svg-native-wasm-diff)
 *   --profile <p>      layer 경로 비교 (renderPageSvgWithProfile ↔ export-svg --profile)
 *   --pages <n,n,...>  0-based 페이지 서브셋 (기본 전체)
 *   --rhwp <path>      native 바이너리 (기본 target/release/rhwp)
 *   --pkg <dir>        wasm pkg 디렉터리 (기본 pkg)
 *   --limit <n>        디렉터리 입력 시 최대 문서 수
 *   --max-hunk <n>     보고서에 남길 diff 줄 수 (기본 40)
 *   --keep-match       일치한 SVG 파일도 삭제하지 않고 보존
 *
 * 종료 코드: 0 = 전 문서 일치, 1 = 불일치 존재, 2 = 사용법/환경 오류
 *
 * 주의: native 축은 RHWP_* 환경변수를 제거한 채 실행한다. WASM 에서는
 * std::env::var 가 항상 Err 이므로, env 기본값끼리 비교해야 공정하다.
 */
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync, spawnSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..');

// ---------- 인자 파싱 ----------
const args = process.argv.slice(2);
const opts = {
  out: path.join(repoRoot, 'output', 'svg-native-wasm-diff'),
  profile: null,
  pages: null,
  rhwp: path.join(repoRoot, 'target', 'release', 'rhwp'),
  pkg: path.join(repoRoot, 'pkg'),
  limit: Infinity,
  maxHunk: 40,
  keepMatch: false,
};
const inputs = [];
for (let i = 0; i < args.length; i++) {
  const a = args[i];
  if (a === '--out') opts.out = path.resolve(args[++i]);
  else if (a === '--profile') opts.profile = args[++i];
  else if (a === '--pages') opts.pages = args[++i].split(',').map((s) => parseInt(s, 10));
  else if (a === '--rhwp') opts.rhwp = path.resolve(args[++i]);
  else if (a === '--pkg') opts.pkg = path.resolve(args[++i]);
  else if (a === '--limit') opts.limit = parseInt(args[++i], 10);
  else if (a === '--max-hunk') opts.maxHunk = parseInt(args[++i], 10);
  else if (a === '--keep-match') opts.keepMatch = true;
  else if (a.startsWith('--')) {
    console.error(`알 수 없는 옵션: ${a}`);
    process.exit(2);
  } else inputs.push(path.resolve(a));
}
if (inputs.length === 0) {
  console.error('사용법: node scripts/svg_native_wasm_diff.mjs <문서|디렉터리>... [--profile print] [--pages 0,1]');
  process.exit(2);
}

// ---------- 문서 목록 수집 ----------
const DOC_EXT = new Set(['.hwp', '.hwpx']);
const docs = [];
for (const input of inputs) {
  const st = fs.statSync(input);
  if (st.isDirectory()) {
    const walk = (dir) => {
      for (const e of fs.readdirSync(dir, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
        if (docs.length >= opts.limit) return;
        const p = path.join(dir, e.name);
        if (e.isDirectory()) walk(p);
        else if (DOC_EXT.has(path.extname(e.name).toLowerCase())) docs.push(p);
      }
    };
    walk(input);
  } else {
    docs.push(input);
  }
  if (docs.length >= opts.limit) break;
}
if (docs.length === 0) {
  console.error('입력에서 .hwp/.hwpx 문서를 찾지 못했습니다.');
  process.exit(2);
}

// ---------- 환경 준비 ----------
if (!fs.existsSync(opts.rhwp)) {
  console.error(`native 바이너리 없음: ${opts.rhwp} — cargo build --release 먼저 실행`);
  process.exit(2);
}
const wasmJs = path.join(opts.pkg, 'rhwp.js');
if (!fs.existsSync(wasmJs)) {
  console.error(`wasm pkg 없음: ${wasmJs} — wasm-pack build --target web --out-dir pkg 먼저 실행`);
  process.exit(2);
}

// WASM 은 env 를 못 보므로 native 도 RHWP_* 제거한 env 로 실행한다.
const scrubbedEnv = { ...process.env };
const strippedVars = Object.keys(scrubbedEnv).filter((k) => k.startsWith('RHWP_'));
for (const k of strippedVars) delete scrubbedEnv[k];

const wasmMod = await import(pathToFileURL(wasmJs).href);
await wasmMod.default({ module_or_path: fs.readFileSync(path.join(opts.pkg, 'rhwp_bg.wasm')) });

const nativeVersion = execFileSync(opts.rhwp, ['--version'], { env: scrubbedEnv }).toString().trim();
const wasmVersion = wasmMod.version();
const gitHead = spawnSync('git', ['-C', repoRoot, 'rev-parse', '--short', 'HEAD']).stdout?.toString().trim() ?? 'unknown';
const versionMatch = nativeVersion.includes(wasmVersion);
if (!versionMatch) {
  console.error(`경고: 버전 불일치 — native "${nativeVersion}" vs wasm "${wasmVersion}". 같은 커밋에서 재빌드 권장.`);
}

fs.mkdirSync(opts.out, { recursive: true });

// ---------- 유틸 ----------
const stemOf = (p) => path.basename(p).replace(/\.[^.]+$/, '');
// 동명 문서 충돌 방지용 짧은 해시
const shortHash = (s) => {
  let h = 0x811c9dc5;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return h.toString(16).padStart(8, '0');
};

function diffHead(fileA, fileB, maxLines) {
  const r = spawnSync('diff', ['--speed-large-files', '-u', fileA, fileB], {
    maxBuffer: 64 * 1024 * 1024,
  });
  const out = r.stdout?.toString() ?? '';
  const lines = out.split('\n');
  const changed = lines.filter((l) => /^[+-][^+-]/.test(l)).length;
  return { changedLines: changed, hunkHead: lines.slice(0, maxLines).join('\n') };
}

// ---------- 문서별 비교 ----------
const report = {
  schemaVersion: '1.0',
  gitHead,
  nativeVersion,
  wasmVersion,
  profile: opts.profile,
  strippedEnvVars: strippedVars,
  docs: [],
};
let anyMismatch = false;

for (const [idx, docPath] of docs.entries()) {
  const label = `${stemOf(docPath)}-${shortHash(docPath)}`;
  const docOut = path.join(opts.out, label);
  const nativeDir = path.join(docOut, 'native');
  const wasmDir = path.join(docOut, 'wasm');
  fs.mkdirSync(nativeDir, { recursive: true });
  fs.mkdirSync(wasmDir, { recursive: true });
  const entry = { source: docPath, status: 'match', pages: [] };
  report.docs.push(entry);
  console.log(`[${idx + 1}/${docs.length}] ${docPath}`);

  // native 축
  let manifest;
  {
    const cliArgs = ['export-svg', docPath, '--json', '-o', nativeDir];
    if (opts.profile) cliArgs.push('--profile', opts.profile);
    const r = spawnSync(opts.rhwp, cliArgs, { env: scrubbedEnv, maxBuffer: 256 * 1024 * 1024 });
    if (r.status !== 0) {
      entry.status = 'native-error';
      entry.error = (r.stderr?.toString() ?? '').slice(0, 2000);
      anyMismatch = true;
      continue;
    }
    try {
      manifest = JSON.parse(r.stdout.toString());
    } catch {
      entry.status = 'native-error';
      entry.error = 'export-svg --json 매니페스트 파싱 실패';
      anyMismatch = true;
      continue;
    }
  }

  // wasm 축
  let doc;
  try {
    doc = new wasmMod.HwpDocument(new Uint8Array(fs.readFileSync(docPath)));
  } catch (e) {
    entry.status = 'wasm-error';
    entry.error = String(e).slice(0, 2000);
    anyMismatch = true;
    continue;
  }

  try {
    const wasmPageCount = doc.pageCount();
    entry.nativePageCount = manifest.pageCount;
    entry.wasmPageCount = wasmPageCount;
    if (manifest.pageCount !== wasmPageCount) {
      entry.status = 'page-count-mismatch';
      anyMismatch = true;
      // 페이지 수가 달라도 겹치는 구간은 계속 비교한다.
    }

    const available = Math.min(manifest.pageCount, wasmPageCount);
    const targets = (opts.pages ?? [...Array(available).keys()]).filter((p) => p < available);
    const nativePathByPage = new Map(manifest.pages.map((p) => [p.page, p.path]));

    for (const pageNum of targets) {
      const nativeFile = nativePathByPage.get(pageNum);
      if (!nativeFile) continue;
      let svg;
      try {
        svg = opts.profile ? doc.renderPageSvgWithProfile(pageNum, opts.profile) : doc.renderPageSvg(pageNum);
      } catch (e) {
        entry.status = 'wasm-error';
        entry.pages.push({ page: pageNum, result: 'wasm-render-error', error: String(e).slice(0, 500) });
        anyMismatch = true;
        continue;
      }
      const wasmFile = path.join(wasmDir, path.basename(nativeFile));
      fs.writeFileSync(wasmFile, svg);

      const nativeBytes = fs.readFileSync(nativeFile);
      if (nativeBytes.equals(Buffer.from(svg))) {
        entry.pages.push({ page: pageNum, result: 'match', bytes: nativeBytes.length });
        if (!opts.keepMatch) {
          fs.rmSync(nativeFile);
          fs.rmSync(wasmFile);
        }
      } else {
        const d = diffHead(nativeFile, wasmFile, opts.maxHunk);
        entry.pages.push({
          page: pageNum,
          result: 'diff',
          nativeBytes: nativeBytes.length,
          wasmBytes: svg.length,
          changedLines: d.changedLines,
          native: nativeFile,
          wasm: wasmFile,
          diffHead: d.hunkHead,
        });
        if (entry.status === 'match') entry.status = 'diff';
        anyMismatch = true;
      }
    }
  } finally {
    doc.free();
  }

  // 일치 문서의 빈 디렉터리 정리
  if (entry.status === 'match' && !opts.keepMatch) {
    fs.rmSync(docOut, { recursive: true, force: true });
  }
  const diffPages = entry.pages.filter((p) => p.result !== 'match').length;
  console.log(`  → ${entry.status} (페이지 ${entry.pages.length}개 비교, 불일치 ${diffPages})`);
}

// ---------- 보고 ----------
const reportPath = path.join(opts.out, 'report.json');
fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));

const counts = {};
for (const d of report.docs) counts[d.status] = (counts[d.status] ?? 0) + 1;
console.log('\n=== 요약 ===');
console.log(`문서 ${report.docs.length}개:`, JSON.stringify(counts));
console.log(`보고서: ${reportPath}`);
if (!versionMatch) console.log(`경고: native/wasm 버전 불일치 상태로 실행됨 (${nativeVersion} vs ${wasmVersion})`);
process.exit(anyMismatch ? 1 : 0);
