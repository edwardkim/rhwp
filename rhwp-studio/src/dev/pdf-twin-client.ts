import {
  PDF_TWIN_LOOKUP_PATH,
  type PdfTwinLookupResponse,
} from './pdf-twin-contract.ts';

export type { PdfTwinFound } from './pdf-twin-contract.ts';

export type PdfTwinLookupResult =
  | PdfTwinLookupResponse
  | { status: 'busy'; retryAfterMs: number }
  | { status: 'error' };

export async function sha256Hex(bytes: Uint8Array): Promise<string> {
  const copied = bytes.slice().buffer;
  const digest = await crypto.subtle.digest('SHA-256', copied);
  return Array.from(new Uint8Array(digest), byte => byte.toString(16).padStart(2, '0')).join('');
}

export async function lookupPdfTwin(
  fileName: string,
  bytes: Uint8Array,
): Promise<PdfTwinLookupResult> {
  const sha256 = await sha256Hex(bytes);
  const response = await fetch(PDF_TWIN_LOOKUP_PATH, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ fileName, size: bytes.byteLength, sha256 }),
  });
  if (response.status === 404) return { status: 'none' };
  if (response.status === 503) {
    const busy = await response.json() as { status?: unknown; retryAfterMs?: unknown };
    return {
      status: 'busy',
      retryAfterMs: typeof busy.retryAfterMs === 'number'
        ? Math.min(30_000, Math.max(100, busy.retryAfterMs))
        : 1_000,
    };
  }
  if (!response.ok) throw new Error(`PDF twin lookup failed (${response.status})`);
  const result = await response.json() as PdfTwinLookupResponse;
  if (
    result.status === 'found'
    || result.status === 'none'
    || result.status === 'ambiguous'
  ) {
    if (
      result.status === 'found'
      && !/^[A-Za-z0-9_-]{43}$/.test(result.errorLogCapability)
    ) throw new Error('PDF twin lookup returned an invalid capability');
    return result;
  }
  throw new Error('PDF twin lookup returned an invalid response');
}
