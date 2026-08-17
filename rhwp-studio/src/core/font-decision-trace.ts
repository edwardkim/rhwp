import { sha256 } from '@noble/hashes/sha2.js';
import { bytesToHex } from '@noble/hashes/utils.js';

import { getDetectedOSFonts, getWebFontSupplySnapshot, resolveCanvasKitFontPlan } from './font-loader.ts';
import { fontFamilyCandidatesForDisplay } from './font-substitution.ts';
import { getLocalFontState, resolveLocalFont, type LocalFontState } from './local-fonts.ts';

export interface FontDecisionBackendV1 {
  status: string;
  certainty: 'observed' | 'resolved' | 'planned' | 'notObserved' | 'unsupported';
  requested: string | null;
  candidates: string[];
  resolved: string | null;
  source: string | null;
  capabilities: string[];
  failures: string[];
}

export interface FontDecisionTraceRecordV1 {
  recordId: string;
  source: { character: string; [key: string]: unknown };
  document: { face: string | null; altType: number | null; languageSlot: number | null; [key: string]: unknown };
  layoutName: { normalizedFace: string | null; cssFamilyChain: string[]; [key: string]: unknown };
  paint: {
    native: FontDecisionBackendV1;
    canvas2d: FontDecisionBackendV1;
    canvaskit: FontDecisionBackendV1;
  };
  [key: string]: unknown;
}

export interface EmbedFontDecisionTraceV1 {
  schemaVersion: 1;
  status: string;
  records: FontDecisionTraceRecordV1[];
  backendSummary: Record<string, { status: string; reasons: string[] }>;
  reasons: Array<{ code: string; detail: string | null }>;
  layoutHash: { algorithm: 'sha256'; value: string | null };
  normalizedHash: { algorithm: 'sha256'; value: string | null };
  [key: string]: unknown;
}

export interface StudioFontDecisionSnapshot {
  localState?: LocalFontState;
  canvasKitEvidence?: (record: FontDecisionTraceRecordV1) => FontDecisionBackendV1 | null;
}

function unique(values: readonly string[]): string[] {
  return [...new Set(values.filter(Boolean))];
}

function canvas2dDecision(
  record: FontDecisionTraceRecordV1,
  localState: LocalFontState,
): FontDecisionBackendV1 {
  const requested = record.document.face
    ?? record.layoutName.normalizedFace
    ?? record.paint.canvas2d.requested
    ?? '';
  const candidates = fontFamilyCandidatesForDisplay(
    requested,
    record.document.altType ?? 0,
    record.document.languageSlot ?? 0,
  );
  const detectedOs = getDetectedOSFonts();
  let source: string | null = null;
  let supplyStatus: string | null = null;
  for (const candidate of candidates) {
    if (resolveLocalFont(candidate) || detectedOs.has(candidate)) {
      source = 'local';
      supplyStatus = 'available';
      break;
    }
    if (['serif', 'sans-serif', 'monospace'].includes(candidate)) {
      source = 'generic';
      supplyStatus = 'available';
      break;
    }
    const web = getWebFontSupplySnapshot(candidate);
    if (web.status !== 'absent') {
      source = 'web';
      supplyStatus = web.status;
      if (web.status === 'loaded') break;
    }
  }
  const capabilities = ['cssFamilyChainObserved'];
  const failures = ['cssActualGlyphFaceUnobservable'];
  if (!localState.supported) {
    failures.push('localFontApiUnsupported');
  } else if (localState.lastError && /permission|denied|notallowed/i.test(localState.lastError)) {
    failures.push('localFontPermissionDenied');
  } else if (!localState.stored) {
    failures.push('localFontSnapshotUnavailable');
  } else if (!localState.complete) {
    failures.push('localFontEnumerationPartial');
  }
  if (localState.source === 'local-font-access') capabilities.push('localFontAccessEnumerated');
  if (localState.source === 'font-presence-probe') capabilities.push('fontPresenceProbeObserved');
  if (source) capabilities.push(`${source}FontSupply${supplyStatus === 'loaded' ? 'Loaded' : 'Known'}`);
  return {
    status: 'complete',
    certainty: 'notObserved',
    requested: requested || null,
    candidates,
    resolved: null,
    source,
    capabilities: unique(capabilities),
    failures: unique(failures),
  };
}

