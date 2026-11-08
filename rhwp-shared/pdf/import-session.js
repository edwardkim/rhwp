/** Opt-in browser PDF session. The caller owns this instance and must dispose it. */
export class PdfImportSession {
  constructor({ workerFactory = () => new Worker(new URL('./import-worker.js', import.meta.url), { type: 'module' }) } = {}) {
    this._workerFactory = workerFactory;
    this._active = null;
    this._candidate = null;
    this._importing = false;
    this._disposed = false;
  }

  async importPdf(bytes, options = {}, { signal, timeoutMs = 30_000, onProgress } = {}) {
    this._assertAlive();
    if (this._importing) throw failure('PDF_IMPORT_BUSY', 'another import is in progress');
    if (!(bytes instanceof Uint8Array)) throw failure('PDF_INVALID_INPUT', 'expected Uint8Array');
    if (!Number.isSafeInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > 2_147_483_647) {
      throw failure('PDF_DEADLINE_INVALID', 'timeoutMs must be a positive timer interval');
    }
    if (signal?.aborted) throw failure('PDF_IMPORT_CANCELLED', 'request cancelled');
    if (onProgress !== undefined && typeof onProgress !== 'function') throw failure('PDF_IMPORT_OPTIONS', 'onProgress must be a function');
    const inputLength = bytes.byteLength;
    const snapshot = snapshotOptions(options);
    this._importing = true;
    try {
      const started = performance.now();
      const candidate = this._candidate ??= new Channel(this._workerFactory());
      const requestOptions = () => {
        this._assertAlive();
        if (signal?.aborted) {
          const error = failure('PDF_IMPORT_CANCELLED', 'request cancelled');
          candidate.close(error);
          throw error;
        }
        const remaining = Math.ceil(timeoutMs - (performance.now() - started));
        if (remaining <= 0) {
          const error = failure('PDF_IMPORT_TIMEOUT', 'worker deadline exceeded');
          candidate.close(error);
          throw error;
        }
        return { signal, timeoutMs: remaining, onProgress };
      };
      // Ask the initialized product before copying or structured-cloning input.
      // Policy and import share the same candidate and one overall deadline.
      const policy = validatePolicy(
        await candidate.request('policy', { options: snapshot }, requestOptions()),
      );
      if (inputLength > policy.backend.inputBytes) {
        throw failure('PDF_RESOURCE_LIMIT', `input exceeds ${policy.backend.inputBytes} bytes`);
      }
      const checkView = () => { if (bytes.byteLength !== inputLength) throw failure('PDF_INVALID_INPUT', 'input view changed during admission'); };
      checkView();
      await candidate.request('reserveInput', { byteLength: inputLength }, requestOptions());
      checkView();
      // Copy only the selected view, after the product reserves its backing.
      const privateBytes = new Uint8Array(inputLength);
      privateBytes.set(bytes);
      const result = await candidate.request('importPdf', { bytes: privateBytes, options: snapshot }, requestOptions());
      // No await between successful response and publication. Failed/cancelled candidates
      // never become the active session, even when synchronous WASM had already completed.
      requestOptions();
      this._active?.close(failure('PDF_SESSION_REPLACED', 'document replaced'));
      this._active = candidate;
      candidate.options = snapshot;
      this._candidate = null;
      return result;
    } catch (error) {
      if (error instanceof RangeError) {
        const failureError = failure('PDF_WORKER_MEMORY', 'input copy allocation failed');
        this._candidate?.close(failureError);
        throw failureError;
      }
      if (error?.code === 'PDF_WORKER_PROTOCOL') this._candidate?.close(error);
      throw error;
    } finally {
      // Do not retain an opening credential in editor reopen options or sessions.
      delete snapshot.password;
      if (this._candidate?.closed) this._candidate = null;
      this._importing = false;
    }
  }

  inspect(options) { return this._request('inspect', {}, options); }
  renderPageSvg(page, { outputBytes, ...options } = {}) {
    return this._limitedRequest('render', { page }, outputBytes, options);
  }
  exportDocument(format = 'hwpx', { outputBytes, ...options } = {}) {
    return this._limitedRequest('export', { format }, outputBytes, options);
  }

  _limitedRequest(method, params, maximumBytes, options) {
    if (maximumBytes !== undefined && (!Number.isSafeInteger(maximumBytes) || maximumBytes <= 0
        || (this._active?.limits && maximumBytes > this._active.limits.outputBytes))) {
      return Promise.reject(failure('PDF_RESOURCE_POLICY', 'invalid output byte limit'));
    }
    return this._request(method, { ...params, maximumBytes }, options);
  }

