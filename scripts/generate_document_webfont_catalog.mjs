#!/usr/bin/env node

import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const BEGIN = '  // BEGIN SURVEY_WEBFONT_CATALOG';
const END = '  // END SURVEY_WEBFONT_CATALOG';
const INSERT_BEFORE = '  // === D2 Coding (OFL, 로컬) ===';

function optionValue(name) {
  const index = process.argv.indexOf(name);
  return index === -1 ? undefined : process.argv[index + 1];
}

function quote(value) {
  return `'${value.replaceAll('\\', '\\\\').replaceAll("'", "\\'")}'`;
}

function parseTsv(source) {
  const [header, ...rows] = source.trimEnd().split('\n');
  const columns = header.split('\t');
  const index = Object.fromEntries(columns.map((name, position) => [name, position]));
  const required = ['font', 'status', 'webfont_usable', 'download_url'];
  for (const name of required) {
    if (index[name] === undefined) {
      throw new Error(`TSV 필수 열이 없습니다: ${name}`);
    }
  }
  return rows.map(row => row.split('\t')).filter(fields => (
    fields[index.status] === 'available' && fields[index.webfont_usable] === '가능'
  )).map(fields => ({
    name: fields[index.font],
    url: fields[index.download_url],
  }));
}

function fontFormat(url) {
  const pathname = new URL(url).pathname.toLowerCase();
  if (pathname.endsWith('.woff2')) return undefined;
  if (pathname.endsWith('.woff')) return 'woff';
  if (pathname.endsWith('.ttf')) return 'truetype';
  if (pathname.endsWith('.otf')) return 'opentype';
  throw new Error(`지원하지 않는 웹폰트 확장자: ${url}`);
}

const input = optionValue('--input');
const target = optionValue('--target') ?? 'rhwp-studio/src/core/font-loader.ts';
if (!input) {
  throw new Error('사용법: node scripts/generate_document_webfont_catalog.mjs --input <survey.tsv> [--target <font-loader.ts>]');
}

const targetPath = resolve(target);
const source = readFileSync(targetPath, 'utf8');
const baseSource = source.replace(
  new RegExp(`${BEGIN}[\\s\\S]*?${END}\\n?`, 'u'),
  '',
);
if (!baseSource.includes(INSERT_BEFORE)) {
  throw new Error(`삽입 기준점을 찾지 못했습니다: ${INSERT_BEFORE}`);
}

const knownNames = new Set(Array.from(baseSource.matchAll(/name:\s*'([^']+)'/gu), match => match[1]));
const catalog = parseTsv(readFileSync(resolve(input), 'utf8'));
const uniqueCatalog = Array.from(new Map(catalog.map(entry => [entry.name, entry])).values());
const missingEntries = uniqueCatalog.filter(entry => !knownNames.has(entry.name));
for (const entry of missingEntries) {
  const parsed = new URL(entry.url);
  if (parsed.protocol !== 'https:') {
    throw new Error(`HTTPS가 아닌 웹폰트 URL: ${entry.url}`);
  }
}

const generated = [
  BEGIN,
  '  // 조사 증적에서 웹폰트 사용 가능으로 판정된 문서 글꼴. 생성 스크립트로 갱신한다.',
  ...missingEntries.map(entry => {
    const format = fontFormat(entry.url);
    return `  { name: ${quote(entry.name)}, file: ${quote(entry.url)}${format ? `, format: '${format}'` : ''} },`;
  }),
  END,
  '',
].join('\n');

writeFileSync(targetPath, baseSource.replace(INSERT_BEFORE, `${generated}${INSERT_BEFORE}`));
console.log(`조사 대상 ${uniqueCatalog.length}건 중 기존 ${uniqueCatalog.length - missingEntries.length}건, 새 카탈로그 ${missingEntries.length}건을 반영했습니다.`);
