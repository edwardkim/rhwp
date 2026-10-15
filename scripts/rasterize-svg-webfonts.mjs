#!/usr/bin/env node

import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, renameSync, rmSync, writeFileSync } from 'node:fs';
import { basename, delimiter, dirname, resolve } from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(SCRIPT_DIR, '..');
const PROJECTION_PATH = 'rhwp-studio/src/core/generated/font-rule-projections/webfont-supply.ts';
const TERMINAL_FALLBACK_FAMILY = '__rhwp_visual_sweep_noto_sans_kr__';

function cssString(value) {
  return `"${value.replaceAll('\\', '\\\\').replaceAll('"', '\\"')}"`;
}

function fontFaceFamily(rule) {
  const value = rule.match(/(?:^|[;{])\s*font-family\s*:\s*([^;}]+)/iu)?.[1]?.trim();
  if (!value) return null;
  return value.replace(/^(['"])(.*)\1$/u, '$2').toLocaleLowerCase('en-US');
}

// Embedded full fonts can make a single @font-face rule tens of megabytes long.
// V8's regexp iterator over that rule can exhaust its stack before Chrome runs.
function fontFaceRanges(source) {
  const ranges = [];
  let offset = 0;
  while (offset < source.length) {
    const start = source.indexOf('@', offset);
    if (start < 0) break;
    const opening = source.slice(start, start + 64).match(/^@font-face\s*\{/iu);
    if (!opening) {
      offset = start + 1;
      continue;
    }
    const close = source.indexOf('}', start + opening[0].length);
    if (close < 0) break;
    ranges.push([start, close + 1]);
    offset = close + 1;
  }
  return ranges;
}

// 반복된 전체 글꼴을 HTML에서 다시 해석하지 않고 같은 바이트의 파일을 참조한다.
// 원본 SVG와 글꼴 선택·별칭·형상은 보존하며 공용 검토 cache를 재사용한다.
function externalizeEmbeddedFontSources(source) {
  const cacheDir = resolve(ROOT, 'output/pr-review/font-raster-cache');
  const payloads = new Map();
  const chunks = [];
  let cursor = 0;
  for (const [start, end] of fontFaceRanges(source)) {
    chunks.push(source.slice(cursor, start));
    let face = source.slice(start, end);
    const dataStart = face.indexOf('data:');
    const header = dataStart < 0 ? null : face.slice(dataStart, dataStart + 256)
      .match(/^data:(?:font\/[^;,\s]+|application\/[^;,\s]+);(?:charset=[^;,]+;)?base64,/u);
    if (header) {
      const payloadStart = dataStart + header[0].length;
      const closing = face.indexOf(')', payloadStart);
      if (closing < 0) throw new Error('내장 글꼴 URL의 끝을 찾지 못했습니다.');
      const payloadEnd = ['"', "'"].includes(face[closing - 1]) ? closing - 1 : closing;
      const encoded = face.slice(payloadStart, payloadEnd);
      const bytes = Buffer.from(encoded, 'base64');
      if (bytes.toString('base64') !== encoded) {
        throw new Error('내장 글꼴의 base64 바이트가 올바르지 않습니다.');
      }
      const sha256 = createHash('sha256').update(bytes).digest('hex');
      const path = resolve(cacheDir, `${sha256}.font`);
      if (!payloads.has(sha256)) {
        mkdirSync(cacheDir, { recursive: true });
        if (existsSync(path)) {
          if (createHash('sha256').update(readFileSync(path)).digest('hex') !== sha256) {
            throw new Error(`공용 글꼴 cache hash 불일치: ${path}`);
          }
        } else {
          // 병렬 캡처도 완성된 동일 payload만 읽도록 원자적으로 게시한다.
          const temporary = `${path}.${process.pid}.tmp`;
          try {
            writeFileSync(temporary, bytes, { flag: 'wx' });
            renameSync(temporary, path);
          } finally {
            rmSync(temporary, { force: true });
          }
        }
        payloads.set(sha256, { sha256, bytes: bytes.length, path });
      }
      face = face.slice(0, dataStart) + pathToFileURL(path).href + face.slice(payloadEnd);
    }
    chunks.push(face);
    cursor = end;
  }
  chunks.push(source.slice(cursor));
  return { svg: chunks.join(''), payloads: [...payloads.values()] };
}

function declaredFontFaceFamilies(source) {
  return new Set(fontFaceRanges(source)
    .map(([start, end]) => (
      fontFaceFamily(source.slice(start, Math.min(end, start + 4096)))
      ?? fontFaceFamily(source.slice(Math.max(start, end - 4096), end))
    ))
    .filter(family => family !== null));
}

export function parseWebfontRules(source) {
  const marker = 'export const FONT_RULE_CANVAS2D_WEBFONT_RULES';
  const markerOffset = source.indexOf(marker);
  if (markerOffset < 0) throw new Error('webfont projection array를 찾지 못했습니다.');
  const freezeOffset = source.indexOf('Object.freeze(', markerOffset);
  const arrayStart = source.indexOf('[', freezeOffset);
  const arrayEnd = source.indexOf('\n]);', arrayStart);
  if (arrayStart < 0 || arrayEnd < 0) throw new Error('webfont projection array 경계가 올바르지 않습니다.');
  const rules = JSON.parse(source.slice(arrayStart, arrayEnd + 2));
  if (!Array.isArray(rules)) throw new Error('webfont projection은 배열이어야 합니다.');
  return rules.filter(rule => (
    rule
    && typeof rule.sourceFace === 'string'
    && rule.supply
    && typeof rule.supply.fontFamily === 'string'
    && typeof rule.supply.sourceUrl === 'string'
    && typeof rule.supply.format === 'string'
  ));
}

export function selectWebfontRules(svgSource, rules) {
  const lowerSource = svgSource.toLocaleLowerCase('en-US');
  const declaredFamilies = declaredFontFaceFamilies(svgSource);
  const selected = new Map();
  for (const rule of rules) {
    const sourceFace = rule.sourceFace.toLocaleLowerCase('en-US');
    if (lowerSource.includes(sourceFace)
      && !declaredFamilies.has(rule.supply.fontFamily.toLocaleLowerCase('en-US'))) {
      selected.set(`${rule.supply.fontFamily}\u0000${rule.supply.sourceUrl}`, rule);
    }
  }
  return [...selected.values()];
}

function webfontUrl(root, sourceUrl) {
  if (/^https?:\/\//iu.test(sourceUrl)) return sourceUrl;
  if (!sourceUrl.startsWith('fonts/')) {
    throw new Error(`지원하지 않는 로컬 webfont 경로: ${sourceUrl}`);
  }
  return pathToFileURL(resolve(root, 'assets', 'fonts', sourceUrl.slice('fonts/'.length))).href;
}

export function buildWebfontCss(root, selectedRules) {
  const faces = selectedRules.map(rule => {
    const supply = rule.supply;
    const unicodeRange = supply.unicodeRange ? ` unicode-range: ${supply.unicodeRange};` : '';
    return `@font-face { font-family: ${cssString(supply.fontFamily)}; src: url(${cssString(webfontUrl(root, supply.sourceUrl))}) format(${cssString(supply.format)}); font-display: block;${unicodeRange} }`;
  });
  const terminalUrl = pathToFileURL(resolve(root, 'assets', 'fonts', 'NotoSansKR-Regular.woff2')).href;
  faces.push(`@font-face { font-family: ${cssString(TERMINAL_FALLBACK_FAMILY)}; src: url(${cssString(terminalUrl)}) format("woff2"); font-display: block; }`);
  return faces.join('\n');
}

function appendTerminalFallback(fontList) {
  if (fontList.includes(TERMINAL_FALLBACK_FAMILY)) return fontList;
  // This identifier needs no CSS quotes, which would break quoted SVG attributes.
  return `${fontList.trim()}, ${TERMINAL_FALLBACK_FAMILY}`;
}

export function prepareSvgForWebfontRaster(svgSource, webfontCss) {
  // [#6891] export-svg --font-style owns local aliases and legacy-face safety
  // ordering. Webfont supply must supplement, not discard or shadow, that policy.
  const declaredFamilies = declaredFontFaceFamilies(svgSource);
  const supplementalCss = webfontCss.replace(
    /@font-face\s*\{[^{}]*\}/giu,
    rule => declaredFamilies.has(fontFaceFamily(rule)) ? '' : rule,
  );
  const withAttributeFallback = svgSource.replace(
    /font-family=(['"])(.*?)\1/giu,
    (_match, quote, fontList) => `font-family=${quote}${appendTerminalFallback(fontList)}${quote}`,
  );
  // Leave embedded @font-face blocks byte-for-byte intact. A regexp that spans
  // their data URLs overflows the JS stack on full HCR/Haansoft font exports.
  const cssFallback = part => part.replace(
    /(font-family\s*:\s*)([^;}]+)/giu,
    (_match, prefix, fontList) => `${prefix}${appendTerminalFallback(fontList)}`,
  );
  const chunks = [];
  let cursor = 0;
  for (const [start, end] of fontFaceRanges(withAttributeFallback)) {
    chunks.push(cssFallback(withAttributeFallback.slice(cursor, start)));
    chunks.push(withAttributeFallback.slice(start, end));
    cursor = end;
  }
  chunks.push(cssFallback(withAttributeFallback.slice(cursor)));
  const withCssFallback = chunks.join('');
  return withCssFallback.replace(
    /<svg\b[^>]*>/iu,
    match => `${match}<style>${supplementalCss}</style>`,
  );
}

export function svgViewport(svgSource) {
  const tag = svgSource.match(/<svg\b[^>]*>/iu)?.[0] ?? '';
  const width = Number.parseFloat(tag.match(/\bwidth=['"]?([0-9.]+)/iu)?.[1] ?? '0');
  const height = Number.parseFloat(tag.match(/\bheight=['"]?([0-9.]+)/iu)?.[1] ?? '0');
  if (width > 0 && height > 0) return { width, height };
  const viewBox = tag.match(/\bviewBox=['"]?\s*[-0-9.]+\s+[-0-9.]+\s+([0-9.]+)\s+([0-9.]+)/iu);
  if (viewBox) return { width: Number(viewBox[1]), height: Number(viewBox[2]) };
  throw new Error('SVG width/height 또는 viewBox를 해석하지 못했습니다.');
}

function optionValue(args, name) {
  const index = args.indexOf(name);
  return index === -1 ? undefined : args[index + 1];
}

export function findChrome(configured) {
  const candidates = configured ? [configured] : [
    process.env.VISUAL_SWEEP_CHROME,
    '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
    '/Applications/Chromium.app/Contents/MacOS/Chromium',
    'google-chrome',
    'google-chrome-stable',
    'chromium',
    'chromium-browser',
  ].filter(Boolean);
  for (const candidate of candidates) {
    if (existsSync(candidate)) return resolve(candidate);
    for (const directory of (process.env.PATH ?? '').split(delimiter).filter(Boolean)) {
      const executable = resolve(directory, candidate);
      if (existsSync(executable)) return executable;
      if (process.platform === 'win32' && existsSync(`${executable}.exe`)) return `${executable}.exe`;
    }
  }
  throw new Error('Chrome/Chromium을 찾지 못했습니다. VISUAL_SWEEP_CHROME으로 실행 경로를 지정하세요.');
}

export async function recoverUnavailableLocalBoldFaces(page) {
  return page.evaluate(async () => {
    const familyKey = value => value.trim().replace(/^(['"])(.*)\1$/u, '$2').toLowerCase();
    const weightValue = value => {
      if (value === 'normal' || value === '') return 400;
      if (value === 'bold') return 700;
      return /^\d+$/u.test(value) ? Number(value) : null;
    };
    const faces = [...document.fonts];
    const recovered = [];
    for (const sheet of document.styleSheets) {
      let rules;
      try {
        rules = [...sheet.cssRules];
      } catch {
        // Cross-origin sheets are not owned by this SVG capture.
        continue;
      }
      for (const rule of rules) {
        if (rule.type !== CSSRule.FONT_FACE_RULE) continue;
        const source = rule.style.getPropertyValue('src').trim();
        const remainder = source.replace(
          /local\(\s*(?:"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|[^)]*)\s*\)/giu,
          '',
        ).replace(/[\s,]/gu, '');
        // Never remove embedded/web font rules or change an available real bold face.
        if (!source || remainder !== '') continue;
        const family = familyKey(rule.style.getPropertyValue('font-family'));
        const weight = weightValue(rule.style.getPropertyValue('font-weight').trim());
        if (weight === null || weight < 600) continue;
        const style = rule.style.getPropertyValue('font-style').trim() || 'normal';
        const stretch = rule.style.getPropertyValue('font-stretch').trim() || 'normal';
        const failed = faces.find(face => (
          familyKey(face.family) === family
          && weightValue(face.weight) === weight
          && face.style === style
          && face.stretch === stretch
          && face.status === 'error'
        ));
        if (!failed) continue;
        let regular;
        for (const face of faces) {
          if (familyKey(face.family) !== family || weightValue(face.weight) !== 400
            || face.style !== failed.style || face.stretch !== failed.stretch
            || face.unicodeRange !== failed.unicodeRange) continue;
          try {
            await face.load();
            regular = face;
            break;
          } catch {
            // Keep searching; failure alone must not discard the bold declaration.
          }
        }
        if (!regular) continue;
        // A failed explicit bold face can send Blink to LastResort instead of the
        // safe regular alias. Removing only that rule permits synthetic bold from
        // the loaded regular face without rewriting the SVG's text or geometry.
        const index = [...sheet.cssRules].indexOf(rule);
        if (index < 0) continue;
        sheet.deleteRule(index);
        recovered.push({ family, weight, fallback: 'synthetic-bold-from-loaded-regular' });
      }
    }
    if (recovered.length > 0) {
      // Flush style invalidation before waiting for the replacement font selection.
      document.documentElement.getBoundingClientRect();
      await document.fonts.ready;
    }
    return recovered;
  });
}

function rasterTimeout() {
  const timeoutMs = Number(process.env.RHWP_VISUAL_RASTER_TIMEOUT_MS ?? '30000');
  if (!Number.isSafeInteger(timeoutMs) || timeoutMs <= 0) {
    throw new Error('RHWP_VISUAL_RASTER_TIMEOUT_MS는 양의 정수여야 합니다.');
  }
  return timeoutMs;
}

async function launchRasterBrowser(chrome, profileDir) {
  const timeoutMs = rasterTimeout();
  const studioRequire = createRequire(resolve(ROOT, 'rhwp-studio/package.json'));
  let puppeteerPath;
  try {
    puppeteerPath = studioRequire.resolve('puppeteer-core');
  } catch {
    throw new Error('puppeteer-core가 없습니다. npm --prefix rhwp-studio ci를 먼저 실행하세요.');
  }
  const { default: puppeteer } = await import(pathToFileURL(puppeteerPath).href);
  return puppeteer.launch({
    executablePath: chrome,
    headless: true,
    userDataDir: profileDir,
    timeout: timeoutMs,
    protocolTimeout: timeoutMs,
    args: ['--disable-gpu', '--hide-scrollbars', '--allow-file-access-from-files'],
  });
}

async function closeRasterBrowser(browser) {
  if (!browser) return;
  try {
    await browser.close();
  } finally {
    // 이 실행의 Chrome만 종료하고 남은 자식 출력 스트림도 닫는다.
    for (const stream of browser.process()?.stdio ?? []) stream?.destroy();
  }
}

async function renderWithChrome({ browser, htmlPath, outputPath, viewport, zoom }) {
  const timeoutMs = rasterTimeout();
  // 시작 때의 빈 탭도 재사용한다. 캡처마다 닫아 문서 DOM·글꼴 선언이 쌓이지 않는다.
  const initialPages = await browser.pages();
  const page = initialPages.find(candidate => candidate.url() === 'about:blank')
    ?? await browser.newPage();
  try {
    await page.setViewport({
      width: Math.ceil(viewport.width),
      height: Math.ceil(viewport.height),
      deviceScaleFactor: zoom,
    });
    await page.goto(pathToFileURL(htmlPath).href, { waitUntil: 'load', timeout: timeoutMs });
    await page.evaluate(() => document.fonts.ready.then(() => undefined));
    const recoveredFontFaces = await recoverUnavailableLocalBoldFaces(page);
    const openPageCount = (await browser.pages()).length;
    await page.screenshot({ path: outputPath, type: 'png' });
    return { recoveredFontFaces, openPageCount };
  } finally {
    await page.close();
  }
}

export function rasterJobs(args) {
  const batch = optionValue(args, '--batch-json');
  const input = optionValue(args, '--input');
  const output = optionValue(args, '--output');
  const zoom = Number(optionValue(args, '--zoom') ?? '1');
  if (batch && (input || output)) throw new Error('--batch-json과 --input/--output은 함께 쓰지 않습니다.');
  const jobs = batch ? JSON.parse(readFileSync(batch, 'utf8')) : [{ input, output, zoom }];
  if (!Array.isArray(jobs) || jobs.length === 0) throw new Error('캡처할 SVG 목록이 비어 있습니다.');
  return jobs.map(job => {
    if (!job || typeof job.input !== 'string' || !job.input
      || typeof job.output !== 'string' || !job.output
      || !Number.isFinite(Number(job.zoom ?? zoom)) || Number(job.zoom ?? zoom) <= 0) {
      throw new Error('SVG input/output과 양수 zoom이 필요합니다.');
    }
    return { input: job.input, output: job.output, zoom: Number(job.zoom ?? zoom) };
  });
}

async function captureSvg({ input, output, zoom }, session) {
  const inputPath = resolve(input);
  const outputPath = resolve(output);
  const svgSource = readFileSync(inputPath, 'utf8');
  const projectionSource = readFileSync(resolve(ROOT, PROJECTION_PATH), 'utf8');
  // 내장 글꼴을 같은 바이트의 공용 파일로 먼저 바꿔 규칙 탐색의 입력 크기를 줄인다.
  const prepared = externalizeEmbeddedFontSources(svgSource);
  const rules = selectWebfontRules(prepared.svg, parseWebfontRules(projectionSource));
  const webfontCss = buildWebfontCss(ROOT, rules);
  const preparedSvg = prepareSvgForWebfontRaster(prepared.svg, webfontCss);
  const viewport = svgViewport(preparedSvg);
  const wrapperPath = resolve(dirname(inputPath), `.${basename(inputPath)}.webfont-${process.pid}.html`);
  const html = `<!doctype html><meta charset="utf-8"><style>html,body{margin:0;padding:0;overflow:hidden;width:${viewport.width}px;height:${viewport.height}px}svg{display:block}</style>${preparedSvg}`;
  writeFileSync(wrapperPath, html);
  session.wrappers.add(wrapperPath);
  let recoveredFontFaces;
  try {
    session.browser ??= await launchRasterBrowser(session.chrome, session.profileDir);
    const capture = await renderWithChrome({
      browser: session.browser,
      htmlPath: wrapperPath,
      outputPath,
      viewport,
      zoom,
    });
    recoveredFontFaces = capture.recoveredFontFaces;
    session.openPageCount = capture.openPageCount;
  } finally {
    rmSync(wrapperPath, { force: true });
    session.wrappers.delete(wrapperPath);
  }
  console.log(JSON.stringify({
    rasterizer: 'chrome-webfont',
    input: inputPath,
    output: outputPath,
    zoom,
    viewport,
    projectionSha256: createHash('sha256').update(projectionSource).digest('hex'),
    appliedRuleIds: rules.map(rule => rule.ruleId),
    preservedFontFaceFamilies: [...declaredFontFaceFamilies(svgSource)],
    recoveredFontFaces,
    terminalFallbackFamily: TERMINAL_FALLBACK_FAMILY,
    externalizedFontPayloads: prepared.payloads,
    browserPid: session.browser.process()?.pid ?? null,
    openPageCount: session.openPageCount,
    pagesAfterCapture: (await session.browser.pages()).length,
  }));
}

async function main() {
  const jobs = rasterJobs(process.argv);
  const workers = Number(optionValue(process.argv, '--workers') ?? '1');
  if (!Number.isSafeInteger(workers) || workers < 1) {
    throw new Error('--workers는 양의 정수여야 합니다.');
  }
  if (new Set(jobs.map(job => resolve(job.output))).size !== jobs.length) {
    throw new Error('같은 PNG에 여러 캡처 작업을 배정할 수 없습니다.');
  }
  if (workers > 1 && jobs.length > 1) {
    const { runParallelRasterJobs } = await import('./rasterize-svg-webfont-workers.mjs');
    await runParallelRasterJobs(jobs, workers, fileURLToPath(import.meta.url),
      findChrome(optionValue(process.argv, '--chrome')));
    return;
  }
  const session = {
    chrome: findChrome(optionValue(process.argv, '--chrome')),
    profileDir: resolve(dirname(resolve(jobs[0].output)), `.webfont-chrome-profile-${process.pid}`),
    browser: null,
    wrappers: new Set(),
  };
  let closing = false;
  const onSignal = async signal => {
    if (closing) return;
    closing = true;
    try {
      await closeRasterBrowser(session.browser);
    } finally {
      for (const wrapper of session.wrappers) rmSync(wrapper, { force: true });
      rmSync(session.profileDir, { recursive: true, force: true });
      process.exit(signal === 'SIGINT' ? 130 : 143);
    }
  };
  const interrupt = () => { void onSignal('SIGINT'); };
  const terminate = () => { void onSignal('SIGTERM'); };
  // Windows의 강제 terminate 대신 부모가 stdin으로 정상 정리를 요청한다.
  let control = '';
  const onControl = chunk => {
    control += chunk.toString('utf8');
    if (control.includes('\n')) {
      if (control.trim() === 'stop') terminate();
      control = '';
    }
  };
  if (optionValue(process.argv, '--batch-json')) {
    process.stdin.on('data', onControl);
    process.stdin.unref?.();
  }
  process.once('SIGINT', interrupt);
  process.once('SIGTERM', terminate);
  try {
    // 각 쪽의 PNG 완료를 즉시 알린다. 호출자는 기존 쪽별 checkpoint를 계속 저장한다.
    for (const job of jobs) await captureSvg(job, session);
  } finally {
    process.removeListener('SIGINT', interrupt);
    process.removeListener('SIGTERM', terminate);
    process.stdin.removeListener('data', onControl);
    process.stdin.pause();
    await closeRasterBrowser(session.browser);
    for (const wrapper of session.wrappers) rmSync(wrapper, { force: true });
    rmSync(session.profileDir, { recursive: true, force: true });
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(error => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  });
}
