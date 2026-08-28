#!/usr/bin/env node

import { createHash } from 'node:crypto';
import { existsSync, readFileSync, rmSync, statSync, unlinkSync, writeFileSync } from 'node:fs';
import { basename, dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { spawn } from 'node:child_process';

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(SCRIPT_DIR, '..');
const PROJECTION_PATH = 'rhwp-studio/src/core/generated/font-rule-projections/webfont-supply.ts';
const TERMINAL_FALLBACK_FAMILY = '__rhwp_visual_sweep_noto_sans_kr__';

function cssString(value) {
  return `"${value.replaceAll('\\', '\\\\').replaceAll('"', '\\"')}"`;
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
  const selected = new Map();
  for (const rule of rules) {
    const sourceFace = rule.sourceFace.toLocaleLowerCase('en-US');
    if (lowerSource.includes(sourceFace)) {
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
  return `${fontList.trim()}, ${cssString(TERMINAL_FALLBACK_FAMILY)}`;
}

export function prepareSvgForWebfontRaster(svgSource, webfontCss) {
  const withoutLocalFaces = svgSource.replace(/@font-face\s*\{[^{}]*\}/giu, '');
  const withAttributeFallback = withoutLocalFaces.replace(
    /font-family=(['"])(.*?)\1/giu,
    (_match, quote, fontList) => `font-family=${quote}${appendTerminalFallback(fontList)}${quote}`,
  );
  const withCssFallback = withAttributeFallback.replace(
    /(font-family\s*:\s*)([^;}]+)/giu,
    (_match, prefix, fontList) => `${prefix}${appendTerminalFallback(fontList)}`,
  );
  return withCssFallback.replace(
    /<svg\b[^>]*>/iu,
    match => `${match}<style>${webfontCss}</style>`,
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

function findChrome(configured) {
  if (configured) return configured;
  const candidates = [
    process.env.VISUAL_SWEEP_CHROME,
    '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
    '/Applications/Chromium.app/Contents/MacOS/Chromium',
    'google-chrome',
    'google-chrome-stable',
    'chromium',
    'chromium-browser',
  ].filter(Boolean);
  for (const candidate of candidates) {
    if (!candidate.includes('/') || existsSync(candidate)) return candidate;
  }
  throw new Error('Chrome/Chromium을 찾지 못했습니다. VISUAL_SWEEP_CHROME으로 실행 경로를 지정하세요.');
}

async function renderWithChrome({ chrome, htmlPath, outputPath, viewport, zoom, profileDir }) {
  const args = [
    '--headless=new',
    '--disable-gpu',
    '--hide-scrollbars',
    '--no-first-run',
    '--no-default-browser-check',
    '--allow-file-access-from-files',
    `--user-data-dir=${profileDir}`,
    '--virtual-time-budget=10000',
    `--force-device-scale-factor=${zoom}`,
    `--window-size=${Math.ceil(viewport.width)},${Math.ceil(viewport.height)}`,
    `--screenshot=${outputPath}`,
    pathToFileURL(htmlPath).href,
  ];
  await new Promise((resolvePromise, reject) => {
    const child = spawn(chrome, args, { stdio: ['ignore', 'ignore', 'pipe'] });
    let stderr = '';
    let completed = false;
    let screenshotReady = false;
    let killTimer;
    const finish = error => {
      if (completed) return;
      completed = true;
      clearInterval(poll);
      clearTimeout(timeout);
      clearTimeout(killTimer);
      if (error) reject(error);
      else resolvePromise();
    };
    const terminate = () => {
      if (child.killed) return;
      child.kill('SIGTERM');
      killTimer = setTimeout(() => child.kill('SIGKILL'), 2000);
    };
    const poll = setInterval(() => {
      if (existsSync(outputPath) && statSync(outputPath).size > 0) {
        screenshotReady = true;
        terminate();
      }
    }, 100);
    const timeout = setTimeout(() => {
      terminate();
      finish(new Error(`Chrome webfont raster timeout: ${stderr.trim()}`));
    }, 30000);
    child.stderr.on('data', chunk => { stderr += chunk; });
    child.on('error', error => finish(error));
    child.on('close', code => {
      if (screenshotReady && existsSync(outputPath) && statSync(outputPath).size > 0) {
        finish();
      } else if (!completed) {
        finish(new Error(`Chrome webfont raster failed (exit=${code}): ${stderr.trim()}`));
      }
    });
  });
}

async function main() {
  const input = optionValue(process.argv, '--input');
  const output = optionValue(process.argv, '--output');
  const zoom = Number(optionValue(process.argv, '--zoom') ?? '1');
  const configuredChrome = optionValue(process.argv, '--chrome');
  if (!input || !output || !Number.isFinite(zoom) || zoom <= 0) {
    throw new Error('사용법: rasterize-svg-webfonts.mjs --input <svg> --output <png> [--zoom <양수>] [--chrome <경로>]');
  }

  const inputPath = resolve(input);
  const outputPath = resolve(output);
  const svgSource = readFileSync(inputPath, 'utf8');
  const projectionSource = readFileSync(resolve(ROOT, PROJECTION_PATH), 'utf8');
  const rules = selectWebfontRules(svgSource, parseWebfontRules(projectionSource));
  const webfontCss = buildWebfontCss(ROOT, rules);
  const preparedSvg = prepareSvgForWebfontRaster(svgSource, webfontCss);
  const viewport = svgViewport(preparedSvg);
  const wrapperPath = resolve(dirname(inputPath), `.${basename(inputPath)}.webfont-${process.pid}.html`);
  const profileDir = resolve(dirname(outputPath), `.webfont-chrome-profile-${process.pid}`);
  const html = `<!doctype html><meta charset="utf-8"><style>html,body{margin:0;padding:0;overflow:hidden;width:${viewport.width}px;height:${viewport.height}px}svg{display:block}</style>${preparedSvg}`;
  writeFileSync(wrapperPath, html);
  try {
    await renderWithChrome({
      chrome: findChrome(configuredChrome),
      htmlPath: wrapperPath,
      outputPath,
      viewport,
      zoom,
      profileDir,
    });
  } finally {
    unlinkSync(wrapperPath, { force: true });
    rmSync(profileDir, { recursive: true, force: true });
  }
  console.log(JSON.stringify({
    rasterizer: 'chrome-webfont',
    input: inputPath,
    output: outputPath,
    zoom,
    viewport,
    projectionSha256: createHash('sha256').update(projectionSource).digest('hex'),
    appliedRuleIds: rules.map(rule => rule.ruleId),
    terminalFallbackFamily: TERMINAL_FALLBACK_FAMILY,
  }));
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(error => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  });
}
