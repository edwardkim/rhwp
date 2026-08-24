const MAX_REFERENCE_FETCH_ATTEMPTS = 3;

export async function fetchReferenceWithRetry(
  src: string,
  signal: AbortSignal,
  options: {
    send?: typeof fetch;
    wait?: (delayMs: number, signal: AbortSignal) => Promise<void>;
  } = {},
): Promise<Response> {
  const send = options.send ?? fetch;
  const wait = options.wait ?? ((delayMs, waitSignal) => new Promise<void>((resolve, reject) => {
    if (waitSignal.aborted) {
      reject(new DOMException('reference retry aborted', 'AbortError'));
      return;
    }
    const timer = globalThis.setTimeout(done, delayMs);
    const abort = (): void => {
      globalThis.clearTimeout(timer);
      waitSignal.removeEventListener('abort', abort);
      reject(new DOMException('reference retry aborted', 'AbortError'));
    };
    function done(): void {
      waitSignal.removeEventListener('abort', abort);
      resolve();
    }
    waitSignal.addEventListener('abort', abort, { once: true });
  }));
  for (let attempt = 1; attempt <= MAX_REFERENCE_FETCH_ATTEMPTS; attempt++) {
    if (signal.aborted) throw new DOMException('reference image load aborted', 'AbortError');
    const response = await send(src, { signal });
    if (response.status !== 503 || attempt === MAX_REFERENCE_FETCH_ATTEMPTS) return response;
    const seconds = Number(response.headers.get('Retry-After'));
    const delayMs = Number.isFinite(seconds)
      ? Math.min(2_000, Math.max(100, seconds * 1_000))
      : 1_000;
    await wait(delayMs, signal);
  }
  throw new Error('reference retry exhausted');
}
