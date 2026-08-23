#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const METRIC_SOURCE = path.join(ROOT, 'src', 'renderer', 'font_metrics_data.rs');
const W1_BASELINE = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4939',
  'font_rule_baseline.json',
);
const OUTPUT = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4964',
  'font_metric_pre_split_baseline.json',
);
const GENERATOR_VERSION = '1.0.0';
const SOURCE_COMMIT = 'd1ad0eb8784dbc55f0796e2ba8775f7363247b91';
const OVERLAY_NAMES = [
  'HanyangSinMyeongJo',
  'HanyangJungGothic',
  'HanyangKyunMyeongJo',
  'HanyangKyunGothic',
  'HumanMyeongJo',
];

function canonicalValue(value) {
  if (Array.isArray(value)) return value.map(canonicalValue);
  if (value === null || typeof value !== 'object') return value;
  return Object.fromEntries(
    Object.keys(value)
      .sort()
      .map(key => [key, canonicalValue(value[key])]),
  );
}

export function canonicalJson(value) {
  return `${JSON.stringify(canonicalValue(value), null, 2)}\n`;
}

export function sha256Text(value) {
  return crypto.createHash('sha256').update(value, 'utf8').digest('hex');
}

function sha256File(file) {
  return crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
}

function decodeQuotedString(value) {
  return JSON.parse(`"${value}"`);
}

function extractBalanced(source, startAt, openCharacter, closeCharacter) {
  const openAt = source.indexOf(openCharacter, startAt);
  if (openAt === -1) throw new Error(`opening ${openCharacter} not found`);
  let depth = 0;
  let quote = null;
  let escaped = false;
  let lineComment = false;
  let blockComment = false;
  for (let index = openAt; index < source.length; index += 1) {
    const character = source[index];
    const next = source[index + 1];
    if (lineComment) {
      if (character === '\n') lineComment = false;
      continue;
    }
    if (blockComment) {
      if (character === '*' && next === '/') {
        blockComment = false;
        index += 1;
      }
      continue;
    }
    if (quote !== null) {
      if (escaped) escaped = false;
      else if (character === '\\') escaped = true;
      else if (character === quote) quote = null;
      continue;
    }
    if (character === '/' && next === '/') {
      lineComment = true;
      index += 1;
      continue;
    }
    if (character === '/' && next === '*') {
      blockComment = true;
      index += 1;
      continue;
    }
    if (character === '"' || character === "'") {
      quote = character;
      continue;
    }
    if (character === openCharacter) depth += 1;
    if (character === closeCharacter) {
      depth -= 1;
      if (depth === 0) {
        return {
          body: source.slice(openAt + 1, index),
          closeAt: index,
          openAt,
        };
      }
    }
  }
  throw new Error(`closing ${closeCharacter} not found`);
}

function parseInteger(value) {
  return Number.parseInt(value.replaceAll('_', ''), value.toLowerCase().startsWith('0x') ? 16 : 10);
}

function parseIntegerArrayDeclarations(source) {
  const arrays = new Map();
  const pattern = /(?:pub(?:\([^)]*\))?\s+)?static\s+([A-Z0-9_]+):\s*\[(u8|u16);\s*(\d+)\]\s*=\s*\[/g;
  for (const match of source.matchAll(pattern)) {
    const balanced = extractBalanced(source, match.index + match[0].length - 1, '[', ']');
    const values = [...balanced.body.matchAll(/\b(?:0x[0-9a-fA-F_]+|\d[\d_]*)\b/g)]
      .map(value => parseInteger(value[0]));
    const declaredLength = Number.parseInt(match[3], 10);
    if (values.length !== declaredLength) {
      throw new Error(`${match[1]} parsed ${values.length}/${declaredLength} integer values`);
    }
    arrays.set(match[1], {
      elementType: match[2],
      values,
    });
  }
  return arrays;
}

