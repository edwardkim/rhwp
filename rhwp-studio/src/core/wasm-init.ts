type WasmInitScope<T> = {
  __dx_mainPromise?: Promise<T>;
  __rhwpWasmInitPromise?: Promise<T>;
};

/** Reuse dx's eager initialization instead of creating a second WASM instance. */
export function initializeWasmOnce<T>(
  fallback: () => Promise<T>,
  scope: WasmInitScope<T> = globalThis as WasmInitScope<T>,
): Promise<T> {
  const existing = scope.__dx_mainPromise ?? scope.__rhwpWasmInitPromise;
  if (existing) return existing;
  const pending = fallback();
  scope.__rhwpWasmInitPromise = pending;
  void pending.catch(() => {
    if (scope.__rhwpWasmInitPromise === pending) delete scope.__rhwpWasmInitPromise;
  });
  return pending;
}
