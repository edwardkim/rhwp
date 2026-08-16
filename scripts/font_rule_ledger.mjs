#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
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

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function nonEmptyString(value) {
  return typeof value === 'string' && value.length > 0;
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

const invokedPath = process.argv[1] ? path.resolve(process.argv[1]) : '';
if (invokedPath === fileURLToPath(import.meta.url)) {
  try {
    const command = process.argv[2];
    if (command === 'boundary') {
      runBoundary(process.argv.slice(3));
    } else {
      throw new Error('usage: node scripts/font_rule_ledger.mjs boundary --sources <path>');
    }
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
