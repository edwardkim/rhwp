import { blake3 } from '@noble/hashes/blake3.js';
import { bytesToHex } from '@noble/hashes/utils.js';
import { PdfImportSession, type PdfImportOptions, type PdfPageProgress } from '../../../rhwp-shared/pdf/import-session.js';

export interface PdfImportPolicy { backend: { inputBytes: number }; [key: string]: unknown }

export interface PdfInputSource {
  byteLength: number;
  read(): Promise<Uint8Array>;
}

export interface PreparedPdfDocument {
  input: Uint8Array;
  editable: Uint8Array;
  diagnostics: Record<string, unknown>;
  pageCount: number;
  deadline: number;
  reopen: Readonly<{ inputBytes: number; decodedBytes: number; options: string }>;
}

export function isPdfInput(data: Uint8Array, fileName: string): boolean {
  return /\.pdf$/i.test(fileName)
    || new TextDecoder('ascii').decode(data.subarray(0, 1024)).includes('%PDF-');
}

/** One deadline covers initialization, conversion and bounded serialization. */
export async function preparePdfForEditor(
  source: PdfInputSource,
  options: PdfImportOptions,
  policy: { backend: { inputBytes: number } },
  request: { signal: AbortSignal; timeoutMs: number; onProgress?: (phase: string, progress?: PdfPageProgress) => void },
  session = new PdfImportSession(),
): Promise<PreparedPdfDocument> {
  const deadline = performance.now() + request.timeoutMs;
  const remaining = () => {
    if (request.signal.aborted) throw Object.assign(new Error('PDF_IMPORT_CANCELLED'), { code: 'PDF_IMPORT_CANCELLED' });
    const timeoutMs = Math.ceil(deadline - performance.now());
    if (timeoutMs <= 0) throw Object.assign(new Error('PDF_IMPORT_TIMEOUT'), { code: 'PDF_IMPORT_TIMEOUT' });
    return { signal: request.signal, timeoutMs };
  };
  try {
    remaining();
    if (!Number.isSafeInteger(source.byteLength) || source.byteLength < 0
      || source.byteLength > policy.backend.inputBytes) {
      throw new Error(`PDF_RESOURCE_LIMIT: 입력 한도 ${policy.backend.inputBytes} bytes를 초과했습니다.`);
    }
    // The underlying supplier may finish later; its result cannot outlive this request.
    const input = await new Promise<Uint8Array>((resolve, reject) => {
      const cleanup = () => { clearTimeout(timer); request.signal.removeEventListener('abort', cancel); };
      const fail = (code: string) => { cleanup(); reject(Object.assign(new Error(code), { code })); };
      const cancel = () => fail('PDF_IMPORT_CANCELLED');
      const timer = setTimeout(() => fail('PDF_IMPORT_TIMEOUT'), remaining().timeoutMs);
      request.signal.addEventListener('abort', cancel, { once: true });
      Promise.resolve().then(() => { remaining(); return source.read(); }).then(
        value => { cleanup(); resolve(value); }, error => { cleanup(); reject(error); });
    });
    remaining();
    if (!(input instanceof Uint8Array) || input.byteLength !== source.byteLength) throw new Error('PDF_INVALID_INPUT');
    const result = await session.importPdf(input, options, { ...remaining(), onProgress: request.onProgress });
    request.onProgress?.('serializing');
    const editable = await session.exportDocument('hwpx', remaining());
    remaining();
    const reopen = Object.freeze(session.getEditorReopenLimits());
    if (!(editable instanceof Uint8Array) || editable.byteLength > reopen.inputBytes) throw new Error('PDF_WORKER_PROTOCOL');
    return { input, editable, ...result, deadline, reopen };
  } finally {
    // Studio owns a separate, fully editable document after successful handoff.
    // Failed candidates and exported Worker instances are always retired here.
    session.dispose();
  }
}

/** A candidate owns all resources until the single synchronous publication. */
export async function publishPreparedPdf<T>(
  result: PreparedPdfDocument, signal: AbortSignal, isCurrent: () => boolean,
  lifecycle: { create(timeoutMs: number): T; prepare(candidate: T): void;
    publish(candidate: T, digest: string): void; destroy(candidate: T): void },
): Promise<void> {
  const check = () => {
    const code = signal.aborted ? 'PDF_IMPORT_CANCELLED' : !isCurrent() ? 'PDF_IMPORT_SUPERSEDED'
      : performance.now() >= result.deadline ? 'PDF_IMPORT_TIMEOUT' : null;
    if (code) throw Object.assign(new Error(code), { code });
  };
  const yieldOwner = () => new Promise<void>(resolve => setTimeout(resolve, 0));
  let candidate: T | undefined, owned = false;
  try {
    check();
    candidate = lifecycle.create(result.deadline - performance.now()); owned = true;
    check(); lifecycle.prepare(candidate); check();
    const hash = blake3.create();
    for (let offset = 0; offset < result.input.byteLength; offset += 4 * 1024 * 1024) {
      check(); hash.update(result.input.subarray(offset, offset + 4 * 1024 * 1024));
      if (offset > 0) { await yieldOwner(); check(); }
    }
    const digest = `blake3:${bytesToHex(hash.digest())}`;
    // Queued cancel/timeout/another open must be observed after synchronous WASM.
    await yieldOwner(); check();
    lifecycle.publish(candidate, digest); owned = false;
  } finally {
    if (owned) lifecycle.destroy(candidate!);
  }
}