function parseLatinRangeDeclarations(source, integerArrays) {
  const ranges = new Map();
  const pattern = /static\s+([A-Z0-9_]+):\s*\[LatinRange;\s*(\d+)\]\s*=\s*\[/g;
  for (const match of source.matchAll(pattern)) {
    const balanced = extractBalanced(source, match.index + match[0].length - 1, '[', ']');
    const rows = [...balanced.body.matchAll(
      /LatinRange\s*\{\s*start:\s*(0x[0-9a-fA-F_]+|\d[\d_]*),\s*end:\s*(0x[0-9a-fA-F_]+|\d[\d_]*),\s*widths:\s*&([A-Z0-9_]+),\s*\}/g,
    )].map(row => {
      const start = parseInteger(row[1]);
      const end = parseInteger(row[2]);
      const widthsSymbol = row[3];
      const widthArray = integerArrays.get(widthsSymbol);
      if (widthArray === undefined) throw new Error(`${match[1]} references missing ${widthsSymbol}`);
      if (widthArray.elementType !== 'u16') throw new Error(`${widthsSymbol} must be u16`);
      if (widthArray.values.length !== end - start + 1) {
        throw new Error(`${widthsSymbol} length does not cover U+${start.toString(16)}..U+${end.toString(16)}`);
      }
      return { start, end, widthsSymbol };
    });
    const declaredLength = Number.parseInt(match[2], 10);
    if (rows.length !== declaredLength) {
      throw new Error(`${match[1]} parsed ${rows.length}/${declaredLength} Latin ranges`);
    }
    ranges.set(match[1], rows);
  }
  return ranges;
}

function parseHangulDeclarations(source, integerArrays) {
  const hangul = new Map();
  const pattern = /static\s+([A-Z0-9_]+):\s*HangulMetric\s*=\s*HangulMetric\s*\{/g;
  for (const match of source.matchAll(pattern)) {
    const balanced = extractBalanced(source, match.index + match[0].length - 1, '{', '}');
    const field = name => {
      const value = balanced.body.match(new RegExp(`${name}:\\s*(\\d+)`));
      if (value === null) throw new Error(`${match[1]} missing ${name}`);
      return Number.parseInt(value[1], 10);
    };
    const symbol = name => {
      const value = balanced.body.match(new RegExp(`${name}:\\s*&([A-Z0-9_]+)`));
      if (value === null) throw new Error(`${match[1]} missing ${name}`);
      if (!integerArrays.has(value[1])) throw new Error(`${match[1]} references missing ${value[1]}`);
      return value[1];
    };
    const value = {
      choGroups: field('cho_groups'),
      jungGroups: field('jung_groups'),
      jongGroups: field('jong_groups'),
      choMapSymbol: symbol('cho_map'),
      jungMapSymbol: symbol('jung_map'),
      jongMapSymbol: symbol('jong_map'),
      widthsSymbol: symbol('widths'),
    };
    const expectedWidthCount = value.choGroups * value.jungGroups * value.jongGroups;
    if (integerArrays.get(value.choMapSymbol).values.length !== 19) {
      throw new Error(`${match[1]} choseong map must contain 19 values`);
    }
    if (integerArrays.get(value.jungMapSymbol).values.length !== 21) {
      throw new Error(`${match[1]} jungseong map must contain 21 values`);
    }
    if (integerArrays.get(value.jongMapSymbol).values.length !== 28) {
      throw new Error(`${match[1]} jongseong map must contain 28 values`);
    }
    if (integerArrays.get(value.widthsSymbol).values.length !== expectedWidthCount) {
      throw new Error(`${match[1]} width grid does not match declared groups`);
    }
    hangul.set(match[1], value);
  }
  return hangul;
}

function parseMetricEntries(source, latinRanges, hangul) {
  const declarationAt = source.indexOf('static FONT_METRICS:');
  if (declarationAt === -1) throw new Error('FONT_METRICS declaration not found');
  const assignmentAt = source.indexOf('=', declarationAt);
  const balanced = extractBalanced(source, assignmentAt, '[', ']');
  const pattern = /FontMetric\s*\{\s*name:\s*"((?:\\.|[^"\\])*)",\s*bold:\s*(true|false),\s*italic:\s*(true|false),\s*em_size:\s*(\d+),\s*latin_ranges:\s*&([A-Z0-9_]+),\s*hangul:\s*(?:Some\(&([A-Z0-9_]+)\)|None),\s*\}/g;
  const entries = [...balanced.body.matchAll(pattern)].map((match, index) => {
    if (!latinRanges.has(match[5])) throw new Error(`metric ${index} references missing ${match[5]}`);
    if (match[6] !== undefined && !hangul.has(match[6])) {
      throw new Error(`metric ${index} references missing ${match[6]}`);
    }
    return {
      index,
      name: decodeQuotedString(match[1]),
      bold: match[2] === 'true',
      italic: match[3] === 'true',
      emSize: Number.parseInt(match[4], 10),
      latinRangesSymbol: match[5],
      hangulSymbol: match[6] ?? null,
    };
  });
  const declared = Number.parseInt(
    source.slice(declarationAt, assignmentAt).match(/\[FontMetric;\s*(\d+)\]/)?.[1] ?? '',
    10,
  );
  if (!Number.isInteger(declared) || entries.length !== declared) {
    throw new Error(`FONT_METRICS parsed ${entries.length}/${declared}`);
  }
  return entries;
}

