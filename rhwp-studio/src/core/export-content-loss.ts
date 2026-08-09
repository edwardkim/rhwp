export const CONTENT_LOSS_SCHEMA_VERSION = 1 as const;

export type SerializedOutputFormat = 'hwp' | 'hwpx';
export type ContentLossCode =
  | 'binaryContentEmptied'
  | 'controlOmitted'
  | 'metadataReduced';
export type ContentLossSubject = 'binaryData' | 'control' | 'fieldParameters';
export type ContentLossReason =
  | 'resourceReadFailedOrLimitExceeded'
  | 'storedCompressionMismatch'
  | 'rawPassthroughUnavailable'
  | 'unsupportedByOutputFormat';

export interface ContentLossRecord {
  code: ContentLossCode;
  subject: ContentLossSubject;
  path: string;
  reason: ContentLossReason;
  resourceId?: number;
}

export interface ContentLossReport {
  schemaVersion: typeof CONTENT_LOSS_SCHEMA_VERSION;
  outputFormat: SerializedOutputFormat;
  count: number;
  losses: ContentLossRecord[];
}

export interface DocumentExportArtifact {
  bytes: Uint8Array;
  contentLoss: ContentLossReport;
}

/** wasm-bindgen이 내보내는 transaction-owned 결과의 최소 표면. */
export interface WasmDocumentExport {
  contentLoss(): string;
  hasBytes(): boolean;
  takeBytes(): Uint8Array;
  free(): void;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function isContentLossCode(value: unknown): value is ContentLossCode {
  return value === 'binaryContentEmptied'
    || value === 'controlOmitted'
    || value === 'metadataReduced';
}

function isContentLossSubject(value: unknown): value is ContentLossSubject {
  return value === 'binaryData' || value === 'control' || value === 'fieldParameters';
}

function isContentLossReason(value: unknown): value is ContentLossReason {
  return value === 'resourceReadFailedOrLimitExceeded'
    || value === 'storedCompressionMismatch'
    || value === 'rawPassthroughUnavailable'
    || value === 'unsupportedByOutputFormat';
}

function parseLoss(value: unknown): ContentLossRecord | null {
  if (!isRecord(value)
    || !isContentLossCode(value.code)
    || !isContentLossSubject(value.subject)
    || typeof value.path !== 'string'
    || !isContentLossReason(value.reason)) return null;
  if (value.resourceId !== undefined
    && (typeof value.resourceId !== 'number'
      || !Number.isSafeInteger(value.resourceId)
      || value.resourceId < 0)) return null;
  return {
    code: value.code,
    subject: value.subject,
    path: value.path,
    reason: value.reason,
    ...(value.resourceId === undefined ? {} : { resourceId: value.resourceId as number }),
  };
}

/** 알 수 없거나 오래된 보고서 모양은 조용히 버리지 않고 저장 자체를 실패시킨다. */
export function parseContentLossReport(value: unknown): ContentLossReport {
  if (!isRecord(value)
    || value.schemaVersion !== CONTENT_LOSS_SCHEMA_VERSION
    || (value.outputFormat !== 'hwp' && value.outputFormat !== 'hwpx')
    || typeof value.count !== 'number'
    || !Number.isSafeInteger(value.count)
    || value.count < 0
    || !Array.isArray(value.losses)) {
    throw new Error('내보내기 내용 손실 보고서 형식을 확인할 수 없습니다');
  }

  const losses = value.losses.map(parseLoss);
  if (losses.some((loss) => loss === null) || value.count !== losses.length) {
    throw new Error('내보내기 내용 손실 보고서 항목이 올바르지 않습니다');
  }

  return {
    schemaVersion: CONTENT_LOSS_SCHEMA_VERSION,
    outputFormat: value.outputFormat,
    count: value.count as number,
    losses: losses as ContentLossRecord[],
  };
}

/**
 * WASM 결과에서 바이트와 보고서를 한 번에 꺼낸다.
 *
 * 보고서는 `takeBytes()` 뒤에도 유효하다는 Rust 계약을 실제 호출 순서로 검증한다.
 * 어느 단계든 실패하면 artifact를 반환하지 않으며 WASM 소유권은 항상 해제한다.
 */
export function consumeWasmDocumentExport(exported: WasmDocumentExport): DocumentExportArtifact {
  try {
    if (!exported.hasBytes()) throw new Error('내보내기 결과에 바이트가 없습니다');
    const bytes = exported.takeBytes();
    if (exported.hasBytes()) throw new Error('내보내기 바이트 소유권을 옮기지 못했습니다');
    const contentLoss = parseContentLossReport(JSON.parse(exported.contentLoss()));
    return { bytes, contentLoss };
  } finally {
    exported.free();
  }
}

export function runReportedExport(exporter: () => WasmDocumentExport): DocumentExportArtifact {
  return consumeWasmDocumentExport(exporter());
}

function subjectLabel(loss: ContentLossRecord): string {
  switch (loss.subject) {
    case 'binaryData': return '그림·첨부 데이터';
    case 'control': return '문서 개체';
    case 'fieldParameters': return '필드 속성';
    default: return '문서 내용';
  }
}

/** 저장 완료 뒤 사용자에게 보여줄 영속 알림 문구. */
export function buildContentLossNotice(report: ContentLossReport): string | null {
  if (report.losses.length === 0) return null;
  const format = report.outputFormat.toUpperCase();
  const shown = report.losses.slice(0, 3).map((loss) => {
    const resource = loss.resourceId === undefined ? '' : ` #${loss.resourceId}`;
    return `• ${subjectLabel(loss)}${resource}: ${loss.path}`;
  });
  const remaining = report.losses.length - shown.length;
  if (remaining > 0) shown.push(`• 그 밖의 손실 ${remaining}건`);
  return [
    `${format} 파일은 저장되었지만 일부 내용을 보존하지 못했습니다.`,
    ...shown,
  ].join('\n');
}

/**
 * 영속화가 성공한 뒤에만 손실 알림을 전달한다.
 *
 * `wasPersisted`가 false인 fallback 선택 단계나 `persist` 예외에서는 알림이 없다.
 */
export async function persistWithContentLoss<T>(
  report: ContentLossReport | null,
  persist: () => Promise<T>,
  wasPersisted: (result: T) => boolean,
  notify: (report: ContentLossReport) => void,
): Promise<T> {
  const result = await persist();
  if (report && report.losses.length > 0 && wasPersisted(result)) notify(report);
  return result;
}

/** 동기 download 시작이 성공한 뒤 같은 규칙으로 알린다. */
export function persistDownloadWithContentLoss(
  report: ContentLossReport | null,
  download: () => void,
  notify: (report: ContentLossReport) => void,
): void {
  download();
  if (report && report.losses.length > 0) notify(report);
}
