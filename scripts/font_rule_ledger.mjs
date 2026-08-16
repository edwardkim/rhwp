#!/usr/bin/env node

import fs from 'node:fs';
import crypto from 'node:crypto';
import path from 'node:path';
import process from 'node:process';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const SCRIPT_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const SCHEMA_PATH = path.join(
  SCRIPT_ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4939',
  'font_rule_ledger.schema.json',
);

const REQUIRED_OWNER_IDS = [
  'rust-style-resolution',
  'rust-metric',
  'rust-measurement',
  'rust-paint-chain',
  'native-skia',
  'paint-resource',
  'studio-substitution',
  'studio-supply',
  'studio-detection',
  'studio-canvas-patch',
  'asset-authority',
  'tests-history',
];

const RULE_REQUIRED_FIELDS = [
  'ruleId',
  'sourceOwner',
  'sourceLocation',
  'decisionPlane',
  'relationType',
  'sourceFace',
  'targetFaceOrPolicy',
  'conditions',
  'backends',
  'order',
  'evidence',
  'evidenceStatus',
  'licenseOrDistribution',
  'tests',
  'knownLimitations',
  'status',
];

const CONDITION_FIELDS = [
  'languageSlot',
  'altType',
  'bold',
  'italic',
  'weight',
  'availability',
  'profile',
];

const GENERATOR_VERSION = '2.0.0';
const PUBLIC_PARITY_FIXTURES = [
  'samples/exam_kor.hwp',
  'samples/exam_eng.hwp',
  'samples/exam_math.hwp',
  'samples/exam_science.hwp',
  'samples/synam-001.hwp',
  'samples/aift.hwp',
  'samples/2010-01-06.hwp',
];

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function nonEmptyString(value) {
  return typeof value === 'string' && value.length > 0;
}

