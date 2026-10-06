import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import {
  buildContentLossNotice,
  consumeWasmDocumentExport,
  parseContentLossReport,
  persistDownloadWithContentLoss,
  persistWithContentLoss,
  runReportedExport,
  type ContentLossReport,
  type WasmDocumentExport,
} from '../src/core/export-content-loss.ts';

import { assertShowsText, assertDoesNotShowText } from './support/i18n-text.ts';
const fileCommandSource = readFileSync(
  new URL('../src/command/commands/file.ts', import.meta.url),
  'utf8',
);

function between(source: string, start: string, end: string): string {
  const startIndex = source.indexOf(start);
  assert.notEqual(startIndex, -1, `시작 표식이 있어야 합니다: ${start}`);
  const endIndex = source.indexOf(end, startIndex + start.length);
  assert.notEqual(endIndex, -1, `끝 표식이 있어야 합니다: ${end}`);
  return source.slice(startIndex, endIndex);
}

const lossReport: ContentLossReport = {
  schemaVersion: 1,
  outputFormat: 'hwpx',
  count: 1,
  losses: [{
    code: 'binaryContentEmptied',
    subject: 'binaryData',
    path: 'BinData/image7.png',
    reason: 'resourceReadFailedOrLimitExceeded',
    resourceId: 7,
  }],
};

function wasmExport(events: string[]): WasmDocumentExport {
  let bytes: Uint8Array | null = new Uint8Array([4, 4, 3, 0]);
  return {
    hasBytes: () => {
      events.push('hasBytes');
      return bytes !== null;
    },
    takeBytes: () => {
      events.push('takeBytes');
      if (bytes === null) throw new Error('already taken');
      const owned = bytes;
      bytes = null;
      return owned;
    },
    contentLoss: () => {
      events.push('contentLoss');
      return JSON.stringify(lossReport);
    },
    free: () => { events.push('free'); },
  };
}

test('WASM artifact는 바이트를 옮긴 뒤에도 같은 보고서를 읽고 반드시 해제한다', () => {
  const events: string[] = [];
  const artifact = consumeWasmDocumentExport(wasmExport(events));

  assert.deepEqual(artifact.bytes, new Uint8Array([4, 4, 3, 0]));
  assert.deepEqual(artifact.contentLoss, lossReport);
  assert.deepEqual(events, [
    'hasBytes',
    'takeBytes',
    'hasBytes',
    'contentLoss',
    'free',
  ]);
});

test('보고서 파싱 실패는 바이트만 반환하지 않고 WASM artifact도 해제한다', () => {
  const events: string[] = [];
  const exported = wasmExport(events);
  exported.contentLoss = () => {
    events.push('contentLoss');
    return JSON.stringify({ ...lossReport, count: 2 });
  };

  assert.throws(
    () => consumeWasmDocumentExport(exported),
    /보고서 항목이 올바르지 않습니다/,
  );
  assert.equal(events.at(-1), 'free');
});

test('직렬화 실패는 이전 artifact를 재사용하거나 새 보고서를 만들지 않는다', () => {
  const previousEvents: string[] = [];
  const previous = runReportedExport(() => wasmExport(previousEvents));
  let failedArtifact: ReturnType<typeof runReportedExport> | undefined;

  assert.throws(
    () => {
      failedArtifact = runReportedExport(() => {
        throw new Error('serializer failed');
      });
    },
    /serializer failed/,
  );
  assert.equal(failedArtifact, undefined);
  assert.deepEqual(previous.contentLoss, lossReport);
});

test('영속화 성공 뒤에만 내용 손실 경고를 보인다', async () => {
  const events: string[] = [];
  const result = await persistWithContentLoss(
    lossReport,
    async () => { events.push('persisted'); return { method: 'save-picker' }; },
    (saved) => saved.method === 'save-picker',
    () => { events.push('committed'); },
    () => { events.push('notified'); },
  );

  assert.deepEqual(result, { method: 'save-picker' });
  assert.deepEqual(events, ['persisted', 'committed', 'notified']);
});

test('영속화 실패나 fallback 선택에는 stale/new 경고가 없다', async () => {
  const rejectedEvents: string[] = [];
  await assert.rejects(
    persistWithContentLoss(
      lossReport,
      async () => { rejectedEvents.push('persist'); throw new Error('write failed'); },
      () => true,
      () => { rejectedEvents.push('commit'); },
      () => { rejectedEvents.push('notify'); },
    ),
    /write failed/,
  );
  assert.deepEqual(rejectedEvents, ['persist']);

  const fallbackEvents: string[] = [];
  await persistWithContentLoss(
    lossReport,
    async () => { fallbackEvents.push('fallback'); return { method: 'fallback' }; },
    (saved) => saved.method !== 'fallback',
    () => { fallbackEvents.push('commit'); },
    () => { fallbackEvents.push('notify'); },
  );
  assert.deepEqual(fallbackEvents, ['fallback']);
});

