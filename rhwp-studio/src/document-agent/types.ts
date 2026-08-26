export interface RhwpBodyParagraphTargetV1 {
  kind: 'body_paragraph';
  section: number;
  paragraph: number;
  charOffset: 0;
  length: number;
}

export interface RhwpDocumentStateV1 {
  schemaVersion: 1;
  format: 'hwp' | 'hwpx';
  documentEpoch: number;
  changeSeq: number;
  dirty: boolean;
  pageCount: number;
  documentSha256: string;
}

export interface RhwpSelectionContextV1 {
  schemaVersion: 1;
  documentEpoch: number;
  changeSeq: number;
  page: number;
  editable: boolean;
  collapsed: boolean;
  target: RhwpBodyParagraphTargetV1 | null;
  selectedTextSha256: string | null;
}

export interface RhwpApplyTextCommandV1 {
  schemaVersion: 1;
  commandId: string;
  expectedDocumentEpoch: number;
  expectedChangeSeq: number;
  expectedDocumentSha256: string;
  target: RhwpBodyParagraphTargetV1;
  expectedBeforeSha256: string;
  expectedFormatSha256: string;
  expectedAdjacentContextSha256: string;
  replacement: string;
}

export interface RhwpRevertTextCommandV1 {
  schemaVersion: 1;
  commandId: string;
  expectedDocumentEpoch: number;
  expectedChangeSeq: number;
  expectedAfterDocumentSha256: string;
  expectedAfterSha256: string;
}

export interface RhwpTextCommandReceiptV1 {
  schemaVersion: 1;
  commandId: string;
  operation: 'apply' | 'revert';
  documentEpoch: number;
  beforeChangeSeq: number;
  afterChangeSeq: number;
  beforeDocumentSha256: string;
  afterDocumentSha256: string;
  beforeTextSha256: string;
  afterTextSha256: string;
  formatSha256: string;
  adjacentContextSha256: string;
  pageCountBefore: number;
  pageCountAfter: number;
  target: RhwpBodyParagraphTargetV1;
}

export const DOCUMENT_AGENT_ERROR_CODES = [
  'CAPABILITY_UNSUPPORTED',
  'INVALID_COMMAND',
  'DOCUMENT_EPOCH_MISMATCH',
  'CHANGE_SEQ_MISMATCH',
  'DOCUMENT_SHA_MISMATCH',
  'TARGET_NOT_FOUND',
  'TARGET_PREIMAGE_MISMATCH',
  'TARGET_FORMAT_MISMATCH',
  'TARGET_CONTEXT_MISMATCH',
  'PAGE_COUNT_CHANGED',
  'NON_TARGET_CHANGED',
  'COMMAND_NOT_LATEST',
  'COMMAND_REPLAY_MISMATCH',
  'COMMAND_TOO_SLOW',
  'TRANSACTION_FAILED',
  'RENDER_FAILED',
] as const;

export type DocumentAgentErrorCode = typeof DOCUMENT_AGENT_ERROR_CODES[number];

export class DocumentAgentError extends Error {
  readonly code: DocumentAgentErrorCode;
  readonly recovered?: boolean;

  constructor(code: DocumentAgentErrorCode, message: string, recovered?: boolean) {
    super(message);
    this.name = 'DocumentAgentError';
    this.code = code;
    if (recovered !== undefined) this.recovered = recovered;
  }
}

export function isDocumentAgentError(value: unknown): value is DocumentAgentError {
  return value instanceof Error && DOCUMENT_AGENT_ERROR_CODES.includes(
    (value as { code?: DocumentAgentErrorCode }).code as DocumentAgentErrorCode,
  );
}