function canonicalValue(value) {
  if (Array.isArray(value)) return value.map(canonicalValue);
  if (!isObject(value)) return value;
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

function valuesFromSchema(property) {
  return new Set(property.enum);
}

function ruleEnums() {
  const rule = readJson(SCHEMA_PATH).$defs.rule.properties;
  return {
    decisionPlane: valuesFromSchema(rule.decisionPlane),
    relationType: valuesFromSchema(rule.relationType),
    backend: valuesFromSchema(rule.backends.items),
    evidenceStatus: valuesFromSchema(rule.evidenceStatus),
    evidenceKind: valuesFromSchema(rule.evidence.items.properties.kind),
    status: valuesFromSchema(rule.status),
  };
}

function rejectUnknownFields(value, allowed, location, errors) {
  for (const key of Object.keys(value)) {
    if (!allowed.includes(key)) errors.push(`${location}: unexpected field ${key}`);
  }
}

function validateNullableString(value, location, errors) {
  if (value !== null && !nonEmptyString(value)) {
    errors.push(`${location} must be null or a non-empty string`);
  }
}

function validateRule(rule, index, enums, errors) {
  const location = `rules[${index}]`;
  if (!isObject(rule)) {
    errors.push(`${location} must be an object`);
    return;
  }

  rejectUnknownFields(rule, RULE_REQUIRED_FIELDS, location, errors);
  for (const field of RULE_REQUIRED_FIELDS) {
    if (!Object.hasOwn(rule, field)) errors.push(`${location}.${field} is required`);
  }

  if (!nonEmptyString(rule.ruleId) || !/^[a-z0-9]+(?:[.-][a-z0-9]+)*$/.test(rule.ruleId)) {
    errors.push(`${location}.ruleId must be a stable lowercase semantic ID`);
  }
  if (!nonEmptyString(rule.sourceOwner)) errors.push(`${location}.sourceOwner must not be empty`);

  if (!isObject(rule.sourceLocation)) {
    errors.push(`${location}.sourceLocation must be an object`);
  } else {
    const fields = ['path', 'symbol', 'selector'];
    rejectUnknownFields(rule.sourceLocation, fields, `${location}.sourceLocation`, errors);
    for (const field of fields) {
      if (!nonEmptyString(rule.sourceLocation[field])) {
        errors.push(`${location}.sourceLocation.${field} must not be empty`);
      }
    }
  }

  for (const field of ['decisionPlane', 'relationType', 'evidenceStatus', 'status']) {
    if (!enums[field].has(rule[field])) {
      errors.push(`${location}.${field} has unknown value ${JSON.stringify(rule[field])}`);
    }
  }

  validateNullableString(rule.sourceFace, `${location}.sourceFace`, errors);
  if (!nonEmptyString(rule.targetFaceOrPolicy)) {
    errors.push(`${location}.targetFaceOrPolicy must not be empty`);
  }

  if (!isObject(rule.conditions)) {
    errors.push(`${location}.conditions must be an object`);
  } else {
    rejectUnknownFields(rule.conditions, CONDITION_FIELDS, `${location}.conditions`, errors);
    for (const field of ['languageSlot', 'altType', 'weight', 'availability', 'profile']) {
      if (Object.hasOwn(rule.conditions, field)) {
        validateNullableString(rule.conditions[field], `${location}.conditions.${field}`, errors);
      }
    }
    for (const field of ['bold', 'italic']) {
      const value = rule.conditions[field];
      if (Object.hasOwn(rule.conditions, field) && value !== null && typeof value !== 'boolean') {
        errors.push(`${location}.conditions.${field} must be null or boolean`);
      }
    }
  }

  if (!Array.isArray(rule.backends) || rule.backends.length === 0) {
    errors.push(`${location}.backends must be a non-empty array`);
  } else {
    const seen = new Set();
    for (const backend of rule.backends) {
      if (!enums.backend.has(backend)) errors.push(`${location}.backends has unknown value ${backend}`);
      if (seen.has(backend)) errors.push(`${location}.backends has duplicate value ${backend}`);
      seen.add(backend);
    }
  }

  if (rule.order !== null && (!Number.isInteger(rule.order) || rule.order < 0)) {
    errors.push(`${location}.order must be null or a non-negative integer`);
  }

  if (!Array.isArray(rule.evidence)) {
    errors.push(`${location}.evidence must be an array`);
  } else {
    rule.evidence.forEach((entry, evidenceIndex) => {
      const evidenceLocation = `${location}.evidence[${evidenceIndex}]`;
      if (!isObject(entry)) {
        errors.push(`${evidenceLocation} must be an object`);
        return;
      }
      rejectUnknownFields(entry, ['kind', 'reference'], evidenceLocation, errors);
      if (!enums.evidenceKind.has(entry.kind)) {
        errors.push(`${evidenceLocation}.kind has unknown value ${JSON.stringify(entry.kind)}`);
      }
      if (!nonEmptyString(entry.reference)) {
        errors.push(`${evidenceLocation}.reference must not be empty`);
      }
    });
  }

  if (!nonEmptyString(rule.licenseOrDistribution)) {
    errors.push(`${location}.licenseOrDistribution must not be empty`);
  }
  for (const field of ['tests', 'knownLimitations']) {
    if (!Array.isArray(rule[field]) || rule[field].some(value => !nonEmptyString(value))) {
      errors.push(`${location}.${field} must be an array of non-empty strings`);
    }
  }
}

export function validateLedger(ledger) {
  const errors = [];
  if (!isObject(ledger)) return ['ledger must be an object'];
  rejectUnknownFields(ledger, ['schemaVersion', 'kind', 'issue', 'sourceCommit', 'rules'], 'ledger', errors);
  if (ledger.schemaVersion !== '1.0') errors.push('ledger.schemaVersion must be "1.0"');
  if (ledger.kind !== 'font-rule-investigation-ledger') {
    errors.push('ledger.kind must be "font-rule-investigation-ledger"');
  }
  if (ledger.issue !== 4939) errors.push('ledger.issue must be 4939');
  if (typeof ledger.sourceCommit !== 'string' || !/^[0-9a-f]{40}$/.test(ledger.sourceCommit)) {
    errors.push('ledger.sourceCommit must be a lowercase 40-character Git SHA');
  }
  if (!Array.isArray(ledger.rules)) {
    errors.push('ledger.rules must be an array');
    return errors;
  }

  const enums = ruleEnums();
  ledger.rules.forEach((rule, index) => validateRule(rule, index, enums, errors));
  const seenRuleIds = new Set();
  for (const rule of ledger.rules) {
    if (!isObject(rule) || !nonEmptyString(rule.ruleId)) continue;
    if (seenRuleIds.has(rule.ruleId)) errors.push(`duplicate ruleId: ${rule.ruleId}`);
    seenRuleIds.add(rule.ruleId);
  }
  return errors;
}

function commonFixtureRule(snippet) {
  return {
    sourceOwner: snippet.sourceOwner,
    sourceLocation: snippet.sourceLocation,
    decisionPlane: snippet.decisionPlane,
    relationType: snippet.relationType,
    conditions: snippet.conditions,
    backends: snippet.backends,
    evidence: [],
    evidenceStatus: 'unknown',
    licenseOrDistribution: 'not-assessed',
    tests: ['scripts/tests/font_rule_ledger.test.mjs'],
    knownLimitations: ['Stage 1 fixture; not a production rule'],
    status: 'candidate',
  };
}

export function expandFixtureSnippets(fixture) {
  if (!isObject(fixture) || fixture.schemaVersion !== '1.0' || !Array.isArray(fixture.snippets)) {
    throw new Error('fixture must contain schemaVersion 1.0 and snippets[]');
  }
  const rules = [];
  for (const snippet of fixture.snippets) {
    const common = commonFixtureRule(snippet);
    if (snippet.kind === 'grouped-mapping') {
      snippet.sourceFaces.forEach((sourceFace, index) => {
        rules.push({
          ruleId: `${snippet.ruleIdPrefix}.${index + 1}`,
          ...common,
          sourceFace,
          targetFaceOrPolicy: snippet.targetFaceOrPolicy,
          order: null,
        });
      });
    } else if (snippet.kind === 'ordered-chain') {
      snippet.targets.forEach((targetFaceOrPolicy, index) => {
        rules.push({
          ruleId: `${snippet.ruleIdPrefix}.${index + 1}`,
          ...common,
          sourceFace: snippet.sourceFace,
          targetFaceOrPolicy,
          order: index,
        });
      });
    } else if (snippet.kind === 'predicate') {
      rules.push({
        ruleId: snippet.ruleId,
        ...common,
        sourceFace: snippet.sourceFace,
        targetFaceOrPolicy: snippet.targetFaceOrPolicy,
        order: null,
      });
    } else {
      throw new Error(`unknown fixture snippet kind: ${snippet.kind}`);
    }
  }
  return {
    schemaVersion: '1.0',
    kind: 'font-rule-investigation-ledger',
    issue: 4939,
    sourceCommit: fixture.sourceCommit,
    rules,
  };
}

function countLiteral(source, needle) {
  if (!needle) return 0;
  let count = 0;
  let position = 0;
  while ((position = source.indexOf(needle, position)) !== -1) {
    count += 1;
    position += needle.length;
  }
  return count;
}

export function assertSourceBoundary(manifest, repositoryRoot = SCRIPT_ROOT) {
  const errors = [];
  if (!isObject(manifest)) return ['source boundary manifest must be an object'];
  if (manifest.schemaVersion !== '1.0') errors.push('source boundary schemaVersion must be "1.0"');
  if (manifest.kind !== 'font-rule-source-boundary') {
    errors.push('source boundary kind must be "font-rule-source-boundary"');
  }
  if (!Array.isArray(manifest.owners) || manifest.owners.length === 0) {
    errors.push('source boundary owners must not be empty');
    return errors;
  }

  const ownerIds = manifest.owners.map(owner => owner.ownerId);
  for (const ownerId of REQUIRED_OWNER_IDS) {
    if (!ownerIds.includes(ownerId)) errors.push(`missing owner: ${ownerId}`);
  }
  const duplicateOwners = ownerIds.filter((ownerId, index) => ownerIds.indexOf(ownerId) !== index);
  for (const ownerId of new Set(duplicateOwners)) errors.push(`duplicate owner: ${ownerId}`);

  const selectorIds = new Set();
  for (const owner of manifest.owners) {
    const ownerLocation = `owner ${JSON.stringify(owner.ownerId)}`;
    if (!nonEmptyString(owner.ownerId)) errors.push(`${ownerLocation} ownerId must not be empty`);
    if (!nonEmptyString(owner.scope)) errors.push(`${ownerLocation} scope must not be empty`);
    if (!Array.isArray(owner.selectors) || owner.selectors.length === 0) {
      errors.push(`${ownerLocation} selectors must not be empty`);
      continue;
    }
    for (const selector of owner.selectors) {
      const selectorKey = `${owner.ownerId}/${selector.selectorId}`;
      if (selectorIds.has(selectorKey)) errors.push(`duplicate selector: ${selectorKey}`);
      selectorIds.add(selectorKey);
      if (selector.matchMode !== 'literal') {
        errors.push(`${selectorKey}: only literal matchMode is allowed in Stage 1`);
        continue;
      }
      if (!Number.isInteger(selector.minMatches) || selector.minMatches < 1) {
        errors.push(`${selectorKey}: minMatches must be a positive integer`);
        continue;
      }
      if (!nonEmptyString(selector.path) || path.isAbsolute(selector.path)) {
        errors.push(`${selectorKey}: path must be repository-relative`);
        continue;
      }
      const sourcePath = path.resolve(repositoryRoot, selector.path);
      const rootPrefix = `${path.resolve(repositoryRoot)}${path.sep}`;
      if (!sourcePath.startsWith(rootPrefix)) {
        errors.push(`${selectorKey}: path escapes repository root`);
        continue;
      }
      if (!fs.existsSync(sourcePath) || !fs.statSync(sourcePath).isFile()) {
        errors.push(`${selectorKey}: source file does not exist: ${selector.path}`);
        continue;
      }
      const source = fs.readFileSync(sourcePath, 'utf8');
      const matches = countLiteral(source, selector.selector);
      if (matches < selector.minMatches) {
        errors.push(
          `${selectorKey}: selector matched ${matches} time(s), expected at least ${selector.minMatches}`,
        );
      }
    }
  }
  return errors;
}

function stringCompare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

export function collectSourceCandidates(manifest, repositoryRoot, sourceCommit) {
  const boundaryErrors = assertSourceBoundary(manifest, repositoryRoot);
  if (boundaryErrors.length > 0) throw new Error(boundaryErrors.join('\n'));
  if (typeof sourceCommit !== 'string' || !/^[0-9a-f]{40}$/.test(sourceCommit)) {
    throw new Error('sourceCommit must be a lowercase 40-character Git SHA');
  }

  const candidates = [];
  for (const owner of manifest.owners) {
    for (const selector of owner.selectors) {
      const sourcePath = path.resolve(repositoryRoot, selector.path);
      const source = fs.readFileSync(sourcePath, 'utf8');
      candidates.push({
        candidateId: `${owner.ownerId}.${selector.selectorId}`,
        ownerId: owner.ownerId,
        selectorId: selector.selectorId,
        path: selector.path,
        symbol: selector.symbol,
        selector: selector.selector,
        extractionMode: selector.extractionMode,
        minMatches: selector.minMatches,
        matchCount: countLiteral(source, selector.selector),
        sourceSha256: sha256File(sourcePath),
      });
    }
  }

  return {
    schemaVersion: '1.0',
    kind: 'font-rule-source-candidates',
    sourceCommit,
    generatorVersion: GENERATOR_VERSION,
    sourcesManifestSha256: sha256Text(canonicalJson(manifest)),
    candidates,
  };
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
      if (depth === 0) return source.slice(openAt + 1, index);
    }
  }
  throw new Error(`closing ${closeCharacter} not found`);
}

