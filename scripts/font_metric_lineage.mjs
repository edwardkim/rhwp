#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { execFileSync } from 'node:child_process';
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
const MANIFEST_OUTPUT = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4964',
  'font_metric_lineage_manifest.json',
);
const MANIFEST_SCHEMA = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4964',
  'font_metric_lineage_manifest.schema.json',
);
const BASELINE_GENERATOR_VERSION = '1.0.0';
const MANIFEST_GENERATOR_VERSION = '1.0.0';
const SOURCE_COMMIT = 'd1ad0eb8784dbc55f0796e2ba8775f7363247b91';
const OVERLAY_NAMES = [
  'HanyangSinMyeongJo',
  'HanyangJungGothic',
  'HanyangKyunMyeongJo',
  'HanyangKyunGothic',
  'HumanMyeongJo',
];
const OVERLAY_EVIDENCE = {
  HanyangSinMyeongJo: {
    displayName: '한양신명조',
    ladder: 'tools/task2430/measured/ladder_한양신명조.tsv',
    oracle: 'mydocs/tech/investigations/issue-4963/profiles/historical_hanyang_sinmyeongjo_exact_installed.json',
  },
  HanyangJungGothic: {
    displayName: '한양중고딕',
    ladder: 'tools/task2430/measured/ladder_한양중고딕.tsv',
    oracle: null,
  },
  HanyangKyunMyeongJo: {
    displayName: '한양견명조',
    ladder: 'tools/task2430/measured/ladder_한양견명조.tsv',
    oracle: null,
  },
  HanyangKyunGothic: {
    displayName: '한양견고딕',
    ladder: 'tools/task2430/measured/ladder_한양견고딕.tsv',
    oracle: null,
  },
  HumanMyeongJo: {
    displayName: '휴먼명조',
    ladder: 'tools/task2430/measured/ladder_휴먼명조.tsv',
    oracle: 'mydocs/tech/investigations/issue-4963/profiles/historical_human_myeongjo_exact_installed.json',
  },
};

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
  const lineStarts = [0];
  for (let index = 0; index < source.length; index += 1) {
    if (source[index] === '\n') lineStarts.push(index + 1);
  }
  const sourceLineAt = offset => {
    let low = 0;
    let high = lineStarts.length;
    while (low < high) {
      const middle = Math.floor((low + high) / 2);
      if (lineStarts[middle] <= offset) low = middle + 1;
      else high = middle;
    }
    return low;
  };
  const declarations = source.includes('static FONT_METRICS: [FontMetric;')
    ? [{ symbol: 'FONT_METRICS', expectedCount: 600 }]
    : [
      { symbol: 'GENERATED_FONT_METRICS', expectedCount: 595 },
      { symbol: 'MEASURED_FONT_METRIC_OVERLAYS', expectedCount: 5 },
    ];
  const pattern = /FontMetric\s*\{\s*name:\s*"((?:\\.|[^"\\])*)",\s*bold:\s*(true|false),\s*italic:\s*(true|false),\s*em_size:\s*(\d+),\s*latin_ranges:\s*&([A-Z0-9_]+),\s*hangul:\s*(?:Some\(&([A-Z0-9_]+)\)|None),\s*\}/g;
  const entries = [];
  for (const declaration of declarations) {
    const declarationAt = source.indexOf(`static ${declaration.symbol}:`);
    if (declarationAt === -1) throw new Error(`${declaration.symbol} declaration not found`);
    const assignmentAt = source.indexOf('=', declarationAt);
    const declared = Number.parseInt(
      source.slice(declarationAt, assignmentAt).match(/\[FontMetric;\s*(\d+)\]/)?.[1] ?? '',
      10,
    );
    const balanced = extractBalanced(source, assignmentAt, '[', ']');
    const baseIndex = entries.length;
    const parsed = [...balanced.body.matchAll(pattern)].map((match, localIndex) => {
      const index = baseIndex + localIndex;
      if (!latinRanges.has(match[5])) throw new Error(`metric ${index} references missing ${match[5]}`);
      if (match[6] !== undefined && !hangul.has(match[6])) {
        throw new Error(`metric ${index} references missing ${match[6]}`);
      }
      return {
        index,
        sourceLine: sourceLineAt(balanced.openAt + 1 + match.index),
        name: decodeQuotedString(match[1]),
        bold: match[2] === 'true',
        italic: match[3] === 'true',
        emSize: Number.parseInt(match[4], 10),
        latinRangesSymbol: match[5],
        hangulSymbol: match[6] ?? null,
      };
    });
    if (!Number.isInteger(declared) || parsed.length !== declared || declared !== declaration.expectedCount) {
      throw new Error(`${declaration.symbol} parsed ${parsed.length}/${declared}, expected ${declaration.expectedCount}`);
    }
    entries.push(...parsed);
  }
  return entries;
}

