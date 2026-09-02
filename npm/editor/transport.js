const PROTOCOL_VERSION = 1;
const CAPABILITIES = [
  'transferable-array-buffer',
  'hml-export',
  'renderer-diagnostics-v1',
  'font-decision-trace-v1',
  'notify-saved-v1',
  'document-state-v1',
  'selection-context-v1',
  'document-agent-command-v1',
  'target-navigation-v1',
  'document-change-events-v1',
];
const LONG_RUNNING_METHODS = new Set([
  'loadFile', 'getFontDecisionTrace', 'exportHwp', 'exportHwpVerify', 'exportHwpx', 'exportHml',
]);

export function requestTimeoutFor(method, configuredTimeout) {
  if (configuredTimeout != null) return configuredTimeout;
  return LONG_RUNNING_METHODS.has(method) ? 60000 : 10000;
}

function sessionId() {
  const secureRandom = globalThis.crypto;
  if (typeof secureRandom?.randomUUID === 'function') return secureRandom.randomUUID();
  if (typeof secureRandom?.getRandomValues !== 'function') {
    throw new Error('Secure random generation is unavailable');
  }
  const bytes = secureRandom.getRandomValues(new Uint8Array(16));
  return Array.from(bytes, (byte) => byte.toString(16).padStart(2, '0')).join('');
}

function copiedBinary(value) {
  if (value instanceof ArrayBuffer) return new Uint8Array(value).slice();
  if (ArrayBuffer.isView(value)) {
    return new Uint8Array(value.buffer, value.byteOffset, value.byteLength).slice();
  }
  return null;
}

function prepareParams(params) {
  const data = copiedBinary(params?.data);
  if (!data) return { params, transfer: [] };
  return { params: { ...params, data }, transfer: [data.buffer] };
}

function isResponseEnvelope(message, legacy, sessionId) {
  if (message?.type !== 'rhwp-response' || !Number.isSafeInteger(message.id)) return false;
  if (legacy) return true;
  if (message.version !== PROTOCOL_VERSION || message.sessionId !== sessionId) return false;
  const hasResult = Object.prototype.hasOwnProperty.call(message, 'result');
  const hasError = Object.prototype.hasOwnProperty.call(message, 'error');
  if (hasResult === hasError) return false;
  const expectedEnvelopeKeys = ['id', hasError ? 'error' : 'result', 'sessionId', 'type', 'version'].sort();
  const envelopeKeys = Object.keys(message).sort();
  if (envelopeKeys.length !== expectedEnvelopeKeys.length
      || envelopeKeys.some((key, index) => key !== expectedEnvelopeKeys[index])) return false;
  if (!hasError) return true;
  if (typeof message.error?.code !== 'string' || typeof message.error?.message !== 'string'
      || (message.error.recovered !== undefined && typeof message.error.recovered !== 'boolean')) {
    return false;
  }
  const allowedErrorKeys = ['code', 'message', 'recovered', 'supportedVersions'];
  return Object.keys(message.error).every(key => allowedErrorKeys.includes(key));
}

export class EditorTransport {
  constructor(iframe, studioUrl, options = {}) {
    this._iframe = iframe;
    const targetUrl = new URL(studioUrl, globalThis.location?.href);
    if (targetUrl.protocol !== 'http:' && targetUrl.protocol !== 'https:') {
      throw new Error('studioUrl must use HTTP(S)');
    }
    this._targetOrigin = targetUrl.origin;
    this._window = options.window || window;
    this._requestTimeoutMs = options.requestTimeoutMs;
    this._handshakeTimeoutMs = options.handshakeTimeoutMs ?? 1000;
    this._sessionId = sessionId();
    this._nextId = 0;
    this._pending = new Map();
    this._listeners = new Map();
    this._port = null;
    this._peerCapabilities = new Set();
    this._legacy = false;
    this._destroyed = false;
    this._onLegacyMessage = (event) => this._handleLegacyMessage(event);
  }

  connect() {
    if (this._destroyed) return Promise.reject(new Error('Editor destroyed'));
    const channel = new MessageChannel();
    this._port = channel.port1;
    this._port.onmessage = (event) => this._handlePortMessage(event.data);
    this._port.start();
    return new Promise((resolve, reject) => {
      this._connectResolve = resolve;
      this._connectReject = reject;
      this._connectTimer = setTimeout(() => this._useLegacy(), this._handshakeTimeoutMs);
      this._iframe.contentWindow.postMessage({
        type: 'rhwp-connect', version: PROTOCOL_VERSION, sessionId: this._sessionId,
        capabilities: CAPABILITIES,
      }, this._targetOrigin, [channel.port2]);
    });
  }