  getEditorReopenLimits() {
    this._assertAlive();
    const active = this._active;
    if (!active || active.closed) throw failure('PDF_NO_DOCUMENT', 'import a PDF first');
    if (!Number.isSafeInteger(active.remainingDecodedBytes) || active.remainingDecodedBytes <= 0) {
      throw failure('PDF_RESOURCE_LIMIT', 'no remaining editor reopen budget');
    }
    return { inputBytes: active.limits.outputBytes, decodedBytes: active.remainingDecodedBytes,
      options: JSON.stringify(active.options) };
  }

  dispose() {
    if (this._disposed) return;
    this._disposed = true;
    const error = failure('PDF_SESSION_DISPOSED', 'session disposed');
    this._candidate?.close(error);
    this._active?.close(error);
    this._candidate = this._active = null;
  }

  _assertAlive() {
    if (this._disposed) throw failure('PDF_SESSION_DISPOSED', 'session disposed');
  }

  async _request(method, params, options) {
    this._assertAlive();
    if (!this._active || this._active.closed) throw failure('PDF_NO_DOCUMENT', 'import a PDF first');
    return this._active.request(method, params, options);
  }
}

function failure(code, message) {
  return Object.assign(new Error(`${code}: ${message}`), { code });
}

// Capture caller-owned options once, before the first await. Otherwise policy
// lookup and input transfer could use different limits after caller mutation.
function snapshotOptions(options) {
  const record = (value, code, maximum) => {
    const proto = value && typeof value === 'object' ? Object.getPrototypeOf(value) : undefined;
    if (proto !== Object.prototype && proto !== null) throw failure(code, 'expected a plain options object');
    const keys = Reflect.ownKeys(value);
    if (keys.length > maximum) throw failure(code, 'too many option fields');
    const result = {};
    for (const key of keys) {
      const descriptor = Object.getOwnPropertyDescriptor(value, key);
      if (typeof key !== 'string' || key.length > 128 || !descriptor?.enumerable
          || !('value' in descriptor)) {
        throw failure(code, 'expected own enumerable data properties');
      }
      Object.defineProperty(result, key, {
        value: descriptor.value, enumerable: true, configurable: true, writable: true,
      });
    }
    return result;
  };
  const result = record(options, 'PDF_IMPORT_OPTIONS', 6);
  for (const key of Object.keys(result)) {
    if (!['sourcePages', 'includeTableInference', 'layout', 'resourceProfile', 'resourceLimits', 'password'].includes(key)) {
      throw failure('PDF_IMPORT_OPTIONS', 'unknown option field');
    }
  }
  if (result.password !== undefined && (typeof result.password !== 'string'
      || result.password.length > 1024 || new TextEncoder().encode(result.password).length > 1024)) {
    throw failure('PDF_IMPORT_OPTIONS', 'password must be a string of at most 1024 UTF-8 bytes');
  }
  if (result.resourceProfile !== undefined && !['standard', 'constrained'].includes(result.resourceProfile)) {
    throw failure('PDF_RESOURCE_POLICY', 'unknown resource profile');
  }
  if (result.layout !== undefined && !['fidelity', 'editable'].includes(result.layout)) {
    throw failure('PDF_IMPORT_OPTIONS', 'unknown import layout');
  }
  if (result.includeTableInference !== undefined && typeof result.includeTableInference !== 'boolean') {
    throw failure('PDF_IMPORT_OPTIONS', 'includeTableInference must be a boolean');
  }
  if (result.sourcePages !== undefined) {
    const pages = result.sourcePages;
    // Both bundled product profiles allow at most 1024 selection entries. This
    // is a bound on option metadata before the first await, not an input-byte
    // policy; the candidate still resolves each request's actual lower limits.
    if (!Array.isArray(pages) || !pages.length || pages.length > 1024) {
      throw failure('PDF_PAGE_SELECTION', 'invalid selection metadata');
    }
    const selected = [];
    for (let i = 0; i < pages.length; i++) {
      const descriptor = Object.getOwnPropertyDescriptor(pages, String(i));
      const page = descriptor?.value;
      if (!descriptor || !('value' in descriptor) || !Number.isSafeInteger(page)
          || page < 0 || (i > 0 && selected[i - 1] >= page)) {
        throw failure('PDF_PAGE_SELECTION', 'invalid selection entry');
      }
      selected.push(page);
    }
    result.sourcePages = selected;
  }
  const limits = (value, root) => {
    const copy = record(value, 'PDF_RESOURCE_POLICY', 64);
    for (const key of Object.keys(copy)) {
      if (root && key === 'backend') copy[key] = limits(copy[key], false);
      else if (!Number.isSafeInteger(copy[key]) || copy[key] <= 0) {
        throw failure('PDF_RESOURCE_POLICY', 'limits must be positive safe integers');
      }
    }
    return copy;
  };
  if (result.resourceLimits !== undefined) result.resourceLimits = limits(result.resourceLimits, true);
  return result;
}