export function loadMetricRepositorySource(root = ROOT) {
  const corePath = path.join(root, 'src', 'renderer', 'font_metrics_data.rs');
  const core = fs.readFileSync(corePath, 'utf8');
  if (core.includes('static FONT_METRICS: [FontMetric;')) return core;
  const generatedPath = path.join(root, 'src', 'renderer', 'font_metrics_generated.rs');
  const overlaysPath = path.join(root, 'src', 'renderer', 'font_metrics_overlays.rs');
  if (!fs.existsSync(generatedPath) || !fs.existsSync(overlaysPath)) {
    throw new Error('split font metric source is incomplete');
  }
  const registryPath = path.join(root, 'assets', 'font-rules', 'font_rule_registry_v2.json');
  const registry = JSON.parse(fs.readFileSync(registryPath, 'utf8'));
  const aliases = registry.rules.filter(rule => (
    rule.status === 'active'
      && rule.projections.some(projection => projection.id === 'rust-layout-metric')
  )).sort((left, right) => left.projectionSequence - right.projectionSequence);
  const expectedAliasCount = registry.summary.countsByProjection['rust-layout-metric'];
  if (aliases.length !== expectedAliasCount) {
    throw new Error(
      `canonical layout-metric projection has ${aliases.length}/${expectedAliasCount} aliases`,
    );
  }
  const aliasProjection = [
    'fn resolve_metric_alias(name: &str) -> &str {',
    '    match name {',
    ...aliases.map(rule => (
      `        ${JSON.stringify(rule.sourceFace)} => ${JSON.stringify(rule.targetFaceOrPolicy)},`
    )),
    '        _ => name,',
    '    }',
    '}',
  ].join('\n');
  return `${core}\n${fs.readFileSync(generatedPath, 'utf8')}\n${fs.readFileSync(overlaysPath, 'utf8')}\n${aliasProjection}`;
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
  const entryHashes = [];
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
    entryHashes.push({
      index: metric.index,
      sha256: crypto.createHash('sha256').update(buffer).digest('hex'),
    });
  }
  return {
    dataBearingCodepointCount,
    entryHashes,
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
  const metricData = metricDataProjection(model);
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
    entryMetricDataHashes: metricData.map(entry => ({
      index: entry.index,
      sha256: sha256Text(canonicalJson(entry)),
    })),
    entryLatinRangeCounts: entries.map(entry => ({
      index: entry.index,
      count: latinRanges.get(entry.latinRangesSymbol).length,
    })),
    entrySourceLines: entries.map(entry => ({ index: entry.index, sourceLine: entry.sourceLine })),
    entryWidthHashes: widths.entryHashes,
    metricDataSha256: sha256Text(canonicalJson(metricData)),
    styleCounts,
    uniqueNameCount: new Set(entries.map(entry => entry.name)).size,
    widthProjection: widths,
  };
}

