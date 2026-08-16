#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { canonicalJson, sha256Text } from './font_rule_ledger.mjs';

const REPOSITORY_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const GENERATOR_VERSION = '3.0.0';

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function sha256File(file) {
  return crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function countBy(values) {
  const counts = {};
  for (const value of values) counts[value] = (counts[value] ?? 0) + 1;
  return Object.fromEntries(Object.entries(counts).sort(([left], [right]) => compareText(left, right)));
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
    if (character === '"' || character === "'" || character === '`') {
      quote = character;
      continue;
    }
    if (character === openCharacter) depth += 1;
    if (character === closeCharacter) {
      depth -= 1;
      if (depth === 0) return source.slice(openAt + 1, index);
    }
  }
  throw new Error(`closing ${closeCharacter} not found`);
}

function functionBody(source, selector) {
  const functionAt = source.indexOf(selector);
  if (functionAt === -1) throw new Error(`function selector not found: ${selector}`);
  return extractBalanced(source, functionAt, '{', '}');
}

function decodeDoubleQuoted(value) {
  return JSON.parse(`"${value}"`);
}

function candidate(boundary, candidateKind, decisionPlane, sourceFace, targetOrPolicy, options = {}) {
  const conditions = options.conditions ?? {};
  const order = options.order ?? null;
  const identity = {
    sourceBoundaryId: `${boundary.ownerId}.${boundary.selectorId}`,
    candidateKind,
    sourceFace,
    targetOrPolicy,
    conditions,
    order,
  };
  return {
    candidateId: `candidate.${sha256Text(canonicalJson(identity)).slice(0, 20)}`,
    sourceBoundaryId: identity.sourceBoundaryId,
    ownerId: boundary.ownerId,
    candidateKind,
    decisionPlane,
    sourceFace,
    targetOrPolicy,
    conditions,
    backends: options.backends ?? ['shared'],
    order,
    sourceLocation: {
      path: boundary.path,
      symbol: boundary.symbol,
      selector: boundary.selector,
      sourceSha256: boundary.sourceSha256,
    },
  };
}

function extracted(strategy, rows, recognizedMappingBlocks = 1) {
  if (rows.length === 0) throw new Error(`${strategy} extracted zero candidates`);
  return { strategy, rows, recognizedMappingBlocks, unrecognizedMappingBlocks: 0 };
}

function predicate(boundary, decisionPlane, targetOrPolicy, backends = ['shared']) {
  return extracted(
    'whole-symbol-predicate',
    [candidate(boundary, 'predicate', decisionPlane, null, targetOrPolicy, { backends })],
  );
}

function orderedPolicies(boundary, decisionPlane, policies, backends = ['shared']) {
  return extracted(
    'ordered-policy-chain',
    policies.map((policy, order) => candidate(
      boundary,
      'ordered-chain',
      decisionPlane,
      null,
      policy,
      { order, backends },
    )),
    policies.length,
  );
}

