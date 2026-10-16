/** Opt-in browser PDF session. The caller owns this instance and must dispose it. */
export class PdfImportSession {
  constructor({ workerFactory = () => new Worker(new URL('./import-worker.js', import.meta.url), { type: 'module' }) } = {}) {
    this._workerFactory = workerFactory;
    this._active = null;
    this._candidate = null;
    this._importing = false;
    this._disposed = false;
  }

  async importPdf(bytes, options = {}, { signal, timeoutMs = 30_000 } = {}) {
    this._assertAlive();
    if (this._importing) throw failure('PDF_IMPORT_BUSY', 'another import is in progress');
    if (!(bytes instanceof Uint8Array)) throw failure('PDF_INVALID_INPUT', 'expected Uint8Array');
    // Before structured clone. Rust independently applies the same frontend limit.
    if (bytes.byteLength > 16 * 1024 * 1024) throw failure('PDF_RESOURCE_LIMIT', 'input exceeds 16 MiB');
    this._importing = true;
    try {
      const candidate = this._candidate ??= new Channel(this._workerFactory());
      // Copy only the selected view, not its possibly much larger backing buffer.
      const result = await candidate.request('importPdf', { bytes: bytes.slice(), options }, { signal, timeoutMs });
      // No await between successful response and publication. Failed/cancelled candidates
      // never become the active session, even when synchronous WASM had already completed.
      this._assertAlive();
      this._active?.close(failure('PDF_SESSION_REPLACED', 'document replaced'));
      this._active = candidate;
      this._candidate = null;
      return result;
    } finally {
      if (this._candidate?.closed) this._candidate = null;
      this._importing = false;
    }
  }

  inspect(options) { return this._request('inspect', {}, options); }
  renderPageSvg(page, options) { return this._request('render', { page }, options); }
  exportDocument(format = 'hwpx', options) { return this._request('export', { format }, options); }

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

class Channel {
  constructor(worker) {
    this.worker = worker;
    this.closed = false;
    this.pending = new Map();
    this.nextId = 0;
    worker.onmessage = ({ data }) => {
      const request = this.pending.get(data?.id);
      if (!request) return;
      if (data.error?.fatal) {
        this.close(failure(data.error.code, data.error.message));
      } else {
        request.finish(data.error ? failure(data.error.code, data.error.message) : null, data.result);
      }
    };
    worker.onerror = () => this.close(failure('PDF_WORKER_FAILED', 'worker execution failed'));
    worker.onmessageerror = () => this.close(failure('PDF_WORKER_FAILED', 'worker response could not be decoded'));
  }

  request(method, params, { signal, timeoutMs = 30_000 } = {}) {
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
      this.pending.set(id, { finish });
      try {
        signal?.addEventListener('abort', abort, { once: true });
        // Only the private input copy is transferred; caller bytes remain available.
        this.worker.postMessage({ id, method, ...params }, params.bytes instanceof Uint8Array ? [params.bytes.buffer] : []);
      } catch (error) {
        finish(failure('PDF_WORKER_PROTOCOL', String(error)));
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