function decodeQuotedString(value) {
  return JSON.parse(`"${value}"`);
}

function parseMetricEntries(source) {
  const declarationAt = source.indexOf('static FONT_METRICS:');
  if (declarationAt === -1) throw new Error('FONT_METRICS declaration not found');
  const assignmentAt = source.indexOf('=', declarationAt);
  const body = extractBalanced(source, assignmentAt, '[', ']');
  const rangeCounts = new Map();
  for (const match of source.matchAll(/static\s+([A-Z0-9_]+):\s*\[LatinRange;\s*(\d+)\]/g)) {
    rangeCounts.set(match[1], Number.parseInt(match[2], 10));
  }

  const entryPattern = /FontMetric\s*\{\s*name:\s*"((?:\\.|[^"\\])*)",\s*bold:\s*(true|false),\s*italic:\s*(true|false),\s*em_size:\s*(\d+),\s*latin_ranges:\s*&([A-Z0-9_]+),\s*hangul:\s*(?:Some\(&([A-Z0-9_]+)\)|None),\s*\}/g;
  const entries = [];
  for (const match of body.matchAll(entryPattern)) {
    const latinRangeSymbol = match[5];
    if (!rangeCounts.has(latinRangeSymbol)) {
      throw new Error(`LatinRange declaration not found: ${latinRangeSymbol}`);
    }
    entries.push({
      index: entries.length,
      name: decodeQuotedString(match[1]),
      bold: match[2] === 'true',
      italic: match[3] === 'true',
      emSize: Number.parseInt(match[4], 10),
      latinRanges: {
        symbol: latinRangeSymbol,
        count: rangeCounts.get(latinRangeSymbol),
      },
      hangulSymbol: match[6] ?? null,
    });
  }
  const declaredCountMatch = source
    .slice(declarationAt, assignmentAt)
    .match(/\[FontMetric;\s*(\d+)\]/);
  const declaredCount = declaredCountMatch ? Number.parseInt(declaredCountMatch[1], 10) : null;
  if (declaredCount === null || entries.length !== declaredCount) {
    throw new Error(`FONT_METRICS parse count ${entries.length} does not match declaration ${declaredCount}`);
  }
  return entries;
}

