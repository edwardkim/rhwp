const MAX_ATTEMPTS = 3;

function wait(ms: number, signal?: AbortSignal | null): Promise<void> {
  if (signal?.aborted) return Promise.reject(new DOMException('aborted', 'AbortError'));
  return new Promise((resolve, reject) => {
    const timer = setTimeout(done, ms);
    const abort = () => done(new DOMException('aborted', 'AbortError'));
    signal?.addEventListener('abort', abort, { once: true });
    function done(error?: DOMException): void {
      clearTimeout(timer);
      signal?.removeEventListener('abort', abort);
      error ? reject(error) : resolve();
    }
  });
}

export async function fetchWithBusyRetry(
  input: RequestInfo | URL,
  init: RequestInit = {},
  send: typeof fetch = fetch,
): Promise<Response> {
  for (let attempt = 1; ; attempt += 1) {
    if (init.signal?.aborted) throw new DOMException('aborted', 'AbortError');
    const response = await send(input, init);
    if (response.status !== 503 || attempt === MAX_ATTEMPTS) return response;
    const seconds = Number(response.headers.get('Retry-After'));
    const delay = Number.isFinite(seconds) ? Math.min(2_000, Math.max(0, seconds * 1_000)) : 250;
    await response.body?.cancel();
    await wait(delay, init.signal);
  }
}