  request(method, params = {}) {
    if (this._destroyed) return Promise.reject(new Error('Editor destroyed'));
    const id = ++this._nextId;
    const prepared = prepareParams(params);
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this._pending.delete(id);
        reject(new Error(`Request timeout: ${method}`));
      }, requestTimeoutFor(method, this._requestTimeoutMs));
      this._pending.set(id, { resolve, reject, timeout });
      try {
        this._send({
          type: 'rhwp-request', version: PROTOCOL_VERSION,
          sessionId: this._sessionId, id, method, params: prepared.params,
        }, prepared.transfer);
      } catch (error) {
        clearTimeout(timeout);
        this._pending.delete(id);
        reject(error);
      }
    });
  }

  supports(capability) {
    return this._peerCapabilities.has(capability);
  }

  on(event, listener) {
    if (this._destroyed) throw new Error('Editor destroyed');
    if (typeof event !== 'string' || typeof listener !== 'function') {
      throw new TypeError('event and listener are required');
    }
    if (!this._listeners.has(event)) this._listeners.set(event, new Set());
    this._listeners.get(event).add(listener);
    return () => this._listeners.get(event)?.delete(listener);
  }

  destroy() {
    if (this._destroyed) return;
    this._destroyed = true;
    clearTimeout(this._connectTimer);
    this._port && (this._port.onmessage = null);
    this._port?.close();
    this._rejectConnect(new Error('Editor destroyed'));
    this._window.removeEventListener('message', this._onLegacyMessage);
    for (const pending of this._pending.values()) {
      clearTimeout(pending.timeout);
      pending.reject(new Error('Editor destroyed'));
    }
    this._pending.clear();
    this._listeners.clear();
  }

  _send(message, transfer) {
    if (this._legacy) {
      const { version, sessionId: ignored, ...legacyMessage } = message;
      this._iframe.contentWindow.postMessage(legacyMessage, this._targetOrigin, transfer);
      return;
    }
    this._port.postMessage(message, transfer);
  }

  _handlePortMessage(message) {
    if (message?.type === 'rhwp-connected'
        && message.version === PROTOCOL_VERSION
        && message.sessionId === this._sessionId
        && message.capabilities?.includes('transferable-array-buffer')) {
      clearTimeout(this._connectTimer);
      this._peerCapabilities = new Set(message.capabilities);
      const resolve = this._connectResolve;
      this._connectResolve = null;
      this._connectReject = null;
      resolve?.();
      return;
    }
    if (message?.type === 'rhwp-event') {
      const keys = Object.keys(message).sort();
      const expectedKeys = ['event', 'payload', 'sessionId', 'type', 'version'];
      if (message.version !== PROTOCOL_VERSION
          || message.sessionId !== this._sessionId
          || !this._peerCapabilities.has('document-change-events-v1')
          || keys.length !== expectedKeys.length
          || keys.some((key, index) => key !== expectedKeys[index])
          || typeof message.event !== 'string') return;
      for (const listener of this._listeners.get(message.event) || []) {
        try { listener(message.payload); } catch { /* 한 listener가 다른 listener를 막지 않는다. */ }
      }
      return;
    }
    if (message?.type === 'rhwp-connect-error'
        && message.version === PROTOCOL_VERSION
        && message.sessionId === this._sessionId
        && typeof message.error?.message === 'string') {
      const error = new Error(message.error.message);
      error.code = message.error.code;
      error.supportedVersions = message.error.supportedVersions;
      this._rejectConnect(error);
      return;
    }
    this._handleResponse(message);
  }

  _handleLegacyMessage(event) {
    if (event.source !== this._iframe.contentWindow || event.origin !== this._targetOrigin) return;
    this._handleResponse(event.data, true);
  }

  _handleResponse(message, legacy = false) {
    if (legacy) {
      if (!isResponseEnvelope(message, true, this._sessionId)) return;
    } else if (message?.type !== 'rhwp-response'
        || !Number.isSafeInteger(message.id)
        || message.sessionId !== this._sessionId) {
      return;
    }
    const pending = this._pending.get(message.id);
    if (!pending) return;
    if (!legacy && Number.isSafeInteger(message.version)
        && message.version !== PROTOCOL_VERSION) {
      this._rejectPendingResponse(message.id, pending, 'UNSUPPORTED_VERSION',
        `Unsupported embed protocol version: ${message.version}`, [PROTOCOL_VERSION]);
      return;
    }
    if (!isResponseEnvelope(message, legacy, this._sessionId)) {
      setTimeout(() => this._rejectPendingResponse(
        message.id, pending, 'INVALID_RESPONSE', 'Invalid response envelope',
      ), 0);
      return;
    }
    this._pending.delete(message.id);
    clearTimeout(pending.timeout);
    if (message.error) {
      const error = new Error(message.error.message || message.error);
      if (message.error.code) error.code = message.error.code;
      if (typeof message.error.recovered === 'boolean') error.recovered = message.error.recovered;
      pending.reject(error);
    }
    else pending.resolve(message.result);
  }

  _rejectPendingResponse(id, pending, code, message, supportedVersions) {
    if (this._pending.get(id) !== pending) return;
    this._pending.delete(id);
    clearTimeout(pending.timeout);
    const error = new Error(message);
    error.code = code;
    if (supportedVersions) error.supportedVersions = supportedVersions;
    pending.reject(error);
  }

  _rejectConnect(error) {
    if (!this._connectReject) return;
    clearTimeout(this._connectTimer);
    this._port && (this._port.onmessage = null);
    this._port?.close();
    this._port = null;
    const reject = this._connectReject;
    this._connectResolve = null;
    this._connectReject = null;
    reject(error);
  }

  _useLegacy() {
    if (this._destroyed || !this._connectResolve) return;
    this._port && (this._port.onmessage = null);
    this._port?.close();
    this._port = null;
    this._legacy = true;
    this._peerCapabilities.clear();
    this._window.addEventListener('message', this._onLegacyMessage);
    const resolve = this._connectResolve;
    this._connectResolve = null;
    this._connectReject = null;
    resolve();
  }
}