function parseMetricAliases(source) {
  const functionAt = source.indexOf('fn resolve_metric_alias(');
  if (functionAt === -1) throw new Error('resolve_metric_alias function not found');
  const body = extractBalanced(source, functionAt, '{', '}');
  const aliases = [];
  const armPattern = /((?:"(?:\\.|[^"\\])*"\s*(?:\|\s*)?)+)=>\s*"((?:\\.|[^"\\])*)"\s*,/gs;
  for (const match of body.matchAll(armPattern)) {
    const target = decodeQuotedString(match[2]);
    for (const sourceMatch of match[1].matchAll(/"((?:\\.|[^"\\])*)"/g)) {
      aliases.push({ source: decodeQuotedString(sourceMatch[1]), target });
    }
  }
  if (aliases.length === 0) throw new Error('resolve_metric_alias produced zero aliases');
  return aliases;
}

function metricProjection(entries, aliases) {
  const aliasMap = new Map(aliases.map(alias => [alias.source, alias.target]));
  const knownInputs = [];
  const inputSeen = new Set();
  for (const name of entries.map(entry => entry.name)) {
    if (!inputSeen.has(name)) {
      inputSeen.add(name);
      knownInputs.push(name);
    }
  }
  for (const alias of aliases) {
    if (!inputSeen.has(alias.source)) {
      inputSeen.add(alias.source);
      knownInputs.push(alias.source);
    }
  }
  knownInputs.push('__rhwp_font_rule_ledger_unregistered__');

  const projection = [];
  for (const inputName of knownInputs) {
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
          metricEntryIndex: selected?.index ?? null,
          metricName: selected?.name ?? null,
          boldFallback: selected === null ? null : exact !== undefined || boldOnly !== undefined ? false : bold,
        });
      }
    }
  }
  return { knownInputs, projection };
}

