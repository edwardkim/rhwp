const SHA256 = /^[0-9a-f]{64}$/;

function contractError(code, message) {
  const error = new TypeError(message);
  error.code = code;
  return error;
}

function record(value, label, code) {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    throw contractError(code, `${label} must be an object`);
  }
  return value;
}

function exactKeys(value, keys, label, code) {
  const actual = Object.keys(value).sort();
  const expected = [...keys].sort();
  if (actual.length !== expected.length || actual.some((key, index) => key !== expected[index])) {
    throw contractError(code, `${label} has invalid fields`);
  }
}

function safeInteger(value, label, minimum, code) {
  if (!Number.isSafeInteger(value) || value < minimum) {
    throw contractError(code, `${label} must be a safe integer >= ${minimum}`);
  }
}

function sha256(value, label, code) {
  if (typeof value !== 'string' || !SHA256.test(value)) {
    throw contractError(code, `${label} must be a lowercase SHA-256 hex digest`);
  }
}

function boolean(value, label, code) {
  if (typeof value !== 'boolean') throw contractError(code, `${label} must be boolean`);
}

export function assertCapability(transport, capability) {
  if (transport.supports(capability)) return;
  const error = new Error(`${capability} is not supported by this Studio`);
  error.code = 'CAPABILITY_UNSUPPORTED';
  throw error;
}

export function validateBodyParagraphTarget(value, code = 'INVALID_COMMAND') {
  const target = record(value, 'target', code);
  exactKeys(
    target,
    ['kind', 'section', 'paragraph', 'charOffset', 'length'],
    'target',
    code,
  );
  if (target.kind !== 'body_paragraph') {
    throw contractError(code, 'target.kind must be body_paragraph');
  }
  safeInteger(target.section, 'target.section', 0, code);
  safeInteger(target.paragraph, 'target.paragraph', 0, code);
  if (target.charOffset !== 0) throw contractError(code, 'target.charOffset must be 0');
  safeInteger(target.length, 'target.length', 0, code);
  if (target.length > 4000) throw contractError(code, 'target.length must be <= 4000');
  return target;
}

export function validateApplyTextCommand(value) {
  const code = 'INVALID_COMMAND';
  const command = record(value, 'command', code);
  exactKeys(command, [
    'schemaVersion', 'commandId', 'expectedDocumentEpoch', 'expectedChangeSeq',
    'expectedDocumentSha256', 'target', 'expectedBeforeSha256',
    'expectedFormatSha256', 'expectedAdjacentContextSha256', 'replacement',
  ], 'command', code);
  if (command.schemaVersion !== 1) throw contractError(code, 'command.schemaVersion must be 1');
  if (typeof command.commandId !== 'string'
      || command.commandId.length < 1 || command.commandId.length > 128) {
    throw contractError(code, 'command.commandId must be a string in 1..=128 characters');
  }
  safeInteger(command.expectedDocumentEpoch, 'command.expectedDocumentEpoch', 1, code);
  safeInteger(command.expectedChangeSeq, 'command.expectedChangeSeq', 0, code);
  sha256(command.expectedDocumentSha256, 'command.expectedDocumentSha256', code);
  validateBodyParagraphTarget(command.target, code);
  sha256(command.expectedBeforeSha256, 'command.expectedBeforeSha256', code);
  sha256(command.expectedFormatSha256, 'command.expectedFormatSha256', code);
  sha256(command.expectedAdjacentContextSha256, 'command.expectedAdjacentContextSha256', code);
  if (typeof command.replacement !== 'string'
      || Array.from(command.replacement).length > 4000) {
    throw contractError(code, 'command.replacement must be a string with at most 4000 characters');
  }
  if (/[\u0000-\u001f\u007f]/u.test(command.replacement)) {
    throw contractError(code, 'command.replacement must not contain control characters');
  }
  return command;
}

export function validateRevertTextCommand(value) {
  const code = 'INVALID_COMMAND';
  const command = record(value, 'command', code);
  exactKeys(command, [
    'schemaVersion', 'commandId', 'expectedDocumentEpoch', 'expectedChangeSeq',
    'expectedAfterDocumentSha256', 'expectedAfterSha256',
  ], 'command', code);
  if (command.schemaVersion !== 1) throw contractError(code, 'command.schemaVersion must be 1');
  if (typeof command.commandId !== 'string'
      || command.commandId.length < 1 || command.commandId.length > 128) {
    throw contractError(code, 'command.commandId must be a string in 1..=128 characters');
  }
  safeInteger(command.expectedDocumentEpoch, 'command.expectedDocumentEpoch', 1, code);
  safeInteger(command.expectedChangeSeq, 'command.expectedChangeSeq', 0, code);
  sha256(command.expectedAfterDocumentSha256, 'command.expectedAfterDocumentSha256', code);
  sha256(command.expectedAfterSha256, 'command.expectedAfterSha256', code);
  return command;
}