function parseAliases(source) {
  const functionAt = source.indexOf('fn resolve_metric_alias(');
  if (functionAt === -1) throw new Error('resolve_metric_alias function not found');
  const balanced = extractBalanced(source, functionAt, '{', '}');
  const aliases = [];
  const pattern = /((?:"(?:\\.|[^"\\])*"\s*(?:\|\s*)?)+)=>\s*"((?:\\.|[^"\\])*)"\s*,/gs;
  for (const match of balanced.body.matchAll(pattern)) {
    const target = decodeQuotedString(match[2]);
    for (const sourceMatch of match[1].matchAll(/"((?:\\.|[^"\\])*)"/g)) {
      aliases.push({ source: decodeQuotedString(sourceMatch[1]), target });
    }
  }
  if (aliases.length === 0) throw new Error('resolve_metric_alias produced zero aliases');
  return aliases;
}

function metricWidth(model, metric, codepoint) {
  if (codepoint >= 0xac00 && codepoint <= 0xd7a3) {
    if (metric.hangulSymbol === null) return null;
    const data = model.hangul.get(metric.hangulSymbol);
    const index = codepoint - 0xac00;
    const cho = Math.floor(index / (21 * 28));
    const jung = Math.floor((index % (21 * 28)) / 28);
    const jong = index % 28;
    const choMap = model.integerArrays.get(data.choMapSymbol).values;
    const jungMap = model.integerArrays.get(data.jungMapSymbol).values;
    const jongMap = model.integerArrays.get(data.jongMapSymbol).values;
    const groupIndex = choMap[cho] * data.jungGroups * data.jongGroups
      + jungMap[jung] * data.jongGroups
      + jongMap[jong];
    return model.integerArrays.get(data.widthsSymbol).values[groupIndex] ?? null;
  }
  for (const range of model.latinRanges.get(metric.latinRangesSymbol)) {
    if (codepoint < range.start || codepoint > range.end) continue;
    const width = model.integerArrays.get(range.widthsSymbol).values[codepoint - range.start];
    return width > 0 ? width : null;
  }
  return null;
}

function widthProjectionHash(model) {
  const hash = crypto.createHash('sha256');
  let dataBearingCodepointCount = 0;
  for (const metric of model.entries) {
    const ranges = model.latinRanges.get(metric.latinRangesSymbol);
    const latinCodepoints = [];
    const seen = new Set();
    for (const range of ranges) {
      if (range.end >= 0xac00 && range.start <= 0xd7a3) {
        throw new Error(`${metric.latinRangesSymbol} overlaps the Hangul-first lookup domain`);
      }
      const widths = model.integerArrays.get(range.widthsSymbol).values;
      for (let offset = 0; offset < widths.length; offset += 1) {
        const codepoint = range.start + offset;
        if (seen.has(codepoint)) {
          throw new Error(`${metric.latinRangesSymbol} contains overlapping U+${codepoint.toString(16)}`);
        }
        seen.add(codepoint);
        latinCodepoints.push({ codepoint, width: widths[offset] > 0 ? widths[offset] : null });
      }
    }
    latinCodepoints.sort((left, right) => left.codepoint - right.codepoint);
    const pairCount = latinCodepoints.length + 11172;
    const buffer = Buffer.allocUnsafe(4 + pairCount * 8);
    let offset = 0;
    buffer.writeUInt32LE(metric.index, offset);
    offset += 4;
    for (const pair of latinCodepoints) {
      buffer.writeUInt32LE(pair.codepoint, offset);
      buffer.writeInt32LE(pair.width ?? -1, offset + 4);
      offset += 8;
    }
    for (let codepoint = 0xac00; codepoint <= 0xd7a3; codepoint += 1) {
      buffer.writeUInt32LE(codepoint, offset);
      buffer.writeInt32LE(metricWidth(model, metric, codepoint) ?? -1, offset + 4);
      offset += 8;
    }
    dataBearingCodepointCount += pairCount;
    hash.update(buffer);
  }
  return {
    dataBearingCodepointCount,
    sha256: hash.digest('hex'),
  };
}

function metricDataProjection(model) {
  return model.entries.map(metric => ({
    index: metric.index,
    latinRanges: model.latinRanges.get(metric.latinRangesSymbol).map(range => ({
      start: range.start,
      end: range.end,
      widths: model.integerArrays.get(range.widthsSymbol).values,
    })),
    hangul: metric.hangulSymbol === null
      ? null
      : (() => {
        const data = model.hangul.get(metric.hangulSymbol);
        return {
          choGroups: data.choGroups,
          jungGroups: data.jungGroups,
          jongGroups: data.jongGroups,
          choMap: model.integerArrays.get(data.choMapSymbol).values,
          jungMap: model.integerArrays.get(data.jungMapSymbol).values,
          jongMap: model.integerArrays.get(data.jongMapSymbol).values,
          widths: model.integerArrays.get(data.widthsSymbol).values,
        };
      })(),
  }));
}

function lookupProjection(entries, aliases) {
  const aliasMap = new Map(aliases.map(alias => [alias.source, alias.target]));
  const inputNames = [];
  const seen = new Set();
  for (const name of entries.map(entry => entry.name)) {
    if (!seen.has(name)) {
      seen.add(name);
      inputNames.push(name);
    }
  }
  for (const { source } of aliases) {
    if (!seen.has(source)) {
      seen.add(source);
      inputNames.push(source);
    }
  }
  inputNames.push('__rhwp_font_metric_lineage_unregistered__');

  const projection = [];
  for (const inputName of inputNames) {
    const resolvedName = aliasMap.get(inputName) ?? inputName;
    for (const bold of [false, true]) {
      for (const italic of [false, true]) {
        const exact = entries.find(
          entry => entry.name === resolvedName && entry.bold === bold && entry.italic === italic,
        );
        const boldOnly = entries.find(
          entry => entry.name === resolvedName && entry.bold === bold && entry.italic === false,
        );
        const first = entries.find(entry => entry.name === resolvedName);
        const selected = exact ?? boldOnly ?? first ?? null;
        projection.push({
          inputName,
          resolvedName,
          bold,
          italic,
          entryIndex: selected?.index ?? null,
          matchKind: exact !== undefined
            ? 'exact'
            : boldOnly !== undefined
              ? 'boldOnly'
              : first !== undefined
                ? 'nameFirst'
                : null,
          boldFallback: selected === null ? null : exact !== undefined || boldOnly !== undefined ? false : bold,
        });
      }
    }
  }
  return { inputNames, projection };
}

export function analyzeMetricSource(source) {
  const integerArrays = parseIntegerArrayDeclarations(source);
  const latinRanges = parseLatinRangeDeclarations(source, integerArrays);
  const hangul = parseHangulDeclarations(source, integerArrays);
  const entries = parseMetricEntries(source, latinRanges, hangul);
  const aliases = parseAliases(source);
  const model = { aliases, entries, hangul, integerArrays, latinRanges };
  const composition = entries.map(entry => ({
    index: entry.index,
    name: entry.name,
    bold: entry.bold,
    italic: entry.italic,
    emSize: entry.emSize,
    latinRangesSymbol: entry.latinRangesSymbol,
    hangulSymbol: entry.hangulSymbol,
  }));
  const lookup = lookupProjection(entries, aliases);
  const widths = widthProjectionHash(model);
  const styleCounts = { regular: 0, bold: 0, italic: 0, boldItalic: 0 };
  for (const entry of entries) {
    const key = entry.bold ? (entry.italic ? 'boldItalic' : 'bold') : (entry.italic ? 'italic' : 'regular');
    styleCounts[key] += 1;
  }
  return {
    aliasCount: aliases.length,
    composition,
    compositionSha256: sha256Text(canonicalJson(composition)),
    entryCount: entries.length,
    lookupInputCount: lookup.inputNames.length,
    lookupProjectionSha256: sha256Text(canonicalJson(lookup.projection)),
    metricDataSha256: sha256Text(canonicalJson(metricDataProjection(model))),
    styleCounts,
    uniqueNameCount: new Set(entries.map(entry => entry.name)).size,
    widthProjection: widths,
  };
}

export function assertMeasuredOverlayRegion(composition) {
  const overlay = composition.slice(-OVERLAY_NAMES.length);
  if (canonicalJson(overlay.map(entry => entry.name)) !== canonicalJson(OVERLAY_NAMES)) {
    throw new Error('expected #2430 overlays are not the final five metric entries');
  }
}

export function buildPreSplitBaseline(root = ROOT) {
  const metricSourcePath = path.join(root, 'src', 'renderer', 'font_metrics_data.rs');
  const w1BaselinePath = path.join(
    root,
    'mydocs',
    'tech',
    'investigations',
    'issue-4939',
    'font_rule_baseline.json',
  );
  const source = fs.readFileSync(metricSourcePath, 'utf8');
  const analysis = analyzeMetricSource(source);
  const w1 = JSON.parse(fs.readFileSync(w1BaselinePath, 'utf8'));
  if (analysis.entryCount !== w1.fontMetrics.entryCount) {
    throw new Error(`W1 entry count drift: ${analysis.entryCount}/${w1.fontMetrics.entryCount}`);
  }
  if (analysis.uniqueNameCount !== w1.fontMetrics.uniqueNameCount) {
    throw new Error(`W1 unique name drift: ${analysis.uniqueNameCount}/${w1.fontMetrics.uniqueNameCount}`);
  }
  if (canonicalJson(analysis.styleCounts) !== canonicalJson(w1.fontMetrics.styleCounts)) {
    throw new Error('W1 style count drift');
  }
  assertMeasuredOverlayRegion(analysis.composition);

  return {
    schemaVersion: '1.0',
    kind: 'font-metric-pre-split-baseline',
    sourceCommit: SOURCE_COMMIT,
    generatorVersion: GENERATOR_VERSION,
    inputs: [
      {
        path: 'src/renderer/font_metrics_data.rs',
        sha256: sha256File(metricSourcePath),
      },
      {
        path: 'mydocs/tech/investigations/issue-4939/font_rule_baseline.json',
        sha256: sha256File(w1BaselinePath),
      },
    ],
    fontMetrics: {
      entryCount: analysis.entryCount,
      uniqueNameCount: analysis.uniqueNameCount,
      styleCounts: analysis.styleCounts,
      historicalGeneratedRegion: {
        firstIndex: 0,
        lastIndex: analysis.entryCount - OVERLAY_NAMES.length - 1,
        entryCount: analysis.entryCount - OVERLAY_NAMES.length,
        provenanceCaveat: 'generated-region-does-not-imply-source-exact',
      },
      measuredOverlayRegion: {
        firstIndex: analysis.entryCount - OVERLAY_NAMES.length,
        lastIndex: analysis.entryCount - 1,
        entryCount: OVERLAY_NAMES.length,
        names: OVERLAY_NAMES,
        evidence: 'tools/task2430/EVIDENCE.md',
      },
    },
    exhaustiveDomain: {
      latin: 'every codepoint in every stored LatinRange, with zero encoded as None',
      hangul: 'U+AC00..U+D7A3 for every entry, including None when no HangulMetric exists',
      unsupported: 'range topology plus boundary tests preserve None outside stored ranges',
      evaluatedEntryCodepointPairs: analysis.widthProjection.dataBearingCodepointCount,
    },
    hashes: {
      compositionSha256: analysis.compositionSha256,
      metricDataSha256: analysis.metricDataSha256,
      widthProjectionSha256: analysis.widthProjection.sha256,
      lookupProjectionSha256: analysis.lookupProjectionSha256,
    },
    lookupContract: {
      aliasCount: analysis.aliasCount,
      inputNameCount: analysis.lookupInputCount,
      styleCombinationsPerInput: 4,
      order: ['name+bold+italic', 'name+bold+italic=false', 'name-first'],
    },
  };
}

export function compareBaseline(expected, actual) {
  const expectedText = canonicalJson(expected);
  const actualText = canonicalJson(actual);
  return expectedText === actualText
    ? []
    : ['font metric pre-split baseline differs; run --generate only in an approved baseline stage'];
}

function usage() {
  console.error('usage: node scripts/font_metric_lineage.mjs --generate|--check');
}

if (process.argv[1] !== undefined && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const mode = process.argv[2];
  if (!['--generate', '--check'].includes(mode) || process.argv.length !== 3) {
    usage();
    process.exit(2);
  }
  const baseline = buildPreSplitBaseline();
  if (mode === '--generate') {
    fs.mkdirSync(path.dirname(OUTPUT), { recursive: true });
    fs.writeFileSync(OUTPUT, canonicalJson(baseline), 'utf8');
    console.log(`generated ${path.relative(ROOT, OUTPUT)}`);
  } else {
    if (!fs.existsSync(OUTPUT)) throw new Error(`baseline does not exist: ${path.relative(ROOT, OUTPUT)}`);
    const expected = JSON.parse(fs.readFileSync(OUTPUT, 'utf8'));
    const errors = compareBaseline(expected, baseline);
    if (errors.length > 0) {
      for (const error of errors) console.error(error);
      process.exit(1);
    }
    console.log(`OK ${path.relative(ROOT, OUTPUT)}`);
  }
}