function countBy(values) {
  const counts = {};
  for (const value of values) counts[value] = (counts[value] ?? 0) + 1;
  return Object.fromEntries(Object.entries(counts).sort(([left], [right]) => stringCompare(left, right)));
}

function metricDuplicates(entries) {
  const groups = new Map();
  for (const entry of entries) {
    const key = `${entry.name}\u0000${entry.bold}\u0000${entry.italic}`;
    const group = groups.get(key) ?? [];
    group.push(entry.index);
    groups.set(key, group);
  }
  return [...groups.entries()]
    .filter(([, indexes]) => indexes.length > 1)
    .map(([key, entryIndexes]) => {
      const [name, bold, italic] = key.split('\u0000');
      return {
        name,
        bold: bold === 'true',
        italic: italic === 'true',
        count: entryIndexes.length,
        entryIndexes,
      };
    });
}

function baselineInputs(candidates, repositoryRoot) {
  const inputsByPath = new Map();
  for (const candidate of candidates.candidates) {
    const input = inputsByPath.get(candidate.path) ?? {
      path: candidate.path,
      sha256: candidate.sourceSha256,
      owners: [],
      extractionModes: [],
    };
    if (!input.owners.includes(candidate.ownerId)) input.owners.push(candidate.ownerId);
    if (!input.extractionModes.includes(candidate.extractionMode)) {
      input.extractionModes.push(candidate.extractionMode);
    }
    inputsByPath.set(candidate.path, input);
  }

  const toolingPaths = [
    'mydocs/tech/investigations/issue-4939/font_rule_ledger.schema.json',
    'mydocs/tech/investigations/issue-4939/font_rule_sources.json',
    'scripts/font_rule_ledger.mjs',
    'scripts/tests/font_rule_ledger.test.mjs',
    'scripts/tests/fixtures/font-rule-ledger/source-snippets.json',
  ];
  for (const toolingPath of toolingPaths) {
    inputsByPath.set(toolingPath, {
      path: toolingPath,
      sha256: sha256File(path.resolve(repositoryRoot, toolingPath)),
      owners: ['stage2-tooling'],
      extractionModes: ['contract'],
    });
  }

  return [...inputsByPath.values()]
    .map(input => ({
      ...input,
      owners: [...input.owners].sort(stringCompare),
      extractionModes: [...input.extractionModes].sort(stringCompare),
    }))
    .sort((left, right) => stringCompare(left.path, right.path));
}