export function analyzeMetricRepository(root = ROOT) {
  return analyzeMetricSource(loadMetricRepositorySource(root));
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
  const source = loadMetricRepositorySource(root);
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
    generatorVersion: BASELINE_GENERATOR_VERSION,
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

export function verifyApprovedBaseline(root = ROOT) {
  const expected = JSON.parse(fs.readFileSync(
    path.join(root, 'mydocs', 'tech', 'investigations', 'issue-4964', 'font_metric_pre_split_baseline.json'),
    'utf8',
  ));
  const analysis = analyzeMetricRepository(root);
  const errors = [];
  if (analysis.entryCount !== expected.fontMetrics.entryCount) errors.push('entry count differs from W6-1');
  if (analysis.uniqueNameCount !== expected.fontMetrics.uniqueNameCount) errors.push('unique name count differs from W6-1');
  if (canonicalJson(analysis.styleCounts) !== canonicalJson(expected.fontMetrics.styleCounts)) {
    errors.push('style counts differ from W6-1');
  }
  const hashes = {
    compositionSha256: analysis.compositionSha256,
    lookupProjectionSha256: analysis.lookupProjectionSha256,
    metricDataSha256: analysis.metricDataSha256,
    widthProjectionSha256: analysis.widthProjection.sha256,
  };
  if (canonicalJson(hashes) !== canonicalJson(expected.hashes)) errors.push('semantic hashes differ from W6-1');
  assertMeasuredOverlayRegion(analysis.composition);
  return errors;
}

function statusValue(status, reason, value, evidenceIds = []) {
  const result = { evidenceIds, reason, status };
  if (value !== undefined) result.value = value;
  return result;
}

function unknownValue(reason) {
  return statusValue('unknown', reason, undefined, []);
}

function notApplicableValue(reason) {
  return statusValue('not-applicable', reason, undefined, []);
}

function verifiedValue(value, reason, evidenceIds) {
  return statusValue('verified', reason, value, evidenceIds);
}

function stableEntryId(entry) {
  const identity = `${entry.name}\0${entry.bold ? 1 : 0}\0${entry.italic ? 1 : 0}`;
  return `font-metric.${sha256Text(identity).slice(0, 20)}`;
}

function parseSfntNamingRecords(file) {
  const data = fs.readFileSync(file);
  if (data.subarray(0, 4).toString('ascii') === 'ttcf') {
    throw new Error(`${path.relative(ROOT, file)} is a TTC; Stage W6-2 canary expects a single-face TTF`);
  }
  const numTables = data.readUInt16BE(4);
  let nameOffset = null;
  let nameLength = null;
  for (let index = 0; index < numTables; index += 1) {
    const record = 12 + index * 16;
    if (data.subarray(record, record + 4).toString('ascii') !== 'name') continue;
    nameOffset = data.readUInt32BE(record + 8);
    nameLength = data.readUInt32BE(record + 12);
    break;
  }
  if (nameOffset === null || nameOffset + nameLength > data.length) {
    throw new Error(`${path.relative(ROOT, file)} has no valid name table`);
  }
  const count = data.readUInt16BE(nameOffset + 2);
  const stringsAt = nameOffset + data.readUInt16BE(nameOffset + 4);
  const records = [];
  for (let index = 0; index < count; index += 1) {
    const record = nameOffset + 6 + index * 12;
    const platformId = data.readUInt16BE(record);
    const encodingId = data.readUInt16BE(record + 2);
    const languageId = data.readUInt16BE(record + 4);
    const nameId = data.readUInt16BE(record + 6);
    const length = data.readUInt16BE(record + 8);
    const valueAt = stringsAt + data.readUInt16BE(record + 10);
    if (![0, 3].includes(platformId) || valueAt + length > data.length || length % 2 !== 0) continue;
    const encoded = data.subarray(valueAt, valueAt + length);
    const littleEndian = Buffer.allocUnsafe(encoded.length);
    for (let byte = 0; byte < encoded.length; byte += 2) {
      littleEndian[byte] = encoded[byte + 1];
      littleEndian[byte + 1] = encoded[byte];
    }
    records.push({
      platformId,
      encodingId,
      languageId,
      nameId,
      value: littleEndian.toString('utf16le'),
    });
  }
  records.sort((left, right) => (
    left.platformId - right.platformId
    || left.encodingId - right.encodingId
    || left.languageId - right.languageId
    || left.nameId - right.nameId
    || left.value.localeCompare(right.value, 'en')
  ));
  if (records.length === 0) throw new Error(`${path.relative(ROOT, file)} has no decodable naming records`);
  return records;
}

function historicalMetricSource(root) {
  return execFileSync(
    'git',
    ['show', `${SOURCE_COMMIT}:src/renderer/font_metrics_data.rs`],
    { cwd: root, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 },
  );
}

function blameCommitsByLine(root) {
  const output = execFileSync(
    'git',
    ['blame', '--line-porcelain', SOURCE_COMMIT, '--', 'src/renderer/font_metrics_data.rs'],
    { cwd: root, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 },
  );
  const commits = new Map();
  for (const match of output.matchAll(/^([0-9a-f]{40}) \d+ (\d+)(?: \d+)?$/gm)) {
    commits.set(Number.parseInt(match[2], 10), match[1]);
  }
  return commits;
}

function evidenceRecord(root, id, kind, relativePath) {
  const file = path.join(root, relativePath);
  if (!fs.existsSync(file)) throw new Error(`evidence path does not exist: ${relativePath}`);
  return { id, kind, path: relativePath, sha256: sha256File(file) };
}

function buildEvidenceCatalog(root) {
  const records = [
    evidenceRecord(root, 'w6-pre-split-baseline', 'baseline', 'mydocs/tech/investigations/issue-4964/font_metric_pre_split_baseline.json'),
    evidenceRecord(root, 'w1-font-rule-ledger', 'ledger', 'mydocs/tech/investigations/issue-4939/font_rule_ledger.json'),
    evidenceRecord(root, 'causal-lineage-report', 'report', 'mydocs/report/font_metrics_fallback_causal_lineage_20260816.md'),
    evidenceRecord(root, 'task2430-evidence', 'measurement-report', 'tools/task2430/EVIDENCE.md'),
    evidenceRecord(root, 'task2430-generator', 'measurement-tool', 'tools/task2430/gen_metrics.py'),
    evidenceRecord(root, 'task2430-preflight', 'measurement-preflight', 'tools/task2430/measured/preflight_report.tsv'),
    evidenceRecord(root, 'noto-sans-kr-regular-font', 'font', 'ttfs/opensource/NotoSansKR-Regular.ttf'),
    evidenceRecord(root, 'noto-sans-kr-ofl', 'license', 'ttfs/opensource/NotoSansKR-OFL.txt'),
    evidenceRecord(root, 'noto-sans-kr-readme', 'provenance', 'ttfs/opensource/README.md'),
  ];
  for (const [name, evidence] of Object.entries(OVERLAY_EVIDENCE)) {
    records.push(evidenceRecord(root, `task2430-ladder-${stableEntryId({ name, bold: false, italic: false }).split('.')[1]}`, 'measurement-data', evidence.ladder));
    if (evidence.oracle !== null) {
      records.push(evidenceRecord(root, `w5-oracle-${stableEntryId({ name, bold: false, italic: false }).split('.')[1]}`, 'oracle-profile', evidence.oracle));
    }
  }
  records.sort((left, right) => left.id.localeCompare(right.id, 'en'));
  return records;
}

function evidenceIdByPath(catalog) {
  return new Map(catalog.map(record => [record.path, record.id]));
}

function metricRulesByIndex(root) {
  const ledger = JSON.parse(fs.readFileSync(
    path.join(root, 'mydocs', 'tech', 'investigations', 'issue-4939', 'font_rule_ledger.json'),
    'utf8',
  ));
  const rules = new Map();
  for (const rule of ledger.rules) {
    if (rule.relationType !== 'metric-entry') continue;
    const match = rule.targetFaceOrPolicy.match(/^metric-entry:(\d+)$/);
    if (match === null) throw new Error(`W1 metric rule has invalid target: ${rule.ruleId}`);
    const index = Number.parseInt(match[1], 10);
    if (rules.has(index)) throw new Error(`duplicate W1 metric rule for index ${index}`);
    rules.set(index, rule);
  }
  if (rules.size !== 600) throw new Error(`W1 metric rule population is ${rules.size}/600`);
  return rules;
}

function unknownFontSource() {
  const reason = 'The repository does not preserve an item-level source font manifest for this legacy metric entry.';
  return {
    status: 'unknown',
    reason,
    verificationScope: 'none',
    sha256: unknownValue(reason),
    faceIndex: unknownValue(reason),
    namingRecords: unknownValue(reason),
    license: unknownValue(reason),
    provenance: unknownValue(reason),
  };
}

function notoRegularFontSource(root) {
  const relativePath = 'ttfs/opensource/NotoSansKR-Regular.ttf';
  const evidenceIds = ['noto-sans-kr-regular-font', 'noto-sans-kr-ofl', 'noto-sans-kr-readme'];
  const sourceReason = 'The tracked subset is byte-identified, but #4442 verifies only printable ASCII advances; this does not prove full metric source-exactness.';
  return {
    status: 'verified',
    reason: sourceReason,
    verificationScope: 'printable-ascii-only',
    sha256: verifiedValue(sha256File(path.join(root, relativePath)), 'Digest of the tracked TTF subset.', ['noto-sans-kr-regular-font']),
    faceIndex: verifiedValue(0, 'The tracked artifact is a single-face TTF.', ['noto-sans-kr-regular-font']),
    namingRecords: verifiedValue(parseSfntNamingRecords(path.join(root, relativePath)), 'Decoded Unicode-platform SFNT name records from the tracked TTF.', ['noto-sans-kr-regular-font']),
    license: verifiedValue({ id: 'SIL-OFL-1.1', path: 'ttfs/opensource/NotoSansKR-OFL.txt' }, 'The tracked source documents SIL OFL 1.1.', ['noto-sans-kr-ofl', 'noto-sans-kr-readme']),
    provenance: verifiedValue(['Google Fonts Noto Sans KR variable source', 'wght=400 instance', 'rhwp project subset'], 'The tracked README records the subset lineage.', evidenceIds),
  };
}

function measurementSourceFor(entry, catalogByPath) {
  const evidence = OVERLAY_EVIDENCE[entry.name];
  if (evidence === undefined) {
    const reason = 'This legacy region has no item-level record proving whether later manual or measured edits occurred.';
    return {
      status: 'unknown',
      reason,
      method: unknownValue(reason),
      inputDigests: unknownValue(reason),
      interpolation: unknownValue(reason),
      evidenceIds: [],
    };
  }
  const evidenceIds = [
    'task2430-evidence',
    'task2430-generator',
    'task2430-preflight',
    catalogByPath.get(evidence.ladder),
  ];
  return {
    status: 'verified',
    reason: 'The committed ASCII overlay reproduces the tracked Hancom COM ladder exactly.',
    method: verifiedValue('Hancom COM unscaled printable-ASCII ladder', '93 characters were measured directly.', evidenceIds),
    inputDigests: verifiedValue(evidenceIds.map(id => ({ evidenceId: id })), 'Every retained measurement input is addressed through the evidence catalog.', evidenceIds),
    interpolation: verifiedValue({ measuredCodepoints: 93, interpolatedCodepoints: 2, method: 'median-ratio-against-corresponding-HY-metric' }, 'Double and single quotes were excluded by Hancom autocorrect and deterministically interpolated.', evidenceIds),
    evidenceIds,
  };
}

function oracleLinksFor(entry, catalogByPath, root) {
  const oraclePath = OVERLAY_EVIDENCE[entry.name]?.oracle;
  if (oraclePath === undefined || oraclePath === null) return [];
  const profile = JSON.parse(fs.readFileSync(path.join(root, oraclePath), 'utf8'));
  const expectedFace = OVERLAY_EVIDENCE[entry.name].displayName;
  if (profile.candidate?.documentFace !== expectedFace) {
    throw new Error(`${oraclePath} documentFace does not match ${entry.name}`);
  }
  return [{
    evidenceId: catalogByPath.get(oraclePath),
    relationType: profile.relationEvidence?.type ?? 'unknown',
    scope: 'face-identity-not-metric-source-exactness',
  }];
}

export function buildLineageManifest(root = ROOT) {
  const source = loadMetricRepositorySource(root);
  const analysis = analyzeMetricSource(source);
  const historicalAnalysis = analyzeMetricSource(historicalMetricSource(root));
  assertMeasuredOverlayRegion(analysis.composition);
  const baseline = JSON.parse(fs.readFileSync(
    path.join(root, 'mydocs', 'tech', 'investigations', 'issue-4964', 'font_metric_pre_split_baseline.json'),
    'utf8',
  ));
  if (analysis.compositionSha256 !== baseline.hashes.compositionSha256
    || analysis.metricDataSha256 !== baseline.hashes.metricDataSha256
    || analysis.widthProjection.sha256 !== baseline.hashes.widthProjectionSha256
    || analysis.lookupProjectionSha256 !== baseline.hashes.lookupProjectionSha256) {
    throw new Error('current metric source no longer matches the approved W6-1 baseline');
  }

  const evidenceCatalog = buildEvidenceCatalog(root);
  const catalogByPath = evidenceIdByPath(evidenceCatalog);
  const w1Rules = metricRulesByIndex(root);
  const blame = blameCommitsByLine(root);
  const sourceLines = new Map(historicalAnalysis.entrySourceLines.map(row => [row.index, row.sourceLine]));
  const metricHashes = new Map(analysis.entryMetricDataHashes.map(row => [row.index, row.sha256]));
  const widthHashes = new Map(analysis.entryWidthHashes.map(row => [row.index, row.sha256]));
  const notoRegular = analysis.composition.find(
    entry => entry.name === 'Noto Sans KR' && !entry.bold && !entry.italic,
  );
  if (notoRegular === undefined) throw new Error('Noto Sans KR regular metric entry not found');
  const splitSource = fs.existsSync(path.join(root, 'src', 'renderer', 'font_metrics_generated.rs'));

  const entries = analysis.composition.map(entry => {
    const measured = entry.index >= 595;
    const sourceLine = sourceLines.get(entry.index);
    const declarationCommit = blame.get(sourceLine);
    if (declarationCommit === undefined) throw new Error(`git blame commit missing for metric ${entry.index}`);
    const w1Rule = w1Rules.get(entry.index);
    if (w1Rule.sourceFace !== entry.name) {
      throw new Error(`W1 rule ${w1Rule.ruleId} name ${w1Rule.sourceFace} does not match ${entry.name}`);
    }
    const entryId = stableEntryId(entry);
    const measurement = measurementSourceFor(entry, catalogByPath);
    const relationEvidence = measured ? measurement.evidenceIds : ['causal-lineage-report'];
    const relations = [{
      relationId: w1Rule.ruleId,
      relationType: 'metric-entry',
      evidenceIds: ['w1-font-rule-ledger'],
    }];
    if (measured) {
      relations.push({
        relationId: `relation.task2430.${entryId.split('.')[1]}`,
        relationType: 'measured-overlay',
        evidenceIds: relationEvidence,
      });
    }
    return {
      entryId,
      currentIndex: entry.index,
      metricIdentity: {
        name: entry.name,
        bold: entry.bold,
        italic: entry.italic,
        emSize: entry.emSize,
      },
      storageRegion: {
        kind: measured ? 'measured-overlay' : 'historical-generated-region',
        sourcePath: splitSource
          ? measured
            ? 'src/renderer/font_metrics_overlays.rs'
            : 'src/renderer/font_metrics_generated.rs'
          : 'src/renderer/font_metrics_data.rs',
        latinRangesSymbol: entry.latinRangesSymbol,
        hangulSymbol: entry.hangulSymbol,
      },
      origin: {
        kind: measured ? 'measured-overlay' : 'historical-generated-snapshot',
        status: measured ? 'verified' : 'unknown',
        reason: measured
          ? 'The #2430 generator exactly reproduces the committed 95-codepoint ASCII overlay from tracked measurement data.'
          : 'No item-level generator input manifest survives; residence in the generated region does not prove source-exact origin.',
        declarationCommit,
        evidenceIds: measured ? relationEvidence : ['causal-lineage-report', 'w6-pre-split-baseline'],
      },
      fontSource: entry.index === notoRegular.index ? notoRegularFontSource(root) : unknownFontSource(),
      measurementSource: measurement,
      composition: {
        latinRangesSymbol: entry.latinRangesSymbol,
        hangulSymbol: entry.hangulSymbol,
      },
      compression: entry.hangulSymbol === null
        ? {
          status: 'not-applicable',
          algorithm: 'none',
          reason: 'This metric entry has no stored HangulMetric.',
          maxAbsoluteError: notApplicableValue('No Hangul compression exists for this entry.'),
          averageAbsoluteError: notApplicableValue('No Hangul compression exists for this entry.'),
        }
        : {
          status: 'unknown',
          algorithm: 'grouped-hangul-width-grid',
          reason: 'The historical generator did not emit per-entry compression error metadata.',
          maxAbsoluteError: unknownValue('The source font and historical compression run are unavailable.'),
          averageAbsoluteError: unknownValue('The source font and historical compression run are unavailable.'),
        },
      relations,
      oracleProfiles: oracleLinksFor(entry, catalogByPath, root),
      semanticHashes: {
        metricDataSha256: metricHashes.get(entry.index),
        widthProjectionSha256: widthHashes.get(entry.index),
      },
    };
  });

  const entryIds = new Set(entries.map(entry => entry.entryId));
  if (entryIds.size !== entries.length) throw new Error('stable metric entry ID collision');
  return {
    schemaVersion: '1.0',
    kind: 'font-metric-lineage-manifest',
    sourceCommit: SOURCE_COMMIT,
    generatorVersion: MANIFEST_GENERATOR_VERSION,
    schema: {
      path: 'mydocs/tech/investigations/issue-4964/font_metric_lineage_manifest.schema.json',
      sha256: sha256File(root === ROOT ? MANIFEST_SCHEMA : path.join(root, 'mydocs', 'tech', 'investigations', 'issue-4964', 'font_metric_lineage_manifest.schema.json')),
    },
    evidenceCatalog,
    summary: {
      entryCount: entries.length,
      stableEntryIdCount: entryIds.size,
      w1MetricEntryLinks: entries.filter(entry => entry.relations.some(relation => relation.relationType === 'metric-entry')).length,
      measuredOverlayEntries: entries.filter(entry => entry.origin.kind === 'measured-overlay').length,
      unknownOriginEntries: entries.filter(entry => entry.origin.status === 'unknown').length,
      fullySourceExactEntries: 0,
      partiallyByteVerifiedFontSources: entries.filter(entry => entry.fontSource.verificationScope === 'printable-ascii-only').length,
      w5OracleProfileLinks: entries.reduce((total, entry) => total + entry.oracleProfiles.length, 0),
    },
    baselineHashes: baseline.hashes,
    entriesSha256: sha256Text(canonicalJson(entries)),
    entries,
  };
}

function validateStatusValue(value, location, evidenceIds, errors) {
  if (value === null || typeof value !== 'object' || Array.isArray(value)) {
    errors.push(`${location} must be a status object`);
    return;
  }
  if (!['verified', 'historical', 'inferred', 'unknown', 'not-applicable'].includes(value.status)) {
    errors.push(`${location}.status is invalid`);
  }
  if (typeof value.reason !== 'string' || value.reason.length === 0) {
    errors.push(`${location}.reason must be non-empty`);
  }
  if (!Array.isArray(value.evidenceIds)) errors.push(`${location}.evidenceIds must be an array`);
  else for (const id of value.evidenceIds) if (!evidenceIds.has(id)) errors.push(`${location} references missing evidence ${id}`);
  if (value.status === 'verified' && !Object.hasOwn(value, 'value')) {
    errors.push(`${location}.value is required for verified status`);
  }
  if (['unknown', 'not-applicable'].includes(value.status) && Object.hasOwn(value, 'value')) {
    errors.push(`${location}.value must be absent for ${value.status} status`);
  }
}

export function validateLineageManifest(manifest, root = ROOT) {
  const errors = [];
  if (manifest.kind !== 'font-metric-lineage-manifest') errors.push('kind is invalid');
  if (!Array.isArray(manifest.entries) || manifest.entries.length !== 600) {
    errors.push(`entries must contain exactly 600 rows, got ${manifest.entries?.length}`);
    return errors;
  }
  const evidenceIds = new Set();
  for (const evidence of manifest.evidenceCatalog ?? []) {
    if (evidenceIds.has(evidence.id)) errors.push(`duplicate evidence id ${evidence.id}`);
    evidenceIds.add(evidence.id);
    if (path.isAbsolute(evidence.path) || evidence.path.includes('..')) errors.push(`unsafe evidence path ${evidence.path}`);
    const file = path.join(root, evidence.path);
    if (!fs.existsSync(file)) errors.push(`missing evidence path ${evidence.path}`);
    else if (sha256File(file) !== evidence.sha256) errors.push(`evidence digest drift ${evidence.path}`);
  }
  const schemaPath = typeof manifest.schema?.path === 'string'
    ? path.join(root, manifest.schema.path)
    : null;
  if (schemaPath === null || !fs.existsSync(schemaPath) || !fs.statSync(schemaPath).isFile()
    || sha256File(schemaPath) !== manifest.schema?.sha256) {
    errors.push('schema path or digest does not match the tracked schema');
  }
  const analysis = analyzeMetricRepository(root);
  const expectedMetricHashes = new Map(analysis.entryMetricDataHashes.map(row => [row.index, row.sha256]));
  const expectedWidthHashes = new Map(analysis.entryWidthHashes.map(row => [row.index, row.sha256]));
  const expectedRules = metricRulesByIndex(root);
  const ids = new Set();
  for (const [index, entry] of manifest.entries.entries()) {
    const location = `entries[${index}]`;
    if (entry.currentIndex !== index) errors.push(`${location}.currentIndex must equal ${index}`);
    if (!/^font-metric\.[0-9a-f]{20}$/.test(entry.entryId)) errors.push(`${location}.entryId is invalid`);
    if (ids.has(entry.entryId)) errors.push(`${location}.entryId is duplicated`);
    ids.add(entry.entryId);
    if (entry.entryId !== stableEntryId(entry.metricIdentity)) errors.push(`${location}.entryId does not match metric identity`);
    const expectedIdentity = analysis.composition[index];
    if (canonicalJson(entry.metricIdentity) !== canonicalJson({
      name: expectedIdentity.name,
      bold: expectedIdentity.bold,
      italic: expectedIdentity.italic,
      emSize: expectedIdentity.emSize,
    })) errors.push(`${location}.metricIdentity differs from Rust source`);
    if (entry.storageRegion.latinRangesSymbol !== expectedIdentity.latinRangesSymbol
      || entry.storageRegion.hangulSymbol !== expectedIdentity.hangulSymbol) {
      errors.push(`${location}.storageRegion differs from Rust source`);
    }
    if (entry.origin.status === 'unknown' && (!entry.origin.reason || entry.origin.reason.length === 0)) {
      errors.push(`${location}.origin unknown requires a reason`);
    }
    if (entry.origin.kind === 'historical-generated-snapshot' && entry.origin.status === 'verified') {
      errors.push(`${location} promotes a legacy generated entry to verified without an item-level source manifest`);
    }
    for (const id of entry.origin.evidenceIds) if (!evidenceIds.has(id)) errors.push(`${location}.origin references missing evidence ${id}`);
    for (const field of ['sha256', 'faceIndex', 'namingRecords', 'license', 'provenance']) {
      validateStatusValue(entry.fontSource[field], `${location}.fontSource.${field}`, evidenceIds, errors);
    }
    for (const field of ['method', 'inputDigests', 'interpolation']) {
      validateStatusValue(entry.measurementSource[field], `${location}.measurementSource.${field}`, evidenceIds, errors);
    }
    for (const field of ['maxAbsoluteError', 'averageAbsoluteError']) {
      validateStatusValue(entry.compression[field], `${location}.compression.${field}`, evidenceIds, errors);
    }
    for (const relation of entry.relations) {
      if (!['metric-entry', 'measured-overlay'].includes(relation.relationType)) errors.push(`${location} has invalid relation type`);
      for (const id of relation.evidenceIds) if (!evidenceIds.has(id)) errors.push(`${location}.relations references missing evidence ${id}`);
    }
    const metricRelations = entry.relations.filter(relation => relation.relationType === 'metric-entry');
    if (metricRelations.length !== 1 || metricRelations[0].relationId !== expectedRules.get(index).ruleId) {
      errors.push(`${location} must link its exact W1 metric-entry rule`);
    }
    for (const oracle of entry.oracleProfiles) {
      if (!evidenceIds.has(oracle.evidenceId)) errors.push(`${location}.oracleProfiles references missing evidence ${oracle.evidenceId}`);
      if (oracle.scope !== 'face-identity-not-metric-source-exactness') errors.push(`${location}.oracleProfiles has unsafe scope`);
    }
    if (entry.semanticHashes.metricDataSha256 !== expectedMetricHashes.get(index)) {
      errors.push(`${location}.semanticHashes.metricDataSha256 differs from Rust source`);
    }
    if (entry.semanticHashes.widthProjectionSha256 !== expectedWidthHashes.get(index)) {
      errors.push(`${location}.semanticHashes.widthProjectionSha256 differs from Rust source`);
    }
  }
  if (manifest.entriesSha256 !== sha256Text(canonicalJson(manifest.entries))) errors.push('entriesSha256 does not match entries');
  if (manifest.summary?.entryCount !== 600 || manifest.summary?.stableEntryIdCount !== 600) errors.push('summary population is not closed');
  if (manifest.summary?.unknownOriginEntries !== 595 || manifest.summary?.measuredOverlayEntries !== 5) errors.push('origin summary must remain 595 unknown + 5 measured');
  if (manifest.summary?.fullySourceExactEntries !== 0) errors.push('Stage W6-2 must not claim fully source-exact entries');
  if (manifest.summary?.w1MetricEntryLinks !== 600) errors.push('every entry must link one W1 metric rule');
  if (manifest.summary?.w5OracleProfileLinks !== 2) errors.push('only the two retained W5 historical profiles may be linked');
  const baseline = JSON.parse(fs.readFileSync(
    path.join(root, 'mydocs', 'tech', 'investigations', 'issue-4964', 'font_metric_pre_split_baseline.json'),
    'utf8',
  ));
  if (canonicalJson(manifest.baselineHashes) !== canonicalJson(baseline.hashes)) {
    errors.push('baselineHashes differ from the approved W6-1 baseline');
  }
  return errors;
}

export function compareManifest(expected, actual) {
  return canonicalJson(expected) === canonicalJson(actual)
    ? []
    : ['font metric lineage manifest differs; run --generate-manifest only in an approved lineage stage'];
}

function usage() {
  console.error('usage: node scripts/font_metric_lineage.mjs --generate|--check|--generate-manifest|--check-manifest');
}

if (process.argv[1] !== undefined && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const mode = process.argv[2];
  if (!['--generate', '--check', '--generate-manifest', '--check-manifest'].includes(mode) || process.argv.length !== 3) {
    usage();
    process.exit(2);
  }
  if (mode === '--generate') {
    const baseline = buildPreSplitBaseline();
    fs.mkdirSync(path.dirname(OUTPUT), { recursive: true });
    fs.writeFileSync(OUTPUT, canonicalJson(baseline), 'utf8');
    console.log(`generated ${path.relative(ROOT, OUTPUT)}`);
  } else if (mode === '--check') {
    if (!fs.existsSync(OUTPUT)) throw new Error(`baseline does not exist: ${path.relative(ROOT, OUTPUT)}`);
    const errors = verifyApprovedBaseline();
    if (errors.length > 0) {
      for (const error of errors) console.error(error);
      process.exit(1);
    }
    console.log(`OK ${path.relative(ROOT, OUTPUT)}`);
  } else {
    const manifest = buildLineageManifest();
    const validationErrors = validateLineageManifest(manifest);
    if (validationErrors.length > 0) throw new Error(validationErrors.join('\n'));
    if (mode === '--generate-manifest') {
      fs.writeFileSync(MANIFEST_OUTPUT, canonicalJson(manifest), 'utf8');
      console.log(`generated ${path.relative(ROOT, MANIFEST_OUTPUT)}`);
    } else {
      if (!fs.existsSync(MANIFEST_OUTPUT)) throw new Error(`manifest does not exist: ${path.relative(ROOT, MANIFEST_OUTPUT)}`);
      const expected = JSON.parse(fs.readFileSync(MANIFEST_OUTPUT, 'utf8'));
      const errors = [...validateLineageManifest(expected), ...compareManifest(expected, manifest)];
      if (errors.length > 0) {
        for (const error of errors) console.error(error);
        process.exit(1);
      }
      console.log(`OK ${path.relative(ROOT, MANIFEST_OUTPUT)}`);
    }
  }
}
