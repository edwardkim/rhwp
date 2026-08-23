import test from 'node:test';
import assert from 'node:assert/strict';

import {
  enrichFontDecisionTrace,
  type EmbedFontDecisionTraceV1,
  type FontDecisionBackendV1,
} from '../src/core/font-decision-trace.ts';
import { resolveCanvasKitFontPlanWithRules } from '../src/core/font-loader.ts';
import {
  FONT_RULE_CANVAS2D_PAINT_RULES,
} from '../src/core/generated/font-rule-projections/canvas2d-paint.ts';
import {
  FONT_RULE_CANVAS2D_WEBFONT_RULES,
} from '../src/core/generated/font-rule-projections/webfont-supply.ts';
import {
  FONT_RULE_CANVASKIT_SFNT_RULES,
} from '../src/core/generated/font-rule-projections/canvaskit-sfnt.ts';
import type { LocalFontState } from '../src/core/local-fonts.ts';
import {
  normalizedTraceHash,
  portableLayoutHash,
  validateTraceEnvelope,
} from '../../scripts/font_decision_trace_contract.mjs';

const unsupported = (requested: string): FontDecisionBackendV1 => ({
  status: 'unsupported', certainty: 'unsupported', requested, candidates: [],
  resolved: null, source: null, capabilities: [], failures: ['studioSnapshotRequired'],
});

function baseTrace(): EmbedFontDecisionTraceV1 {
  const trace: EmbedFontDecisionTraceV1 = {
    schemaVersion: 1,
    status: 'complete',
    scope: {
      pageIndex: 0,
      requestedLimits: { maxCharacters: 8 },
      appliedLimits: { maxCharacters: 8 },
    },
    counts: { runsSeen: 1, charactersSeen: 1, recordsEmitted: 1, recordsOmitted: 0 },
    records: [{
      recordId: 'page:0:run:0:char:0',
      source: {
        status: 'complete', sectionIndex: 0, paragraphIndex: 0, nestedPath: [],
        runIndex: 0, charOffset: 0, character: '가', codePoint: 0xac00,
      },
      document: { face: 'Missing Test Face', altType: 1, languageSlot: 0 },
      layoutName: { normalizedFace: 'Missing Test Face', cssFamilyChain: [] },
      layoutMetric: {},
      paint: {
        native: unsupported('Missing Test Face'),
        canvas2d: unsupported('Missing Test Face'),
        canvaskit: unsupported('Missing Test Face'),
      },
      provenance: [],
      oracle: {},
    }],
    backendSummary: {
      layout: { status: 'complete', reasons: [] },
      native: { status: 'unsupported', reasons: ['nativeSkiaFeatureUnavailable'] },
      canvas2d: { status: 'unsupported', reasons: ['studioSnapshotRequired'] },
      canvaskit: { status: 'unsupported', reasons: ['studioSnapshotRequired'] },
    },
    reasons: [{ code: 'backendUnsupported', detail: 'stage 2' }],
    layoutHash: { algorithm: 'sha256', value: null },
    normalizedHash: { algorithm: 'sha256', value: null },
  };
  trace.layoutHash.value = portableLayoutHash(trace);
  trace.normalizedHash.value = normalizedTraceHash(trace);
  return trace;
}

const deniedLocalState: LocalFontState = {
  supported: true,
  method: 'local-font-access',
  loaded: true,
  stored: false,
  source: null,
  complete: false,
  storage: 'none',
  count: 0,
  checkedFamilies: [],
  probedFamilies: [],
  unresolvedFamilies: [],
  detectedAt: null,
  lastError: 'NotAllowedError: Permission denied',
};

test('Studio trace는 CSS 비관찰, 권한 거부와 CanvasKit resolved를 한 schema에서 분리한다', () => {
  const evidence: FontDecisionBackendV1 = {
    status: 'complete', certainty: 'resolved', requested: 'Missing Test Face',
    candidates: ['Missing Test Face', 'Noto Sans KR'], resolved: 'Noto Sans KR',
    source: 'bundled', capabilities: ['canvaskitSfntPrepared', 'canvaskitGlyphCoverageObserved'],
    failures: [],
  };
  const first = enrichFontDecisionTrace(JSON.stringify(baseTrace()), {
    localState: deniedLocalState,
    canvasKitEvidence: () => evidence,
  });
  const second = enrichFontDecisionTrace(JSON.stringify(baseTrace()), {
    localState: deniedLocalState,
    canvasKitEvidence: () => evidence,
  });
  const record = first.records[0];
  assert.equal(record.paint.canvas2d.certainty, 'notObserved');
  assert.equal(record.paint.canvas2d.resolved, null);
  assert.ok(record.paint.canvas2d.failures.includes('cssActualGlyphFaceUnobservable'));
  assert.ok(record.paint.canvas2d.failures.includes('localFontPermissionDenied'));
  assert.equal(record.paint.canvaskit.resolved, 'Noto Sans KR');
  assert.equal(record.paint.canvaskit.source, 'bundled');
  assert.equal(first.normalizedHash.value, second.normalizedHash.value);
  assert.equal(first.normalizedHash.value, normalizedTraceHash(first));
  assert.deepEqual(validateTraceEnvelope(first), []);
  assert.match(first.normalizedHash.value ?? '', /^[0-9a-f]{64}$/);
});

