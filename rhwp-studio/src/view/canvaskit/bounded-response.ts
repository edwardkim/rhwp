export interface BoundedResponseReadOptions {
  maxBytes: number;
  signal?: AbortSignal;
  isCancelled?: () => boolean;
  cancelledMessage?: string;
}

export async function readBoundedResponseArrayBuffer(
  response: Response,
  options: BoundedResponseReadOptions,
): Promise<ArrayBuffer> {
  const { maxBytes, signal, isCancelled } = options;
  if (!Number.isSafeInteger(maxBytes) || maxBytes <= 0) {
    throw new Error(`response byte limit is invalid: ${maxBytes}`);
  }

  const cancelledMessage = options.cancelledMessage ?? 'response read was cancelled';
  const throwIfCancelled = () => {
    if (signal?.aborted || isCancelled?.()) throw new Error(cancelledMessage);
  };
  throwIfCancelled();

  const rawContentLength = response.headers?.get?.('content-length')?.trim() ?? null;
  const declaredLength = rawContentLength !== null && /^\d+$/.test(rawContentLength)
    ? Number(rawContentLength)
    : null;
  if (declaredLength !== null && Number.isSafeInteger(declaredLength) && declaredLength > maxBytes) {
    const error = new Error(`response payload exceeds ${maxBytes} bytes`);
    try {
      await response.body?.cancel(error);
    } catch {
      /* The size error remains authoritative if cancellation also fails. */
    }
    throw error;
  }

  const body = response.body;
  if (!body || typeof body.getReader !== 'function') {
    throw new Error('bounded response body stream is unavailable');
  }

  const reader = body.getReader();
  const chunks: Uint8Array[] = [];
  let totalBytes = 0;
  const cancelReader = () => {
    try {
      void reader.cancel(cancelledMessage).catch(() => {});
    } catch {
      /* A reader may already be closed or released. */
    }
  };
  signal?.addEventListener('abort', cancelReader, { once: true });
  if (signal?.aborted) cancelReader();

  try {
    while (true) {
      throwIfCancelled();
      const { done, value } = await reader.read();
      throwIfCancelled();
      if (done) break;
      if (!(value instanceof Uint8Array)) {
        const error = new Error('response body produced a non-byte chunk');
        try {
          await reader.cancel(error);
        } catch {
          /* Preserve the malformed-body error. */
        }
        throw error;
      }
      if (value.byteLength > maxBytes - totalBytes) {
        const error = new Error(`response payload exceeds ${maxBytes} bytes`);
        try {
          await reader.cancel(error);
        } catch {
          /* Preserve the size error. */
        }
        throw error;
      }
      chunks.push(value);
      totalBytes += value.byteLength;
    }
  } catch (error) {
    throwIfCancelled();
    throw error;
  } finally {
    signal?.removeEventListener('abort', cancelReader);
    reader.releaseLock();
  }

  if (totalBytes === 0) throw new Error('response payload is empty');
  const bytes = new Uint8Array(totalBytes);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return bytes.buffer;
}
