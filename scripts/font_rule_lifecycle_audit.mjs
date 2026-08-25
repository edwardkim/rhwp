#!/usr/bin/env node

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { canonicalJson, sha256Text } from './font_rule_ledger.mjs';
import { createRuleLifecycleResolver } from './font_rule_registry_v2.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const REGISTRY_PATH = path.join(ROOT, 'assets', 'font-rules', 'font_rule_registry_v2.json');
const SCHEMA_PATH = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-5955',
  'font_rule_lifecycle_audit.schema.json',
);
const RULE_ID_PATTERN = /^[a-z0-9]+(?:[._:-][a-z0-9]+)*$/u;
const SHA256_PATTERN = /^[0-9a-f]{64}$/u;
const MAX_RECORDS = 4096;
const MAX_PROVENANCE_PER_RECORD = 64;
const MAX_BACKEND_REFERENCES_PER_RECORD = 4096;
const MAX_RULE_REFERENCES = 262144;
const MAX_INPUT_BYTES = 16 * 1024 * 1024;
const BACKENDS = ['native', 'canvas2d', 'canvaskit'];
const SUMMARY_FIELDS = Object.freeze({
  'carried-forward-active': 'carriedForwardActive',
  'introduced-active': 'introducedActive',
  'historical-reference-only': 'historicalReferenceOnly',
  'trace-declared-source-drift': 'traceSourceDrift',
  retired: 'retired',
  replaced: 'replaced',
  dangling: 'dangling',
});

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function sha256File(file) {
  return crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
}

function relativePath(file, root = ROOT) {
  return path.relative(root, file).split(path.sep).join('/');
}

function pathDigest(file, root = ROOT) {
  return { path: relativePath(file, root), sha256: sha256File(file) };
}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function validateHash(value, location) {
  if (!isObject(value) || value.algorithm !== 'sha256'
      || !(value.value === null || SHA256_PATTERN.test(value.value ?? ''))) {
    throw new Error(`${location} must contain a SHA-256 value or null`);
  }
  return value.value;
}

function validateRuleId(ruleId, location) {
  if (typeof ruleId !== 'string' || ruleId.length === 0 || ruleId.length > 2048
      || !RULE_ID_PATTERN.test(ruleId)) {
    throw new Error(`${location} must be a stable rule ID of at most 2,048 characters`);
  }
}

function appendReference(
  references,
  recordIndex,
  recordId,
  location,
  ruleId,
  traceReason = null,
) {
  validateRuleId(ruleId, location);
  if (![null, 'ledgerRuleMissing', 'ledgerSourceDrift'].includes(traceReason)) {
    throw new Error(`${location} has an unsupported trace provenance reason`);
  }
  if (references.length >= MAX_RULE_REFERENCES) {
    throw new Error(`trace exceeds the ${MAX_RULE_REFERENCES.toLocaleString('en-US')} rule reference bound`);
  }
  references.push({ recordIndex, recordId, location, ruleId, traceReason });
}

function extractTraceReferences(trace) {
  if (!isObject(trace) || trace.schemaVersion !== 1 || !Array.isArray(trace.records)) {
    throw new Error('trace must be a Font Decision Trace schemaVersion 1 envelope');
  }
  if (trace.records.length > MAX_RECORDS) {
    throw new Error('trace may contain at most 4,096 records');
  }
  const layoutHash = validateHash(trace.layoutHash, 'trace.layoutHash');
  const normalizedHash = validateHash(trace.normalizedHash, 'trace.normalizedHash');
  const references = [];
  trace.records.forEach((record, recordIndex) => {
    if (!isObject(record) || typeof record.recordId !== 'string'
        || record.recordId.length === 0 || record.recordId.length > 2048
        || !RULE_ID_PATTERN.test(record.recordId)) {
      throw new Error(`trace.records[${recordIndex}].recordId must be a stable bounded identifier`);
    }
    const provenance = record.provenance ?? [];
    if (!Array.isArray(provenance) || provenance.length > MAX_PROVENANCE_PER_RECORD) {
      throw new Error(`trace.records[${recordIndex}].provenance exceeds 64 entries`);
    }
    provenance.forEach((entry, provenanceIndex) => {
      if (!isObject(entry)) {
        throw new Error(`trace.records[${recordIndex}].provenance[${provenanceIndex}] must be an object`);
      }
      if (entry.ruleId === null || entry.ruleId === undefined) return;
      appendReference(
        references,
        recordIndex,
        record.recordId,
        `/records/${recordIndex}/provenance/${provenanceIndex}/ruleId`,
        entry.ruleId,
        entry.reason ?? null,
      );
    });
    if (record.paint !== undefined && !isObject(record.paint)) {
      throw new Error(`trace.records[${recordIndex}].paint must be an object`);
    }
    for (const backend of BACKENDS) {
      const decision = record.paint?.[backend];
      if (decision === undefined) continue;
      if (!isObject(decision)) {
        throw new Error(`trace.records[${recordIndex}].paint.${backend} must be an object`);
      }
      const ruleIds = decision.ruleIds ?? [];
      if (!Array.isArray(ruleIds) || ruleIds.length > MAX_BACKEND_REFERENCES_PER_RECORD) {
        throw new Error(
          `trace.records[${recordIndex}].paint.${backend}.ruleIds exceeds 4,096 entries`,
        );
      }
      ruleIds.forEach((ruleId, ruleIndex) => appendReference(
        references,
        recordIndex,
        record.recordId,
        `/records/${recordIndex}/paint/${backend}/ruleIds/${ruleIndex}`,
        ruleId,
      ));
    }
  });
  return { references, layoutHash, normalizedHash };
}