test('CanvasKit renderer snapshot join 실패와 API 미지원은 SFNT 부재와 구분된다', () => {
  const joinMissing = enrichFontDecisionTrace(JSON.stringify(baseTrace()), {
    localState: { ...deniedLocalState, supported: false, method: null, lastError: null },
    canvasKitEvidence: () => null,
  });
  assert.deepEqual(joinMissing.records[0].paint.canvaskit.failures, ['backendJoinMissing']);
  assert.ok(joinMissing.reasons.some(reason => reason.code === 'backendJoinMissing'));

  const unsupportedApi = enrichFontDecisionTrace(JSON.stringify(baseTrace()), {
    localState: { ...deniedLocalState, supported: false, method: null, lastError: null },
  });
  assert.ok(unsupportedApi.records[0].paint.canvaskit.failures.includes('canvaskitApiUnsupported'));
  assert.ok(unsupportedApi.records[0].paint.canvaskit.failures.includes('canvaskitSfntAbsent'));
  assert.ok(unsupportedApi.records[0].paint.canvas2d.failures.includes('localFontApiUnsupported'));
});

test('CanvasKit evidence callback은 face 문자열이 아닌 source record 전체를 받는다', () => {
  let received: unknown = null;
  enrichFontDecisionTrace(JSON.stringify(baseTrace()), {
    localState: deniedLocalState,
    canvasKitEvidence: record => {
      received = record;
      return null;
    },
  });
  assert.equal((received as { recordId?: unknown }).recordId, 'page:0:run:0:char:0');
  assert.equal(
    (received as { source?: { sectionIndex?: unknown } }).source?.sectionIndex,
    0,
  );
});

test('trace 보강은 font fetch나 Local Font Access 권한 요청을 시작하지 않는다', () => {
  const originalFetch = globalThis.fetch;
  const originalQueryLocalFonts = (globalThis as typeof globalThis & {
    queryLocalFonts?: () => Promise<unknown[]>;
  }).queryLocalFonts;
  let fetchCalls = 0;
  let permissionCalls = 0;
  globalThis.fetch = (() => {
    fetchCalls += 1;
    throw new Error('trace must not fetch');
  }) as typeof fetch;
  (globalThis as typeof globalThis & { queryLocalFonts?: () => Promise<unknown[]> })
    .queryLocalFonts = async () => {
      permissionCalls += 1;
      return [];
    };
  try {
    enrichFontDecisionTrace(JSON.stringify(baseTrace()), {
      localState: { ...deniedLocalState, lastError: null },
    });
    assert.equal(fetchCalls, 0);
    assert.equal(permissionCalls, 0);
  } finally {
    globalThis.fetch = originalFetch;
    (globalThis as typeof globalThis & { queryLocalFonts?: () => Promise<unknown[]> })
      .queryLocalFonts = originalQueryLocalFonts;
  }
});

test('Studio trace ruleId는 실제 Canvas2D와 CanvasKit generated projection에 존재한다', () => {
  const trace = baseTrace();
  trace.records[0].document.face = '휴먼명조';
  trace.records[0].layoutName.normalizedFace = '휴먼명조';
  trace.records[0].paint.canvas2d.requested = '휴먼명조';
  trace.records[0].paint.canvaskit.requested = '휴먼명조';
  trace.layoutHash.value = portableLayoutHash(trace);
  trace.normalizedHash.value = normalizedTraceHash(trace);

  const enriched = enrichFontDecisionTrace(JSON.stringify(trace), {
    localState: deniedLocalState,
    detectedOsFonts: new Set(),
  });
  const record = enriched.records[0];
  const canvasRuleIds = new Set([
    ...FONT_RULE_CANVAS2D_PAINT_RULES.map(rule => rule.ruleId),
    ...FONT_RULE_CANVAS2D_WEBFONT_RULES.map(rule => rule.ruleId),
  ]);
  const canvasKitRuleIds = new Set(FONT_RULE_CANVASKIT_SFNT_RULES.map(rule => rule.ruleId));

  assert.ok((record.paint.canvas2d.ruleIds?.length ?? 0) > 0);
  assert.equal(record.paint.canvas2d.ruleIds?.every(ruleId => canvasRuleIds.has(ruleId)), true);
  assert.deepEqual(
    record.paint.canvaskit.ruleIds,
    resolveCanvasKitFontPlanWithRules(['휴먼명조']).ruleIds,
  );
  assert.equal(record.paint.canvaskit.ruleIds?.every(ruleId => canvasKitRuleIds.has(ruleId)), true);
  assert.deepEqual(validateTraceEnvelope(enriched), []);
});