export function validateDocumentState(value) {
  const code = 'INVALID_RESPONSE';
  const state = record(value, 'document state', code);
  exactKeys(state, [
    'schemaVersion', 'format', 'documentEpoch', 'changeSeq', 'dirty', 'pageCount',
    'documentSha256',
  ], 'document state', code);
  if (state.schemaVersion !== 1) throw contractError(code, 'document state schemaVersion must be 1');
  if (state.format !== 'hwp' && state.format !== 'hwpx') {
    throw contractError(code, 'document state format must be hwp or hwpx');
  }
  safeInteger(state.documentEpoch, 'document state documentEpoch', 1, code);
  safeInteger(state.changeSeq, 'document state changeSeq', 0, code);
  boolean(state.dirty, 'document state dirty', code);
  safeInteger(state.pageCount, 'document state pageCount', 1, code);
  sha256(state.documentSha256, 'document state documentSha256', code);
  return state;
}

export function validateSelectionContext(value) {
  const code = 'INVALID_RESPONSE';
  const selection = record(value, 'selection context', code);
  exactKeys(selection, [
    'schemaVersion', 'documentEpoch', 'changeSeq', 'page', 'editable', 'collapsed',
    'target', 'selectedTextSha256',
  ], 'selection context', code);
  if (selection.schemaVersion !== 1) {
    throw contractError(code, 'selection context schemaVersion must be 1');
  }
  safeInteger(selection.documentEpoch, 'selection context documentEpoch', 1, code);
  safeInteger(selection.changeSeq, 'selection context changeSeq', 0, code);
  safeInteger(selection.page, 'selection context page', 1, code);
  boolean(selection.editable, 'selection context editable', code);
  boolean(selection.collapsed, 'selection context collapsed', code);
  if (selection.target !== null) validateBodyParagraphTarget(selection.target, code);
  if (selection.selectedTextSha256 !== null) {
    sha256(selection.selectedTextSha256, 'selection context selectedTextSha256', code);
  }
  return selection;
}

export function validateTextCommandReceipt(value) {
  const code = 'INVALID_RESPONSE';
  const receipt = record(value, 'text command receipt', code);
  exactKeys(receipt, [
    'schemaVersion', 'commandId', 'operation', 'documentEpoch', 'beforeChangeSeq',
    'afterChangeSeq', 'beforeDocumentSha256', 'afterDocumentSha256',
    'beforeTextSha256', 'afterTextSha256', 'formatSha256',
    'adjacentContextSha256', 'pageCountBefore', 'pageCountAfter', 'target',
  ], 'text command receipt', code);
  if (receipt.schemaVersion !== 1) throw contractError(code, 'receipt.schemaVersion must be 1');
  if (typeof receipt.commandId !== 'string'
      || receipt.commandId.length < 1 || receipt.commandId.length > 128) {
    throw contractError(code, 'receipt.commandId is invalid');
  }
  if (receipt.operation !== 'apply' && receipt.operation !== 'revert') {
    throw contractError(code, 'receipt.operation is invalid');
  }
  safeInteger(receipt.documentEpoch, 'receipt.documentEpoch', 1, code);
  safeInteger(receipt.beforeChangeSeq, 'receipt.beforeChangeSeq', 0, code);
  safeInteger(receipt.afterChangeSeq, 'receipt.afterChangeSeq', 1, code);
  if (receipt.afterChangeSeq !== receipt.beforeChangeSeq + 1) {
    throw contractError(code, 'receipt change sequence is invalid');
  }
  for (const key of [
    'beforeDocumentSha256', 'afterDocumentSha256', 'beforeTextSha256',
    'afterTextSha256', 'formatSha256', 'adjacentContextSha256',
  ]) sha256(receipt[key], `receipt.${key}`, code);
  safeInteger(receipt.pageCountBefore, 'receipt.pageCountBefore', 1, code);
  safeInteger(receipt.pageCountAfter, 'receipt.pageCountAfter', 1, code);
  validateBodyParagraphTarget(receipt.target, code);
  return receipt;
}

export function validateFocusTargetResult(value) {
  const code = 'INVALID_RESPONSE';
  const result = record(value, 'focus target result', code);
  exactKeys(result, ['focused', 'page'], 'focus target result', code);
  boolean(result.focused, 'focus target result focused', code);
  safeInteger(result.page, 'focus target result page', 1, code);
  return result;
}

export function validateDocumentChangedEvent(value) {
  const code = 'INVALID_RESPONSE';
  const event = record(value, 'document changed event', code);
  exactKeys(event, [
    'schemaVersion', 'reason', 'documentEpoch', 'changeSeq', 'commandId',
  ], 'document changed event', code);
  if (event.schemaVersion !== 1) {
    throw contractError(code, 'document changed event schemaVersion must be 1');
  }
  if (event.reason !== 'agent_apply' && event.reason !== 'agent_revert') {
    throw contractError(code, 'document changed event reason is invalid');
  }
  safeInteger(event.documentEpoch, 'document changed event documentEpoch', 1, code);
  safeInteger(event.changeSeq, 'document changed event changeSeq', 1, code);
  if (typeof event.commandId !== 'string'
      || event.commandId.length < 1 || event.commandId.length > 128) {
    throw contractError(code, 'document changed event commandId is invalid');
  }
  return event;
}