export function buildFontRuleLifecycleAudit(
  trace,
  registry = readJson(REGISTRY_PATH),
  { root = ROOT } = {},
) {
  const extracted = extractTraceReferences(trace);
  const resolver = createRuleLifecycleResolver(registry, { root });
  const references = extracted.references.map(reference => {
    const resolved = resolver.resolve(reference.ruleId);
    if (resolved.resolution === 'dangling' && reference.traceReason === 'ledgerSourceDrift') {
      return {
        ...reference,
        ...resolved,
        resolution: 'trace-declared-source-drift',
        reason: { code: 'traceDeclaredLedgerSourceDrift', eventId: 'issue-4961' },
      };
    }
    return { ...reference, ...resolved };
  });
  const summary = {
    carriedForwardActive: 0,
    introducedActive: 0,
    historicalReferenceOnly: 0,
    traceSourceDrift: 0,
    retired: 0,
    replaced: 0,
    dangling: 0,
  };
  for (const reference of references) summary[SUMMARY_FIELDS[reference.resolution]] += 1;
  return {
    schemaVersion: '1.0',
    kind: 'font-rule-lifecycle-trace-audit',
    issue: 5955,
    schema: pathDigest(SCHEMA_PATH, root),
    registry: {
      path: relativePath(REGISTRY_PATH, root),
      sha256: sha256Text(canonicalJson(registry)),
      rulesSha256: registry.rulesSha256,
      historicalLedger: resolver.historicalLedger,
    },
    trace: {
      schemaVersion: 1,
      recordCount: trace.records.length,
      ruleReferenceCount: references.length,
      uniqueRuleIdCount: new Set(references.map(reference => reference.ruleId)).size,
      layoutHash: extracted.layoutHash,
      normalizedHash: extracted.normalizedHash,
    },
    summary,
    referencesSha256: sha256Text(canonicalJson(references)),
    references,
  };
}

export function validateFontRuleLifecycleAudit(audit, trace, registry, options = {}) {
  let expected;
  try {
    expected = buildFontRuleLifecycleAudit(trace, registry, options);
  } catch (error) {
    return [error.message];
  }
  return canonicalJson(audit) === canonicalJson(expected)
    ? []
    : ['font rule lifecycle trace audit differs from the validated registry join'];
}

function readBoundedTrace(file) {
  if (!fs.existsSync(file)) throw new Error('trace input does not exist');
  const stat = fs.lstatSync(file);
  if (!stat.isFile() || stat.isSymbolicLink()) {
    throw new Error('trace input must be a regular non-symlink file');
  }
  if (stat.size > MAX_INPUT_BYTES) {
    throw new Error('trace input exceeds the 16 MiB bound');
  }
  return readJson(file);
}

function main(args) {
  if (args.length !== 2 || args[0] !== '--trace') {
    throw new Error('usage: node scripts/font_rule_lifecycle_audit.mjs --trace <trace-json>');
  }
  const trace = readBoundedTrace(path.resolve(args[1]));
  process.stdout.write(canonicalJson(buildFontRuleLifecycleAudit(trace)));
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