test('download 상호작용이 시작된 뒤 영속 경고를 보이며 실패하면 보이지 않는다', () => {
  const events: string[] = [];
  persistDownloadWithContentLoss(
    lossReport,
    () => { events.push('download'); },
    () => { events.push('commit'); },
    () => { events.push('notify'); },
  );
  assert.deepEqual(events, ['download', 'commit', 'notify']);

  const failedEvents: string[] = [];
  assert.throws(
    () => persistDownloadWithContentLoss(
      lossReport,
      () => { failedEvents.push('download'); throw new Error('blocked'); },
      () => { failedEvents.push('commit'); },
      () => { failedEvents.push('notify'); },
    ),
    /blocked/,
  );
  assert.deepEqual(failedEvents, ['download']);
});

test('Studio fallback 뒤 download 성공은 같은 보고서를 정확히 한 번만 알린다', async () => {
  const events: string[] = [];
  let notifications = 0;
  const notify = () => {
    notifications += 1;
    events.push('notify');
  };

  const result = await persistWithContentLoss(
    lossReport,
    async () => { events.push('file-system:fallback'); return { method: 'fallback' as const }; },
    (saved) => saved.method !== 'fallback',
    () => { events.push('file-system:commit'); },
    notify,
  );
  assert.equal(result.method, 'fallback');
  persistDownloadWithContentLoss(
    lossReport,
    () => { events.push('download:click'); },
    () => { events.push('download:commit'); },
    notify,
  );

  assert.deepEqual(events, ['file-system:fallback', 'download:click', 'download:commit', 'notify']);
  assert.equal(notifications, 1);
});

test('내용 손실 알림 실패는 성공한 저장과 상태 commit을 되돌리지 않는다', async () => {
  const events: string[] = [];
  const warnings: string[] = [];
  const previousWarn = console.warn;
  console.warn = (...args: unknown[]) => { warnings.push(args.join(' ')); };
  try {
    const result = await persistWithContentLoss(
      lossReport,
      async () => { events.push('persist'); return 'saved'; },
      () => true,
      () => { events.push('commit'); },
      () => { events.push('notify'); throw new Error('toast failed'); },
    );
    assert.equal(result, 'saved');

    persistDownloadWithContentLoss(
      lossReport,
      () => { events.push('download'); },
      () => { events.push('download-commit'); },
      () => { events.push('download-notify'); throw new Error('toast failed'); },
    );
  } finally {
    console.warn = previousWarn;
  }

  assert.deepEqual(events, [
    'persist',
    'commit',
    'notify',
    'download',
    'download-commit',
    'download-notify',
  ]);
  assert.equal(warnings.length, 2);
});

test('Studio 명시 저장은 reported artifact를 primary 저장 뒤 fallback download까지 전달한다', () => {
  const payloadFactory = between(
    fileCommandSource,
    'function createSavePayload',
    'function showExportContentLoss',
  );
  assert.match(payloadFactory, /exportDocumentWithReportForFormat/);
  assert.match(payloadFactory, /exportPasswordProtectedDocumentWithReportForFormat/);
  assert.doesNotMatch(payloadFactory, /exportDocumentForFormat\s*\(/);
  assert.doesNotMatch(payloadFactory, /exportPasswordProtectedDocumentForFormat\s*\(/);

  const explicitSaveBodies = [
    between(fileCommandSource, 'async function saveAsFormat', 'function reportSaveError'),
    between(
      fileCommandSource,
      'export async function saveCurrentDocument',
      'async function fallbackNameForCurrentSave',
    ),
  ];
  for (const body of explicitSaveBodies) {
    const artifactIndex = body.indexOf('createSavePayload(');
    const primaryIndex = body.indexOf('persistWithContentLoss(');
    const downloadIndex = body.indexOf('persistDownloadWithContentLoss(');
    assert.ok(artifactIndex >= 0, 'reported artifact를 만들어야 합니다');
    assert.ok(primaryIndex > artifactIndex, 'reported artifact를 primary 저장에 전달해야 합니다');
    assert.ok(downloadIndex > primaryIndex, 'primary fallback 뒤 같은 artifact로 download해야 합니다');
    assert.match(body, /saveResult\.method !== 'fallback'/);
    assert.doesNotMatch(body, /exportHwp(?:x)?\s*\(/, '명시 저장이 byte-only WASM API를 우회 호출하면 안 됩니다');
  }

  const notice = between(
    fileCommandSource,
    'function showExportContentLoss',
    'function requirePasswordSaveFormat',
  );
  assert.match(notice, /durationMs:\s*0/);
  assertShowsText(notice, '확인');
});

test('알림은 출력 형식과 정확한 손실 위치를 사용자 상호작용에 남긴다', () => {
  const message = buildContentLossNotice(parseContentLossReport(lossReport));
  assert.match(message ?? '', /HWPX 파일은 저장되었지만/);
  assert.match(message ?? '', /그림·첨부 데이터 #7: BinData\/image7\.png/);
});