export function buildBaseline(candidates, repositoryRoot = SCRIPT_ROOT) {
  if (!isObject(candidates) || candidates.kind !== 'font-rule-source-candidates') {
    throw new Error('baseline requires a font-rule-source-candidates document');
  }
  if (!Array.isArray(candidates.candidates) || candidates.candidates.length === 0) {
    throw new Error('baseline candidate list must not be empty');
  }
  for (const candidate of candidates.candidates) {
    const currentDigest = sha256File(path.resolve(repositoryRoot, candidate.path));
    if (currentDigest !== candidate.sourceSha256) {
      throw new Error(`source digest changed after candidate collection: ${candidate.path}`);
    }
  }

  const metricsPath = path.join(repositoryRoot, 'src', 'renderer', 'font_metrics_data.rs');
  const metricsSource = fs.readFileSync(metricsPath, 'utf8');
  const entries = parseMetricEntries(metricsSource);
  const aliases = parseMetricAliases(metricsSource);
  const { knownInputs, projection } = metricProjection(entries, aliases);
  const styleCounts = {
    regular: entries.filter(entry => !entry.bold && !entry.italic).length,
    bold: entries.filter(entry => entry.bold && !entry.italic).length,
    italic: entries.filter(entry => !entry.bold && entry.italic).length,
    boldItalic: entries.filter(entry => entry.bold && entry.italic).length,
  };
  const fixtures = PUBLIC_PARITY_FIXTURES.map(fixturePath => ({
    id: path.basename(fixturePath, path.extname(fixturePath)),
    path: fixturePath,
    sha256: sha256File(path.resolve(repositoryRoot, fixturePath)),
    purpose: 'native-wasm-svg-byte-parity',
    backend: ['native', 'wasm'],
  }));

  return {
    schemaVersion: '1.0',
    kind: 'font-rule-baseline',
    issue: 4939,
    sourceCommit: candidates.sourceCommit,
    generatorVersion: GENERATOR_VERSION,
    inputs: baselineInputs(candidates, repositoryRoot),
    fontMetrics: {
      entryCount: entries.length,
      uniqueNameCount: new Set(entries.map(entry => entry.name)).size,
      styleCounts,
      duplicateKeys: metricDuplicates(entries),
      tableSha256: sha256Text(canonicalJson(entries)),
    },
    lookupContract: {
      exactOrder: ['name+bold+italic', 'name+bold+italic=false', 'name-first'],
      fallbackOrder: ['exact', 'bold-with-italic-false', 'first-name-entry'],
      aliasCount: aliases.length,
      aliasSha256: sha256Text(canonicalJson(aliases)),
      knownInputCount: knownInputs.length,
      projectionCount: projection.length,
      projectionSha256: sha256Text(canonicalJson(projection)),
      unregisteredSentinel: '__rhwp_font_rule_ledger_unregistered__',
    },
    ruleCandidates: {
      totalCount: candidates.candidates.length,
      countsByOwner: countBy(candidates.candidates.map(candidate => candidate.ownerId)),
      countsByKind: countBy(candidates.candidates.map(candidate => candidate.extractionMode)),
      projectionSha256: sha256Text(canonicalJson(candidates.candidates)),
    },
    fixtures,
    gates: [
      {
        id: 'font-rule-ledger-contract',
        command: 'node --test scripts/tests/font_rule_ledger.test.mjs',
        status: 'required',
        evidence: 'mydocs/working/task_m100_4939_stage2.md',
      },
      {
        id: 'rust-font-metrics',
        command: 'cargo test --profile release-test --lib font_metrics',
        status: 'required',
        evidence: 'mydocs/working/task_m100_4939_stage2.md',
      },
      {
        id: 'studio-font-contracts',
        command: 'node --test rhwp-studio/tests/font-substitution.test.ts rhwp-studio/tests/local-fonts.test.ts rhwp-studio/tests/canvaskit-font-plan.test.ts rhwp-studio/tests/canvaskit-sfnt-face.test.ts rhwp-studio/tests/renderer-baseline-font-loading.test.ts',
        status: 'required',
        evidence: 'mydocs/working/task_m100_4939_stage2.md',
      },
      {
        id: 'font-assets',
        command: 'node --test scripts/frontend-font-assets.test.mjs',
        status: 'required',
        evidence: 'mydocs/working/task_m100_4939_stage2.md',
      },
      {
        id: 'native-wasm-public-parity',
        command: `node scripts/svg_native_wasm_diff.mjs ${PUBLIC_PARITY_FIXTURES.join(' ')}`,
        status: 'required',
        evidence: 'mydocs/working/task_m100_4939_stage2.md',
      },
    ],
  };
}