function canvaskitDecision(
  record: FontDecisionTraceRecordV1,
  snapshot: StudioFontDecisionSnapshot,
): FontDecisionBackendV1 {
  const requested = record.paint.canvaskit.requested
    ?? record.layoutName.normalizedFace
    ?? record.document.face
    ?? '';
  if (snapshot.canvasKitEvidence) {
    const evidence = snapshot.canvasKitEvidence(record);
    if (evidence) return evidence;
    return {
      status: 'notObserved', certainty: 'notObserved', requested: requested || null,
      candidates: [], resolved: null, source: null, capabilities: [],
      failures: ['backendJoinMissing'],
    };
  }
  const local = resolveLocalFont(requested);
  const plan = resolveCanvasKitFontPlan([requested]);
  if (local || plan.sources.length > 0) {
    return {
      status: 'notObserved', certainty: 'planned', requested: requested || null,
      candidates: unique([requested, ...plan.sources.flatMap(source => source.aliases)]),
      resolved: null,
      source: local ? 'local' : 'bundled',
      capabilities: ['canvaskitSfntPlanAvailable'],
      failures: ['canvaskitBackendSnapshotUnavailable'],
    };
  }
  return {
    status: 'unsupported', certainty: 'unsupported', requested: requested || null,
    candidates: requested ? [requested] : [], resolved: null, source: null, capabilities: [],
    failures: ['canvaskitApiUnsupported', 'canvaskitSfntAbsent'],
  };
}

function canonicalize(value: unknown, parentKey?: string, normalized = false): unknown {
  if (Array.isArray(value)) {
    const values = value.map(item => canonicalize(item, parentKey, normalized));
    if (['capabilities', 'failures', 'knownLimitations'].includes(parentKey ?? '')
      && values.every(item => typeof item === 'string')) {
      return unique(values as string[]).sort();
    }
    return values;
  }
  if (typeof value === 'object' && value !== null) {
    const result: Record<string, unknown> = {};
    for (const key of Object.keys(value as Record<string, unknown>).sort()) {
      if (normalized && [
        'layoutHash', 'normalizedHash', 'timestamp', 'generatedAt', 'elapsedMs',
        'durationMs', 'stack',
      ].includes(key)) continue;
      result[key] = canonicalize((value as Record<string, unknown>)[key], key, normalized);
    }
    return result;
  }
  return value;
}

function normalizedDigest(trace: EmbedFontDecisionTraceV1): string {
  const canonical = `${JSON.stringify(canonicalize(trace, undefined, true), null, 2)}\n`;
  return bytesToHex(sha256(new TextEncoder().encode(canonical)));
}

function assertTrace(value: unknown): asserts value is EmbedFontDecisionTraceV1 {
  if (typeof value !== 'object' || value === null
    || (value as { schemaVersion?: unknown }).schemaVersion !== 1
    || !Array.isArray((value as { records?: unknown }).records)) {
    throw new Error('WASM font decision trace must be a schemaVersion 1 envelope');
  }
}

/** WASM layout trace를 현재 Studio snapshot으로만 보강한다. font load/permission/render를 시작하지 않는다. */
export function enrichFontDecisionTrace(
  traceJson: string,
  snapshot: StudioFontDecisionSnapshot = {},
): EmbedFontDecisionTraceV1 {
  const parsed: unknown = JSON.parse(traceJson);
  assertTrace(parsed);
  const trace = structuredClone(parsed);
  const localState = snapshot.localState ?? getLocalFontState();
  for (const record of trace.records) {
    record.paint.canvas2d = canvas2dDecision(record, localState);
    record.paint.canvaskit = canvaskitDecision(record, snapshot);
  }
  const canvasStatuses = trace.records.map(record => record.paint.canvas2d.status);
  const canvaskitStatuses = trace.records.map(record => record.paint.canvaskit.status);
  trace.backendSummary.canvas2d = {
    status: canvasStatuses.every(status => status === 'complete') ? 'complete' : 'notObserved',
    reasons: unique(trace.records.flatMap(record => record.paint.canvas2d.failures)),
  };
  trace.backendSummary.canvaskit = {
    status: canvaskitStatuses.every(status => status === 'complete') ? 'complete' : 'notObserved',
    reasons: unique(trace.records.flatMap(record => record.paint.canvaskit.failures)),
  };
  trace.reasons.push({
    code: 'backendNotObserved',
    detail: 'Canvas2D and CanvasKit evidence was read from current snapshots without loading fonts or rendering.',
  });
  if (trace.records.some(record => record.paint.canvaskit.failures.includes('backendJoinMissing'))) {
    trace.reasons.push({
      code: 'backendJoinMissing',
      detail: 'The current CanvasKit snapshot could not be joined safely to this source record.',
    });
  }
  trace.normalizedHash = { algorithm: 'sha256', value: normalizedDigest(trace) };
  return trace;
}