function validatePolicy(policy) {
  if (policy?.schemaVersion !== 1 || !Number.isSafeInteger(policy.backend?.inputBytes)
      || policy.backend.inputBytes <= 0 || !Number.isSafeInteger(policy.backend.decodedBytes)
      || policy.backend.decodedBytes <= 0) {
    throw failure('PDF_WORKER_PROTOCOL', 'invalid product import policy');
  }
  return policy;
}

class Channel {
  constructor(worker) {
    this.worker = worker;
    this.closed = false;
    this.pending = new Map();
    this.nextId = 0;
    this.limits = null;
    worker.onmessage = ({ data }) => {
      const request = this.pending.get(data?.id);
      if (!request) return;
      if (request.signal?.aborted || performance.now() >= request.deadline) {
        this.close(failure(request.signal?.aborted ? 'PDF_IMPORT_CANCELLED' : 'PDF_IMPORT_TIMEOUT', 'request completed after cancellation/deadline'));
        return;
      }
      if (data.phase !== undefined) {
        if (!['policy', 'importPdf'].includes(request.method) || !['initializing', 'importing', 'ready'].includes(data.phase)) {
          this.close(failure('PDF_WORKER_PROTOCOL', 'unknown import progress phase'));
          return;
        }
        try {
          let progress;
          if (data.progress !== undefined) {
            const p = data.progress, previous = request.progress;
            if (request.method !== 'importPdf' || data.phase !== 'importing' || !p
                || ![p.sourcePage, p.sourcePageCount, p.completedPages, p.totalPages].every(Number.isSafeInteger)
                || p.sourcePage < 1 || p.sourcePage > p.sourcePageCount
                || p.totalPages < 1 || p.totalPages > p.sourcePageCount
                || p.completedPages < 0 || p.completedPages > p.totalPages
                || (previous && (p.totalPages !== previous.totalPages
                  || p.sourcePageCount !== previous.sourcePageCount
                  || p.completedPages < previous.completedPages || p.sourcePage < previous.sourcePage))) {
              throw failure('PDF_WORKER_PROTOCOL', 'invalid page progress');
            }
            progress = Object.freeze({ sourcePage: p.sourcePage, sourcePageCount: p.sourcePageCount,
              completedPages: p.completedPages, totalPages: p.totalPages });
            request.progress = progress;
          }
          request.onProgress?.(data.phase, progress);
        } catch { this.close(failure('PDF_WORKER_PROTOCOL', 'invalid page progress or callback failure')); }
        return;
      }
      try {
        if (data.error) {
          if (typeof data.error.code !== 'string' || !/^PDF_[A-Z_]+$/.test(data.error.code)
              || typeof data.error.message !== 'string' || data.error.message.length > 8192) {
            throw failure('PDF_WORKER_PROTOCOL', 'invalid worker error');
          }
          const error = failure(data.error.code, data.error.message);
          if (data.error.fatal) this.close(error); else request.finish(error);
        } else {
          const result = this._decode(request, data);
          if (performance.now() >= request.deadline) throw failure('PDF_IMPORT_TIMEOUT', 'response decoding exceeded deadline');
          request.finish(null, result);
        }
      } catch (error) {
        this.close(error?.code ? error : failure('PDF_WORKER_PROTOCOL', 'invalid worker response'));
      }
    };
    worker.onerror = () => this.close(failure('PDF_WORKER_FAILED', 'worker execution failed'));
    worker.onmessageerror = () => this.close(failure('PDF_WORKER_FAILED', 'worker response could not be decoded'));
  }