function currentGitHead(repositoryRoot) {
  return execFileSync('git', ['rev-parse', 'HEAD'], {
    cwd: repositoryRoot,
    encoding: 'utf8',
  }).trim();
}

function writeCanonical(file, value) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, canonicalJson(value), 'utf8');
}

function argumentValue(args, name) {
  const index = args.indexOf(name);
  if (index === -1 || index === args.length - 1) return null;
  return args[index + 1];
}

function runBoundary(args) {
  const sourceArgument = argumentValue(args, '--sources');
  if (!sourceArgument) throw new Error('boundary requires --sources <path>');
  const sourcesPath = path.resolve(process.cwd(), sourceArgument);
  const errors = assertSourceBoundary(readJson(sourcesPath), SCRIPT_ROOT);
  if (errors.length > 0) throw new Error(errors.join('\n'));
  process.stdout.write('font rule source boundary: ok\n');
}

function runCollect(args) {
  const outputArgument = argumentValue(args, '--out');
  if (!outputArgument) throw new Error('collect requires --out <path>');
  const sourceArgument = argumentValue(args, '--sources')
    ?? 'mydocs/tech/investigations/issue-4939/font_rule_sources.json';
  const sourcesPath = path.resolve(process.cwd(), sourceArgument);
  const outputPath = path.resolve(process.cwd(), outputArgument);
  const candidates = collectSourceCandidates(
    readJson(sourcesPath),
    SCRIPT_ROOT,
    currentGitHead(SCRIPT_ROOT),
  );
  writeCanonical(outputPath, candidates);
  process.stdout.write(
    `font rule source candidates: ${candidates.candidates.length} -> ${path.relative(SCRIPT_ROOT, outputPath)}\n`,
  );
}

function runBaseline(args) {
  const candidateArgument = argumentValue(args, '--candidates');
  const outputArgument = argumentValue(args, '--out');
  if (!candidateArgument || !outputArgument) {
    throw new Error('baseline requires --candidates <path> --out <path>');
  }
  const candidatePath = path.resolve(process.cwd(), candidateArgument);
  const outputPath = path.resolve(process.cwd(), outputArgument);
  const candidates = readJson(candidatePath);
  const head = currentGitHead(SCRIPT_ROOT);
  if (candidates.sourceCommit !== head) {
    throw new Error(
      `candidate sourceCommit ${candidates.sourceCommit} does not match current HEAD ${head}`,
    );
  }
  const baseline = buildBaseline(candidates, SCRIPT_ROOT);
  writeCanonical(outputPath, baseline);
  process.stdout.write(
    `font rule baseline: ${baseline.fontMetrics.entryCount} metrics, ${baseline.lookupContract.knownInputCount} inputs -> ${path.relative(SCRIPT_ROOT, outputPath)}\n`,
  );
}

const invokedPath = process.argv[1] ? path.resolve(process.argv[1]) : '';
if (invokedPath === fileURLToPath(import.meta.url)) {
  try {
    const command = process.argv[2];
    if (command === 'boundary') {
      runBoundary(process.argv.slice(3));
    } else if (command === 'collect') {
      runCollect(process.argv.slice(3));
    } else if (command === 'baseline') {
      runBaseline(process.argv.slice(3));
    } else {
      throw new Error(
        'usage: node scripts/font_rule_ledger.mjs <boundary|collect|baseline> [options]',
      );
    }
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
