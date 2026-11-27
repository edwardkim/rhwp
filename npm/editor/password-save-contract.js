export const PASSWORD_SAVE_CAPABILITY = 'password-save-v1';
export const PASSWORD_PROTECTION = 'password-protected';

const MESSAGES = Object.freeze({
  INVALID_REQUEST: 'Invalid password-save request.',
  INVALID_RESPONSE: 'Invalid password-save response.',
  UNSUPPORTED_CAPABILITY: 'password-save-v1 was not negotiated.',
  UNSUPPORTED_TRANSPORT: 'Password save requires a bound MessagePort.',
  UNSUPPORTED_VERSION: 'Unsupported embed protocol version.',
  NO_DOCUMENT: 'No document is available for password save.',
  DOCUMENT_OPEN_PENDING: 'Wait for the current document to finish opening.',
  UNSUPPORTED_SAVE_FORMAT: 'The loaded document format does not support password save.',
  STALE_SAVE_REVISION: 'The document or edit revision has changed.',
  INVALID_SAVE_EXPORT: 'The acknowledgment does not match an active export.',
  PASSWORD_SAVE_FAILED: 'The password-save operation failed.',
});

export class PasswordSaveError extends Error {
  constructor(code) {
    const knownCode = typeof code === 'string' && Object.hasOwn(MESSAGES, code)
      ? code : 'PASSWORD_SAVE_FAILED';
    super(MESSAGES[knownCode]);
    this.code = knownCode;
  }
}

export function isPasswordSaveMethod(method) {
  return method === 'getSaveProtectionState'
    || method === 'exportPasswordProtectedDocument'
    || method === 'acknowledgeHostSave';
}

function hasShape(value, required, optional = []) {
  return value !== null && typeof value === 'object' && Object.prototype.toString.call(value) === '[object Object]'
    && required.every(key => Object.hasOwn(value, key))
    && Reflect.ownKeys(value).every(key => required.includes(key) || optional.includes(key));
}

function isCounter(value) {
  return Number.isSafeInteger(value) && value >= 0;
}

function isFormat(value) {
  return value === 'hwp' || value === 'hwpx';
}

function isExportId(value) {
  return typeof value === 'string' && /^[0-9a-f]{32}$/.test(value);
}

function isPassword(value) {
  // Match Studio's minimum and bound the UTF-8 input passed to WASM.
  return typeof value === 'string' && value.length >= 5 && value.length <= 4096
    && new TextEncoder().encode(value).byteLength <= 4096;
}

const revisionKeys = ['documentEpoch', 'changeSeq'];
const receiptKeys = ['schemaVersion', 'exportId', 'format', 'protection', ...revisionKeys];

function isRevision(value) {
  return isCounter(value.documentEpoch) && isCounter(value.changeSeq);
}

function isReceipt(value) {
  return value.schemaVersion === 1 && isExportId(value.exportId) && isFormat(value.format)
    && value.protection === PASSWORD_PROTECTION && isRevision(value);
}

export function validateSaveProtectionQuery(value) {
  if (value !== undefined && !hasShape(value, [])) throw new PasswordSaveError('INVALID_REQUEST');
}

export function validatePasswordExportRequest(value) {
  if (!hasShape(value, ['schemaVersion', 'format', 'password', 'expectedDocumentEpoch', 'expectedChangeSeq'])
    || value.schemaVersion !== 1 || !isFormat(value.format) || !isPassword(value.password)
    || !isCounter(value.expectedDocumentEpoch) || !isCounter(value.expectedChangeSeq)) {
    throw new PasswordSaveError('INVALID_REQUEST');
  }
  return value;
}

export function validateHostSaveRequest(value) {
  if (!hasShape(value, receiptKeys, ['fileName']) || !isReceipt(value)
    || (Object.hasOwn(value, 'fileName') && (typeof value.fileName !== 'string' || !value.fileName))) {
    throw new PasswordSaveError('INVALID_REQUEST');
  }
  return value;
}

export function validateSaveProtectionState(value) {
  if (!hasShape(value, [
    'schemaVersion', 'persistedProtection', 'requiresPasswordForSave', 'loadedFormat', ...revisionKeys,
  ]) || value.schemaVersion !== 1 || !isRevision(value)
    || !['plain', PASSWORD_PROTECTION].includes(value.persistedProtection)
    || value.requiresPasswordForSave !== (value.persistedProtection === PASSWORD_PROTECTION)
    || !['hwp3', 'hwp5', 'hwpx'].includes(value.loadedFormat)) {
    throw new PasswordSaveError('INVALID_RESPONSE');
  }
  return value;
}

function isContentLoss(value, format) {
  return hasShape(value, ['schemaVersion', 'outputFormat', 'count', 'losses'])
    && value.schemaVersion === 1 && value.outputFormat === format
    && Array.isArray(value.losses) && value.count === value.losses.length
    && Array.from(value.losses).every(loss => hasShape(loss, ['code', 'subject', 'path', 'reason'], ['resourceId'])
      && ['binaryContentEmptied', 'controlOmitted', 'metadataReduced'].includes(loss.code)
      && ['binaryData', 'control', 'fieldParameters'].includes(loss.subject)
      && typeof loss.path === 'string'
      && ['resourceReadFailedOrLimitExceeded', 'storedCompressionMismatch', 'rawPassthroughUnavailable',
        'unsupportedByOutputFormat'].includes(loss.reason)
      && (!Object.hasOwn(loss, 'resourceId') || isCounter(loss.resourceId)));
}

export function validatePasswordProtectedExport(value, request) {
  if (!hasShape(value, [...receiptKeys, 'bytes', 'byteCount', 'contentLoss']) || !isReceipt(value)
    || value.format !== request.format || value.documentEpoch !== request.expectedDocumentEpoch
    || value.changeSeq !== request.expectedChangeSeq || !(value.bytes instanceof Uint8Array)
    || value.bytes.byteLength === 0 || value.byteCount !== value.bytes.byteLength
    || !isContentLoss(value.contentLoss, value.format)) {
    throw new PasswordSaveError('INVALID_RESPONSE');
  }
  return value;
}

export function validateHostSaveResult(value) {
  if (!hasShape(value, ['ok', 'wasDirty']) || value.ok !== true || typeof value.wasDirty !== 'boolean') {
    throw new PasswordSaveError('INVALID_RESPONSE');
  }
  return value;
}