function parseRustSomeMappings(boundary, source, languageMode) {
  const body = functionBody(source, boundary.selector);
  const englishAt = body.indexOf('if lang_index == 1');
  const pattern = /((?:"(?:\\.|[^"\\])*"\s*(?:\|\s*)?)+)=>\s*Some\("((?:\\.|[^"\\])*)"\)/gs;
  const matches = [...body.matchAll(pattern)];
  const recognizedMappingBlocks = [...body.matchAll(/=>\s*Some\("/g)].length;
  if (matches.length !== recognizedMappingBlocks) {
    throw new Error(`${boundary.candidateId}: parsed ${matches.length}/${recognizedMappingBlocks} Some mapping arms`);
  }
  const rows = [];
  for (const match of matches) {
    const target = decodeDoubleQuoted(match[2]);
    const languageSlot = languageMode === 'english-only'
      ? '1'
      : languageMode === 'hft'
        ? (englishAt !== -1 && match.index > englishAt ? '1' : 'all')
        : 'all';
    for (const sourceMatch of match[1].matchAll(/"((?:\\.|[^"\\])*)"/g)) {
      rows.push(candidate(
        boundary,
        'finite-mapping',
        'layout-name',
        decodeDoubleQuoted(sourceMatch[1]),
        target,
        { conditions: { languageSlot } },
      ));
    }
  }
  return extracted('rust-some-match-arms', rows, recognizedMappingBlocks);
}

function parseMetricAliases(boundary, source) {
  const body = functionBody(source, boundary.selector);
  const pattern = /((?:"(?:\\.|[^"\\])*"\s*(?:\|\s*)?)+)=>\s*"((?:\\.|[^"\\])*)"\s*,/gs;
  const matches = [...body.matchAll(pattern)];
  const recognizedMappingBlocks = [...body.matchAll(/=>\s*"/g)].length;
  if (matches.length !== recognizedMappingBlocks) {
    throw new Error(`${boundary.candidateId}: parsed ${matches.length}/${recognizedMappingBlocks} alias arms`);
  }
  const rows = [];
  for (const match of matches) {
    const target = decodeDoubleQuoted(match[2]);
    for (const sourceMatch of match[1].matchAll(/"((?:\\.|[^"\\])*)"/g)) {
      rows.push(candidate(
        boundary,
        'finite-mapping',
        'layout-metric',
        decodeDoubleQuoted(sourceMatch[1]),
        target,
      ));
    }
  }
  return extracted('rust-string-match-arms', rows, recognizedMappingBlocks);
}

function parseMetricTable(boundary, source) {
  const declarationAt = source.indexOf('static FONT_METRICS:');
  const assignmentAt = source.indexOf('=', declarationAt);
  const body = extractBalanced(source, assignmentAt, '[', ']');
  const pattern = /FontMetric\s*\{\s*name:\s*"((?:\\.|[^"\\])*)",\s*bold:\s*(true|false),\s*italic:\s*(true|false),\s*em_size:\s*(\d+),\s*latin_ranges:\s*&([A-Z0-9_]+),\s*hangul:\s*(?:Some\(&([A-Z0-9_]+)\)|None),\s*\}/g;
  const rows = [...body.matchAll(pattern)].map((match, index) => candidate(
    boundary,
    'metric-entry',
    'layout-metric',
    decodeDoubleQuoted(match[1]),
    `metric-entry:${index}`,
    {
      conditions: {
        bold: match[2] === 'true',
        italic: match[3] === 'true',
        emSize: Number.parseInt(match[4], 10),
        latinRangeSymbol: match[5],
        hangulSymbol: match[6] ?? null,
      },
    },
  ));
  const declared = Number.parseInt(
    source.slice(declarationAt, assignmentAt).match(/\[FontMetric;\s*(\d+)\]/)?.[1] ?? '',
    10,
  );
  if (rows.length !== declared) throw new Error(`FONT_METRICS parsed ${rows.length}/${declared}`);
  return extracted('rust-metric-table', rows, declared);
}

function parseInstalledAliases(boundary, source) {
  const body = functionBody(source, boundary.selector);
  const pattern = /((?:"(?:\\.|[^"\\])*"\s*(?:\|\s*)?)+)=>\s*&\[([^\]]+)\]/gs;
  const matches = [...body.matchAll(pattern)];
  const rows = [];
  for (const match of matches) {
    const sources = [...match[1].matchAll(/"((?:\\.|[^"\\])*)"/g)]
      .map(value => decodeDoubleQuoted(value[1]));
    const targets = [...match[2].matchAll(/"((?:\\.|[^"\\])*)"/g)]
      .map(value => decodeDoubleQuoted(value[1]));
    for (const sourceFace of sources) {
      targets.forEach((target, order) => rows.push(candidate(
        boundary,
        'ordered-chain',
        'paint',
        sourceFace,
        target,
        { order, conditions: { availability: 'exact-source-unavailable' } },
      )));
    }
  }
  return extracted('rust-array-match-arms', rows, matches.length);
}

function parseStudioSubstitutions(boundary, source) {
  const start = source.indexOf(boundary.selector);
  const assignment = source.indexOf('=', start);
  const body = extractBalanced(source, assignment, '[', ']');
  const rows = [];
  let languageSlot = null;
  let tupleCount = 0;
  for (const line of body.split('\n')) {
    const language = line.match(/Lang\s+(\d+)/);
    if (language) languageSlot = language[1];
    for (const match of line.matchAll(/\['((?:\\.|[^'])*)',\s*(\d+),\s*'((?:\\.|[^'])*)',\s*(\d+)\]/g)) {
      tupleCount += 1;
      rows.push(candidate(
        boundary,
        'finite-mapping',
        'paint',
        match[1],
        match[3],
        {
          conditions: {
            languageSlot,
            sourceAltType: Number.parseInt(match[2], 10),
            targetAltType: Number.parseInt(match[4], 10),
          },
          backends: ['studio', 'canvas2d'],
        },
      ));
    }
  }
  const rawTupleCount = [...body.matchAll(/\['(?:\\.|[^'])*',\s*\d+,\s*'(?:\\.|[^'])*',\s*\d+\]/g)].length;
  if (tupleCount !== rawTupleCount || languageSlot === null) {
    throw new Error(`SUBST_TABLES parsed ${tupleCount}/${rawTupleCount}; language=${languageSlot}`);
  }
  return extracted('typescript-substitution-tuples', rows, tupleCount);
}

function topLevelObjects(arrayBody) {
  const objects = [];
  let quote = null;
  let escaped = false;
  let lineComment = false;
  let blockComment = false;
  let depth = 0;
  let objectStart = -1;
  for (let index = 0; index < arrayBody.length; index += 1) {
    const character = arrayBody[index];
    const next = arrayBody[index + 1];
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
    if (character === '"' || character === "'" || character === '`') {
      quote = character;
      continue;
    }
    if (character === '{') {
      if (depth === 0) objectStart = index;
      depth += 1;
    } else if (character === '}') {
      depth -= 1;
      if (depth === 0 && objectStart !== -1) {
        objects.push(arrayBody.slice(objectStart, index + 1));
        objectStart = -1;
      }
    }
  }
  return objects;
}

function propertyExpression(objectSource, property) {
  const marker = `${property}:`;
  const start = objectSource.indexOf(marker);
  if (start === -1) return null;
  let quote = null;
  let escaped = false;
  let depth = 0;
  const valueStart = start + marker.length;
  for (let index = valueStart; index < objectSource.length; index += 1) {
    const character = objectSource[index];
    if (quote !== null) {
      if (escaped) escaped = false;
      else if (character === '\\') escaped = true;
      else if (character === quote) quote = null;
      continue;
    }
    if (character === '"' || character === "'" || character === '`') {
      quote = character;
      continue;
    }
    if ('([{'.includes(character)) depth += 1;
    else if (')]}'.includes(character)) {
      if (depth === 0 && character === '}') return objectSource.slice(valueStart, index).trim();
      depth -= 1;
    } else if (character === ',' && depth === 0) {
      return objectSource.slice(valueStart, index).trim();
    }
  }
  return null;
}

function unquoteSingle(value) {
  if (value?.startsWith("'") && value.endsWith("'")) return value.slice(1, -1);
  return value;
}

function parseFontList(boundary, source) {
  const start = source.indexOf(boundary.selector);
  const assignment = source.indexOf('=', start);
  const body = extractBalanced(source, assignment, '[', ']');
  const objects = topLevelObjects(body);
  const rows = objects.map(objectSource => {
    const name = unquoteSingle(propertyExpression(objectSource, 'name'));
    const file = propertyExpression(objectSource, 'file');
    const format = unquoteSingle(propertyExpression(objectSource, 'format')) ?? null;
    const canvasKitFile = propertyExpression(objectSource, 'canvasKitFile');
    if (!name || !file) throw new Error(`FONT_LIST object missing name/file: ${objectSource.slice(0, 120)}`);
    return candidate(
      boundary,
      'supply-source',
      'supply',
      name,
      file,
      {
        conditions: { format, canvasKitFile },
        backends: ['studio', 'canvas2d', 'canvaskit'],
      },
    );
  });
  return extracted('typescript-font-entry-objects', rows, objects.length);
}

function parseCanvasKitPlan(boundary, source) {
  const functionAt = source.indexOf(boundary.selector);
  const bodyAt = source.indexOf('): CanvasKitFontPlan {', functionAt);
  if (functionAt === -1 || bodyAt === -1) {
    throw new Error(`${boundary.candidateId}: CanvasKit plan body not found`);
  }
  const body = extractBalanced(source, bodyAt, '{', '}');
  const mapAt = body.indexOf('const canvasKitSubstitutes = new Map([');
  const mapBody = extractBalanced(body, mapAt, '[', ']');
  const mappings = [...mapBody.matchAll(
    /\[normalizeFontFamily\('([^']+)'\),\s*normalizeFontFamily\('([^']+)'\)\]/g,
  )];
  const rows = mappings.map(match => candidate(
    boundary,
    'finite-mapping',
    'supply',
    match[1],
    match[2],
    { backends: ['canvaskit'] },
  ));
  rows.push(candidate(
    boundary,
    'predicate',
    'supply',
    null,
    'resolve requested families to FONT_LIST bytes, group aliases by URL, and fail unavailable fonts closed',
    { backends: ['canvaskit'] },
  ));
  return extracted('typescript-canvaskit-plan', rows, mappings.length + 1);
}

function parseMarkdownRows(boundary, source, endMarker) {
  const start = source.indexOf(boundary.selector);
  const end = endMarker ? source.indexOf(endMarker, start + boundary.selector.length) : -1;
  const section = source.slice(start, end === -1 ? undefined : end);
  const rows = [];
  for (const line of section.split('\n')) {
    if (!/^\|.*\|\s*$/.test(line)) continue;
    const cells = line.slice(1, line.lastIndexOf('|')).split('|').map(cell => cell.trim());
    if (cells.length < 2 || cells.every(cell => /^:?-+:?$/.test(cell))) continue;
    if (['파일명', '폰트/리소스', '항목'].includes(cells[0])) continue;
    rows.push(candidate(
      boundary,
      'supply-source',
      'supply',
      cells[0],
      cells.slice(1).join(' | '),
      { backends: ['asset'] },
    ));
  }
  return extracted('markdown-table-rows', rows, rows.length);
}

function extractBoundary(boundary, source) {
  switch (`${boundary.ownerId}.${boundary.selectorId}`) {
    case 'rust-style-resolution.legacy-latin':
      return parseRustSomeMappings(boundary, source, 'english-only');
    case 'rust-style-resolution.hft':
      return parseRustSomeMappings(boundary, source, 'hft');
    case 'rust-style-resolution.ttf':
      return parseRustSomeMappings(boundary, source, 'all');
    case 'rust-style-resolution.heavy-display':
      return predicate(boundary, 'paint', 'primary face matches the curated heavy-display predicate');
    case 'rust-metric.metric-alias':
      return parseMetricAliases(boundary, source);
    case 'rust-metric.metric-table':
      return parseMetricTable(boundary, source);
    case 'rust-metric.metric-lookup':
      return orderedPolicies(boundary, 'layout-metric', [
        'match name+bold+italic',
        'match name+bold with italic=false',
        'use first physical entry for name and mark requested bold as synthetic fallback',
      ]);
    case 'rust-measurement.estimate-width':
      return predicate(boundary, 'layout-metric', 'estimate cluster and character advances using embedded metrics and guarded heuristics');
    case 'rust-measurement.hancom-space':
      return predicate(boundary, 'layout-metric', 'apply face- and size-gated Hancom regenerated-space width');
    case 'rust-paint-chain.installed-aliases':
      return parseInstalledAliases(boundary, source);
    case 'rust-paint-chain.weight-suffix':
      return predicate(boundary, 'paint', 'strip recognized trailing weight tokens while preserving at least one family token');
    case 'rust-paint-chain.generic-fallback':
      return predicate(boundary, 'paint', 'classify KoPub, monospace, serif, and sans families and return the corresponding ordered CSS chain');
    case 'native-skia.system-family-style':
      return predicate(boundary, 'backend-resource', 'require exact system-family membership before style lookup and cache the result', ['native-skia']);
    case 'native-skia.text-replay':
      return orderedPolicies(boundary, 'paint', [
        'custom typeface chain',
        'exact system typeface chain',
        'bundled typeface chain',
        'legacy FontMgr typeface',
      ], ['native-skia']);
    case 'paint-resource.resource-table':
      return predicate(boundary, 'backend-resource', 'preserve blob digest, source, exact face index, localized names, and portability state');
    case 'paint-resource.fallback-policy':
      return predicate(boundary, 'backend-resource', 'carry the selected fallback policy as an explicit resource identifier');
    case 'studio-substitution.substitution-tables':
      return parseStudioSubstitutions(boundary, source);
    case 'studio-substitution.display-chain':
      return orderedPolicies(boundary, 'paint', [
        'confirmed exact local or registered original family',
        'confirmed government successor family',
        'resolved web substitution family',
        'confirmed or registered document substFont family',
        'system fallback and generic terminal families',
      ], ['studio', 'canvas2d']);
    case 'studio-supply.font-list':
      return parseFontList(boundary, source);
    case 'studio-supply.canvaskit-plan':
      return parseCanvasKitPlan(boundary, source);
    case 'studio-detection.detection-method':
      return orderedPolicies(boundary, 'detection', [
        'use Local Font Access when queryLocalFonts exists',
        'use width probe when CanvasRenderingContext2D.measureText exists',
        'report unsupported when neither capability exists',
      ], ['studio']);
    case 'studio-detection.presence-probe':
      return predicate(boundary, 'detection', 'compare candidate widths against monospace, serif, and sans-serif probes', ['studio']);
    case 'studio-detection.sfnt-bytes':
      return predicate(boundary, 'detection', 'read approved exact PostScript faces in one batch without persistent byte caching', ['studio', 'canvaskit']);
    case 'studio-canvas-patch.css-family-substitution':
      return predicate(boundary, 'paint', 'replace the first quoted Canvas2D family with the display fallback chain', ['canvas2d']);
    case 'studio-canvas-patch.canvas-install':
      return predicate(boundary, 'detection', 'install the CanvasRenderingContext2D font descriptor patch only when the capability contract is complete', ['canvas2d']);
    case 'asset-authority.asset-index':
      return parseMarkdownRows(boundary, source, null);
    case 'asset-authority.metric-source-index':
      return parseMarkdownRows(boundary, source, '## 요약');
    case 'asset-authority.license-index':
      return parseMarkdownRows(boundary, source, '## 도구');
    case 'tests-history.studio-substitution-test':
      return extracted('test-evidence-anchor', [candidate(
        boundary,
        'evidence',
        'oracle',
        null,
        'regression test for exact legacy, official successor, and document substitute order',
        { backends: ['studio'] },
      )]);
    case 'tests-history.government-font-matrix':
      return extracted('historical-evidence-anchor', [candidate(
        boundary,
        'evidence',
        'oracle',
        '정부상징 부처명_16040911',
        'ROKG successor evidence matrix and profile boundaries',
        { backends: ['oracle'] },
      )]);
    default:
      throw new Error(`no extractor registered for ${boundary.ownerId}.${boundary.selectorId}`);
  }
}

function summaryFor(ruleCandidates, dispositions) {
  return {
    totalCount: ruleCandidates.length,
    countsByOwner: countBy(ruleCandidates.map(entry => entry.ownerId)),
    countsByKind: countBy(ruleCandidates.map(entry => entry.candidateKind)),
    countsByDecisionPlane: countBy(ruleCandidates.map(entry => entry.decisionPlane)),
    countsByBackend: countBy(ruleCandidates.flatMap(entry => entry.backends)),
    dispositionCount: dispositions.length,
    notApplicableCount: dispositions.filter(entry => entry.status === 'not-applicable').length,
    unrecognizedMappingBlockCount: dispositions
      .reduce((count, entry) => count + entry.unrecognizedMappingBlocks, 0),
    projectionSha256: sha256Text(canonicalJson(ruleCandidates)),
  };
}

export function collectRuleCandidates(sourceBoundarySnapshot, repositoryRoot = REPOSITORY_ROOT) {
  if (!isObject(sourceBoundarySnapshot)
      || sourceBoundarySnapshot.kind !== 'font-rule-source-candidates'
      || !Array.isArray(sourceBoundarySnapshot.candidates)) {
    throw new Error('Stage 3 requires the Stage 2 font-rule-source-candidates snapshot');
  }
  const ruleCandidates = [];
  const dispositions = [];
  for (const boundary of sourceBoundarySnapshot.candidates) {
    const sourcePath = path.resolve(repositoryRoot, boundary.path);
    const currentDigest = sha256File(sourcePath);
    if (currentDigest !== boundary.sourceSha256) {
      throw new Error(`source digest changed since W0: ${boundary.path}`);
    }
    const result = extractBoundary(boundary, fs.readFileSync(sourcePath, 'utf8'));
    ruleCandidates.push(...result.rows);
    dispositions.push({
      sourceBoundaryId: `${boundary.ownerId}.${boundary.selectorId}`,
      strategy: result.strategy,
      status: 'extracted',
      candidateCount: result.rows.length,
      recognizedMappingBlocks: result.recognizedMappingBlocks,
      unrecognizedMappingBlocks: result.unrecognizedMappingBlocks,
    });
  }
  const snapshot = {
    ...sourceBoundarySnapshot,
    ruleCandidateKind: 'font-rule-candidates',
    ruleGeneratorVersion: GENERATOR_VERSION,
    ruleCandidates,
    dispositions,
    summary: summaryFor(ruleCandidates, dispositions),
  };
  const errors = validateCandidateSnapshot(snapshot, repositoryRoot);
  if (errors.length > 0) throw new Error(errors.join('\n'));
  return snapshot;
}

export function validateCandidateSnapshot(snapshot, repositoryRoot = REPOSITORY_ROOT) {
  const errors = [];
  if (!isObject(snapshot)
      || snapshot.kind !== 'font-rule-source-candidates'
      || snapshot.ruleCandidateKind !== 'font-rule-candidates') {
    return ['snapshot must preserve the W0 source kind and declare ruleCandidateKind'];
  }
  if (!Array.isArray(snapshot.candidates) || snapshot.candidates.length === 0) {
    return ['snapshot.candidates must not be empty'];
  }
  if (!Array.isArray(snapshot.ruleCandidates) || !Array.isArray(snapshot.dispositions)) {
    return ['snapshot must contain ruleCandidates[] and dispositions[]'];
  }
  const boundaries = new Map(snapshot.candidates.map(boundary => [
    `${boundary.ownerId}.${boundary.selectorId}`,
    boundary,
  ]));
  const dispositions = new Map(snapshot.dispositions.map(entry => [entry.sourceBoundaryId, entry]));
  for (const boundaryId of boundaries.keys()) {
    const disposition = dispositions.get(boundaryId);
    if (!disposition) {
      errors.push(`missing disposition for ${boundaryId}`);
      continue;
    }
    if (disposition.status !== 'extracted' && disposition.status !== 'not-applicable') {
      errors.push(`${boundaryId}: unknown disposition status`);
    }
    if (disposition.status === 'extracted' && disposition.candidateCount < 1) {
      errors.push(`${boundaryId}: candidateCount must be positive`);
    }
    const actualCount = snapshot.ruleCandidates
      .filter(candidateEntry => candidateEntry.sourceBoundaryId === boundaryId).length;
    if (actualCount !== disposition.candidateCount) {
      errors.push(`${boundaryId}: disposition count ${disposition.candidateCount} != actual ${actualCount}`);
    }
    if (disposition.unrecognizedMappingBlocks !== 0) {
      errors.push(`${boundaryId}: unrecognized mapping blocks must be zero`);
    }
  }
  for (const dispositionId of dispositions.keys()) {
    if (!boundaries.has(dispositionId)) errors.push(`orphan disposition: ${dispositionId}`);
  }

  const ids = new Set();
  const currentDigests = new Map();
  for (const candidateEntry of snapshot.ruleCandidates) {
    if (!boundaries.has(candidateEntry.sourceBoundaryId)) {
      errors.push(`${candidateEntry.candidateId}: unknown sourceBoundaryId ${candidateEntry.sourceBoundaryId}`);
      continue;
    }
    if (ids.has(candidateEntry.candidateId)) errors.push(`duplicate candidateId: ${candidateEntry.candidateId}`);
    ids.add(candidateEntry.candidateId);
    const boundary = boundaries.get(candidateEntry.sourceBoundaryId);
    if (candidateEntry.sourceLocation.sourceSha256 !== boundary.sourceSha256) {
      errors.push(`${candidateEntry.candidateId}: source digest does not match boundary`);
    }
    const sourcePath = path.resolve(repositoryRoot, candidateEntry.sourceLocation.path);
    if (!sourcePath.startsWith(`${path.resolve(repositoryRoot)}${path.sep}`)) {
      errors.push(`${candidateEntry.candidateId}: source path escapes repository`);
    } else {
      const digest = currentDigests.get(sourcePath) ?? sha256File(sourcePath);
      currentDigests.set(sourcePath, digest);
      if (digest !== candidateEntry.sourceLocation.sourceSha256) {
        errors.push(`${candidateEntry.candidateId}: source digest is stale`);
      }
    }
  }
  const ownerIds = new Set(snapshot.candidates.map(boundary => boundary.ownerId));
  for (const ownerId of ownerIds) {
    const count = snapshot.ruleCandidates.filter(entry => entry.ownerId === ownerId).length;
    if (count === 0) errors.push(`${ownerId}: owner has no candidate or not-applicable disposition`);
  }
  const expectedSummary = summaryFor(snapshot.ruleCandidates, snapshot.dispositions);
  if (canonicalJson(expectedSummary) !== canonicalJson(snapshot.summary)) {
    errors.push('snapshot.summary does not match candidate projection');
  }
  return errors;
}

function argumentValue(args, name) {
  const index = args.indexOf(name);
  if (index === -1 || index === args.length - 1) return null;
  return args[index + 1];
}

const invokedPath = process.argv[1] ? path.resolve(process.argv[1]) : '';
if (invokedPath === fileURLToPath(import.meta.url)) {
  try {
    if (process.argv[2] !== 'collect') {
      throw new Error('usage: node scripts/font_rule_candidates.mjs collect --in <path> --out <path>');
    }
    const inputArgument = argumentValue(process.argv.slice(3), '--in');
    const outputArgument = argumentValue(process.argv.slice(3), '--out');
    if (!inputArgument || !outputArgument) throw new Error('collect requires --in <path> --out <path>');
    const inputPath = path.resolve(process.cwd(), inputArgument);
    const outputPath = path.resolve(process.cwd(), outputArgument);
    const snapshot = collectRuleCandidates(readJson(inputPath), REPOSITORY_ROOT);
    fs.writeFileSync(outputPath, canonicalJson(snapshot), 'utf8');
    process.stdout.write(
      `font rule candidates: ${snapshot.ruleCandidates.length}, boundaries: ${snapshot.dispositions.length}, unrecognized: ${snapshot.summary.unrecognizedMappingBlockCount}\n`,
    );
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