  _decode(request, data) {
    const invalid = () => failure('PDF_WORKER_PROTOCOL', 'invalid or excessive worker response');
    if (request.method === 'policy') {
      const limits = data.limits;
      if (!limits || !['outputBytes', 'diagnosticsBytes'].every(key => Number.isSafeInteger(limits[key])
          && limits[key] > 0 && limits[key] <= 0xffffffff)) throw invalid();
      this.limits = Object.freeze({ outputBytes: limits.outputBytes, diagnosticsBytes: limits.diagnosticsBytes });
      this.policy = validatePolicy(data.result);
    }
    if (!Number.isSafeInteger(data.remainingDecodedBytes) || data.remainingDecodedBytes < 0
        || data.remainingDecodedBytes > this.policy.backend.decodedBytes) throw invalid();
    this.remainingDecodedBytes = data.remainingDecodedBytes;
    if (request.method === 'policy') {
      return this.policy;
    }
    if (!this.limits) throw invalid();
    if (request.method === 'reserveInput') {
      if (data.result !== true) throw invalid();
      return true;
    }
    if (request.method === 'importPdf' || request.method === 'inspect') {
      if (data.encoding !== 'json' || !withinUtf8Limit(data.result, this.limits.diagnosticsBytes)) throw invalid();
      const result = JSON.parse(data.result);
      if (!Number.isSafeInteger(result?.pageCount) || result.pageCount < 1
          || !result.diagnostics || typeof result.diagnostics !== 'object' || Array.isArray(result.diagnostics)) throw invalid();
      return { pageCount: result.pageCount, diagnostics: result.diagnostics };
    }
    const maximum = request.maximumBytes ?? this.limits.outputBytes;
    if (request.method === 'render') {
      if (data.encoding !== undefined || !withinUtf8Limit(data.result, maximum)) throw invalid();
      return data.result;
    }
    if (request.method === 'export') {
      if (!(data.result instanceof Uint8Array) || data.result.byteLength > maximum
          || data.result.byteOffset !== 0 || data.result.byteLength !== data.result.buffer.byteLength) throw invalid();
      return data.result;
    }
    throw invalid();
  }

  request(method, params, { signal, timeoutMs = 30_000, onProgress } = {}) {
    if (this.closed) return Promise.reject(failure('PDF_WORKER_CLOSED', 'worker is closed'));
    if (!Number.isSafeInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > 2_147_483_647) {
      return Promise.reject(failure('PDF_DEADLINE_INVALID', 'timeoutMs must be a positive timer interval'));
    }
    if (signal?.aborted) return Promise.reject(failure('PDF_IMPORT_CANCELLED', 'request cancelled'));
    const id = ++this.nextId;
    return new Promise((resolve, reject) => {
      const finish = (error, result) => {
        if (!this.pending.delete(id)) return;
        clearTimeout(timer);
        signal?.removeEventListener('abort', abort);
        error ? reject(error) : resolve(result);
      };
      // A message cannot interrupt synchronous WASM. Terminate the Worker from its owner.
      const abort = () => this.close(failure('PDF_IMPORT_CANCELLED', 'request cancelled'));
      const timer = setTimeout(() => this.close(failure('PDF_IMPORT_TIMEOUT', 'worker deadline exceeded')), timeoutMs);
      this.pending.set(id, { finish, onProgress, method, maximumBytes: params.maximumBytes, signal,
        deadline: performance.now() + timeoutMs });
      try {
        signal?.addEventListener('abort', abort, { once: true });
        // Only the private input copy is transferred; caller bytes remain available.
        this.worker.postMessage({ id, method, ...params }, params.bytes instanceof Uint8Array ? [params.bytes.buffer] : []);
      } catch (error) {
        this.close(failure('PDF_WORKER_PROTOCOL', String(error)));
      }
    });
  }

  close(error) {
    if (this.closed) return;
    this.closed = true;
    this.worker.onmessage = this.worker.onerror = this.worker.onmessageerror = null;
    this.worker.terminate();
    for (const request of this.pending.values()) request.finish(error);
  }
}

// Count without allocating a second, fully encoded copy of a response string.
function withinUtf8Limit(text, maximum) {
  if (typeof text !== 'string' || text.length > maximum) return false;
  let bytes = 0;
  for (let i = 0; i < text.length; i++) {
    const unit = text.charCodeAt(i);
    if (unit < 0x80) bytes++;
    else if (unit < 0x800) bytes += 2;
    else if (unit >= 0xd800 && unit <= 0xdbff && i + 1 < text.length
        && text.charCodeAt(i + 1) >= 0xdc00 && text.charCodeAt(i + 1) <= 0xdfff) {
      bytes += 4; i++;
    } else bytes += 3;
    if (bytes > maximum) return false;
  }
  return true;
}
