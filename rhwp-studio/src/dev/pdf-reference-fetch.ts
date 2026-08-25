const MAX_REFERENCE_FETCH_ATTEMPTS = 3;

function waitForRetry(delay: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    const abort = (): void => {
      clearTimeout(timer);
      signal.removeEventListener('abort', abort);
      reject(new DOMException('reference retry aborted', 'AbortError'));
    };
    const done = (): void => {
      signal.removeEventListener('abort', abort);
      resolve();
    };
    const timer = setTimeout(done, delay);
    signal.addEventListener('abort', abort, { once: true });
    if (signal.aborted) abort();
  });
}

export async function fetchReferenceWithRetry(
  src: string,
  signal: AbortSignal,
  options: {
    send?: typeof fetch;
    wait?: (delayMs: number, signal: AbortSignal) => Promise<void>;
  } = {},
): Promise<Response> {
  const send = options.send ?? fetch;
  const wait = options.wait ?? waitForRetry;
  for (let attempt = 1; attempt <= MAX_REFERENCE_FETCH_ATTEMPTS; attempt++) {
    if (signal.aborted) throw new DOMException('reference image load aborted', 'AbortError');
    const response = await send(src, { signal });
    if (response.status !== 503 || attempt === MAX_REFERENCE_FETCH_ATTEMPTS) return response;
    const retryAfter = Number(response.headers.get('Retry-After'));
    const delay = Number.isFinite(retryAfter)
      ? Math.min(2_000, Math.max(100, retryAfter * 1_000))
      : 1_000;
    await wait(delay, signal);
  }
  throw new Error('reference retry exhausted');
}
